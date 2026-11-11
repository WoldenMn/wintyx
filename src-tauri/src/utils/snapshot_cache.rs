use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Résultat récent d'une collecte coûteuse, partagé entre commandes qui lisent la
/// même source. Le verrou est tenu pendant la collecte : un appel concurrent attend
/// le résultat au lieu de relancer la même collecte.
pub struct SnapshotCache<T> {
    slot: Mutex<Option<(Instant, T)>>,
}

impl<T: Clone> SnapshotCache<T> {
    pub const fn new() -> Self {
        Self { slot: Mutex::new(None) }
    }

    pub fn get_or_collect(&self, ttl: Duration, collect: impl FnOnce() -> T) -> T {
        let mut slot = self.slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((at, value)) = slot.as_ref() {
            if at.elapsed() < ttl {
                return value.clone();
            }
        }
        let value = collect();
        *slot = Some((Instant::now(), value.clone()));
        value
    }

    pub fn invalidate(&self) {
        *self.slot.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    const TTL: Duration = Duration::from_secs(60);

    #[test]
    fn second_call_within_ttl_reuses_the_first_collection() {
        let cache = SnapshotCache::new();
        let runs = AtomicU32::new(0);
        let collect = || runs.fetch_add(1, Ordering::SeqCst) + 1;
        assert_eq!(cache.get_or_collect(TTL, collect), 1);
        assert_eq!(cache.get_or_collect(TTL, collect), 1);
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn expired_entry_is_collected_again() {
        let cache = SnapshotCache::new();
        let runs = AtomicU32::new(0);
        let collect = || runs.fetch_add(1, Ordering::SeqCst) + 1;
        cache.get_or_collect(Duration::ZERO, collect);
        assert_eq!(cache.get_or_collect(Duration::ZERO, collect), 2);
    }

    #[test]
    fn invalidate_forces_a_new_collection() {
        let cache = SnapshotCache::new();
        let runs = AtomicU32::new(0);
        let collect = || runs.fetch_add(1, Ordering::SeqCst) + 1;
        cache.get_or_collect(TTL, collect);
        cache.invalidate();
        assert_eq!(cache.get_or_collect(TTL, collect), 2);
    }

    #[test]
    fn concurrent_callers_share_one_collection() {
        let cache = SnapshotCache::new();
        let runs = AtomicU32::new(0);
        std::thread::scope(|s| {
            for _ in 0..4 {
                s.spawn(|| {
                    cache.get_or_collect(TTL, || {
                        std::thread::sleep(Duration::from_millis(50));
                        runs.fetch_add(1, Ordering::SeqCst)
                    })
                });
            }
        });
        assert_eq!(runs.load(Ordering::SeqCst), 1);
    }
}
