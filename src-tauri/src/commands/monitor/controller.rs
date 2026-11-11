//! Cycle de vie du thread de monitoring : start/stop serialises sous le Mutex du
//! slot, jeton d'arret propre a chaque thread.
//!
//! Un drapeau global partage entre threads ne tient pas le double montage de
//! React StrictMode (start, stop, start enchaines sans attente) : le `stop` du
//! premier montage couperait le thread du second. Ici chaque thread a son jeton ;
//! un nouveau thread attend la fin de l'ancien avant de boucler (jamais deux
//! boucles en meme temps), et `stop` ne bloque jamais sur un `join`.

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const PREVIOUS_THREAD_GRACE: Duration = Duration::from_secs(5);

fn wait_for_exit(handle: JoinHandle<()>, grace: Duration) -> bool {
    let deadline = Instant::now() + grace;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
    if handle.join().is_err() {
        crate::utils::logger::warn("monitor", "l'ancien thread de monitoring s'est termine par un panic");
    }
    true
}

pub(super) struct StopToken {
    stopped: Mutex<bool>,
    wake: Condvar,
}

impl StopToken {
    pub(super) fn new() -> Self {
        Self {
            stopped: Mutex::new(false),
            wake: Condvar::new(),
        }
    }

    pub(super) fn stop(&self) {
        *self.stopped.lock().unwrap_or_else(|p| p.into_inner()) = true;
        self.wake.notify_all();
    }

    pub(super) fn is_stopped(&self) -> bool {
        *self.stopped.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Dort au plus `duration`, reveille des l'arret. Renvoie `true` si arrete.
    pub(super) fn sleep(&self, duration: Duration) -> bool {
        let guard = self.stopped.lock().unwrap_or_else(|p| p.into_inner());
        let (guard, _) = self
            .wake
            .wait_timeout_while(guard, duration, |stopped| !*stopped)
            .unwrap_or_else(|p| p.into_inner());
        *guard
    }
}

struct Running {
    token: Arc<StopToken>,
    handle: JoinHandle<()>,
}

pub(super) struct MonitorController {
    slot: Mutex<Option<Running>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum StartOutcome {
    Started,
    AlreadyRunning,
}

impl MonitorController {
    pub(super) const fn new() -> Self {
        Self {
            slot: Mutex::new(None),
        }
    }

    pub(super) fn start<F>(&self, body: F) -> StartOutcome
    where
        F: FnOnce(Arc<StopToken>) + Send + 'static,
    {
        self.start_with_grace(body, PREVIOUS_THREAD_GRACE)
    }

    fn start_with_grace<F>(&self, body: F, grace: Duration) -> StartOutcome
    where
        F: FnOnce(Arc<StopToken>) + Send + 'static,
    {
        let mut slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(current) = slot.as_ref() {
            if !current.token.is_stopped() && !current.handle.is_finished() {
                return StartOutcome::AlreadyRunning;
            }
        }
        let previous = slot.take();
        let token = Arc::new(StopToken::new());
        let thread_token = Arc::clone(&token);
        let handle = thread::spawn(move || {
            if let Some(previous) = previous {
                if !wait_for_exit(previous.handle, grace) {
                    crate::utils::logger::warn(
                        "monitor",
                        "l'ancien thread de monitoring ne repond plus (requete WMI bloquee ?), il est abandonne",
                    );
                }
            }
            if thread_token.is_stopped() {
                return;
            }
            body(thread_token);
        });
        *slot = Some(Running { token, handle });
        StartOutcome::Started
    }

    pub(super) fn stop(&self) {
        let slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(current) = slot.as_ref() {
            current.token.stop();
        }
    }

    #[cfg(test)]
    fn is_running(&self) -> bool {
        let slot = self.slot.lock().unwrap_or_else(|p| p.into_inner());
        slot.as_ref()
            .is_some_and(|r| !r.token.is_stopped() && !r.handle.is_finished())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[derive(Default)]
    struct Probe {
        live: AtomicUsize,
        max_live: AtomicUsize,
        started: AtomicUsize,
    }

    fn body(probe: Arc<Probe>, exit_delay: Duration) -> impl FnOnce(Arc<StopToken>) + Send + 'static {
        move |token| {
            probe.started.fetch_add(1, Ordering::SeqCst);
            let live = probe.live.fetch_add(1, Ordering::SeqCst) + 1;
            probe.max_live.fetch_max(live, Ordering::SeqCst);
            while !token.sleep(Duration::from_millis(5)) {}
            thread::sleep(exit_delay);
            probe.live.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if condition() {
                return true;
            }
            thread::sleep(Duration::from_millis(2));
        }
        false
    }

