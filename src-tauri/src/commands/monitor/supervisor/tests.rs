use super::*;
use crate::commands::monitor::worker::SampleError;
use std::sync::Arc;

struct Scripted {
    script: std::collections::VecDeque<Result<Sample, SampleError>>,
    token: Arc<StopToken>,
    stop_on_timeout: bool,
}

impl Sampler for Scripted {
    fn sample(&mut self, _timeout: Duration) -> Result<Sample, SampleError> {
        match self.script.pop_front() {
            Some(Err(SampleError::TimedOut)) if self.stop_on_timeout => {
                self.token.stop();
                Err(SampleError::TimedOut)
            }
            Some(outcome) => outcome,
            None => {
                self.token.stop();
                Ok(Sample::default())
            }
        }
    }
}

fn scripted(token: &Arc<StopToken>, script: Vec<Result<Sample, SampleError>>, stop_on_timeout: bool) -> Scripted {
    Scripted {
        script: script.into(),
        token: Arc::clone(token),
        stop_on_timeout,
    }
}

fn ok() -> Result<Sample, SampleError> {
    Ok(Sample::default())
}

#[test]
fn after_a_blocked_query_a_fresh_worker_takes_over_without_mixing_old_counters() {
    let token = Arc::new(StopToken::new());
    let mut workers = std::collections::VecDeque::from([
        scripted(&token, vec![ok(), ok(), Err(SampleError::TimedOut)], false),
        scripted(&token, vec![ok(), ok()], false),
    ]);
    let mut spawned = 0;
    let mut emitted = 0;
    supervise(
        &token,
        || {
            spawned += 1;
            workers.pop_front().ok_or(SampleError::Unavailable("plus de worker".into()))
        },
        Duration::from_secs(1),
        Duration::ZERO,
        || Duration::ZERO,
        |_| emitted += 1,
    );
    assert_eq!(spawned, 2, "un worker neuf aurait du etre ouvert apres le blocage");
    assert_eq!(emitted, 3, "le premier echantillon du nouveau worker ne doit pas etre compare a l'ancien");
}

#[test]
fn a_failed_first_connection_ends_the_loop_without_emitting() {
    let token = StopToken::new();
    let mut spawned = 0;
    let mut emitted = 0;
    supervise(
        &token,
        || -> Result<Scripted, SampleError> {
            spawned += 1;
            Err(SampleError::Unavailable("Winmgmt".into()))
        },
        Duration::from_secs(1),
        Duration::ZERO,
        || Duration::ZERO,
        |_| emitted += 1,
    );
    assert_eq!((spawned, emitted), (1, 0));
}

#[test]
fn a_dead_sampler_thread_ends_the_loop_so_that_a_later_start_can_run_again() {
    let token = Arc::new(StopToken::new());
    let mut worker = Some(scripted(
        &token,
        vec![ok(), ok(), Err(SampleError::Unavailable("thread termine".into())), ok(), ok()],
        false,
    ));
    let mut spawned = 0;
    let mut emitted = 0;
    supervise(
        &token,
        || {
            spawned += 1;
            worker.take().ok_or(SampleError::Unavailable("deja pris".into()))
        },
        Duration::from_secs(1),
        Duration::ZERO,
        || Duration::ZERO,
        |_| emitted += 1,
    );
    assert_eq!((spawned, emitted), (1, 1));
    assert!(!token.is_stopped());
}

#[test]
fn stopping_during_the_restart_backoff_does_not_reopen_a_worker() {
    let token = Arc::new(StopToken::new());
    let mut worker = Some(scripted(&token, vec![ok(), Err(SampleError::TimedOut)], true));
    let mut spawned = 0;
    let started = Instant::now();
    supervise(
        &token,
        || {
            spawned += 1;
            worker.take().ok_or(SampleError::Unavailable("deja pris".into()))
        },
        Duration::from_secs(1),
        Duration::from_secs(60),
        || Duration::ZERO,
        |_| {},
    );
    assert_eq!(spawned, 1);
    assert!(started.elapsed() < Duration::from_secs(5), "{:?}", started.elapsed());
}