    #[test]
    fn second_start_while_running_does_not_spawn_another_thread() {
        let controller = MonitorController::new();
        let probe = Arc::new(Probe::default());
        assert_eq!(controller.start(body(probe.clone(), Duration::ZERO)), StartOutcome::Started);
        assert!(wait_until(|| probe.started.load(Ordering::SeqCst) == 1));
        assert_eq!(
            controller.start(body(probe.clone(), Duration::ZERO)),
            StartOutcome::AlreadyRunning
        );
        thread::sleep(Duration::from_millis(30));
        assert_eq!(probe.started.load(Ordering::SeqCst), 1);
        controller.stop();
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 0));
    }

    #[test]
    fn strict_mode_start_stop_start_keeps_new_thread_alive_and_never_overlaps() {
        let controller = MonitorController::new();
        let probe = Arc::new(Probe::default());
        controller.start(body(probe.clone(), Duration::from_millis(80)));
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 1));
        controller.stop();
        assert_eq!(
            controller.start(body(probe.clone(), Duration::from_millis(80))),
            StartOutcome::Started
        );
        assert!(wait_until(|| probe.started.load(Ordering::SeqCst) == 2));
        thread::sleep(Duration::from_millis(150));
        assert!(controller.is_running(), "le stop du premier montage a coupe le nouveau thread");
        assert_eq!(probe.live.load(Ordering::SeqCst), 1);
        assert_eq!(probe.max_live.load(Ordering::SeqCst), 1, "deux boucles ont tourne en meme temps");
        controller.stop();
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 0));
    }

    #[test]
    fn stop_does_not_wait_for_a_slow_thread() {
        let controller = MonitorController::new();
        let probe = Arc::new(Probe::default());
        controller.start(body(probe.clone(), Duration::from_millis(400)));
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 1));
        let before = Instant::now();
        controller.stop();
        assert!(before.elapsed() < Duration::from_millis(100), "stop a bloque {:?}", before.elapsed());
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 0));
    }

    #[test]
    fn a_blocked_previous_thread_does_not_keep_the_new_one_from_starting() {
        let controller = MonitorController::new();
        let release = Arc::new(AtomicUsize::new(0));
        let blocked = Arc::clone(&release);
        controller.start_with_grace(
            move |_token| {
                while blocked.load(Ordering::SeqCst) == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            },
            Duration::from_millis(50),
        );
        thread::sleep(Duration::from_millis(30));
        controller.stop();

        let probe = Arc::new(Probe::default());
        let before = Instant::now();
        controller.start_with_grace(body(probe.clone(), Duration::ZERO), Duration::from_millis(100));
        assert!(wait_until(|| probe.started.load(Ordering::SeqCst) == 1), "le nouveau thread n'a jamais demarre");
        assert!(before.elapsed() < Duration::from_secs(2), "{:?}", before.elapsed());
        assert_eq!(probe.live.load(Ordering::SeqCst), 1);

        controller.stop();
        release.store(1, Ordering::SeqCst);
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 0));
    }

    #[test]
    fn a_previous_thread_that_exits_in_time_is_still_waited_for() {
        let controller = MonitorController::new();
        let probe = Arc::new(Probe::default());
        controller.start(body(probe.clone(), Duration::from_millis(60)));
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 1));
        controller.stop();
        controller.start_with_grace(body(probe.clone(), Duration::ZERO), Duration::from_secs(2));
        assert!(wait_until(|| probe.started.load(Ordering::SeqCst) == 2));
        assert_eq!(probe.max_live.load(Ordering::SeqCst), 1, "deux boucles ont tourne en meme temps");
        controller.stop();
        assert!(wait_until(|| probe.live.load(Ordering::SeqCst) == 0));
    }

    #[test]
    fn restarts_after_the_thread_died_on_its_own() {
        let controller = MonitorController::new();
        let probe = Arc::new(Probe::default());
        controller.start(|_token| {});
        assert!(wait_until(|| !controller.is_running()));
        assert_eq!(controller.start(body(probe.clone(), Duration::ZERO)), StartOutcome::Started);
        assert!(wait_until(|| probe.started.load(Ordering::SeqCst) == 1));
        controller.stop();
    }

    #[test]
    fn token_sleep_wakes_up_immediately_on_stop() {
        let token = Arc::new(StopToken::new());
        let sleeper = Arc::clone(&token);
        let handle = thread::spawn(move || {
            let start = Instant::now();
            let stopped = sleeper.sleep(Duration::from_secs(10));
            (stopped, start.elapsed())
        });
        thread::sleep(Duration::from_millis(30));
        token.stop();
        let (stopped, elapsed) = handle.join().unwrap();
        assert!(stopped);
        assert!(elapsed < Duration::from_secs(2));
    }
}
