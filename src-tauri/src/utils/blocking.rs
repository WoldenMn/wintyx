//! Execution des commandes bloquantes hors du thread principal.
//!
//! Une commande Tauri synchrone tourne sur le thread principal : tant qu'elle
//! attend PowerShell, WMI ou un parcours de fichiers, la fenetre ne repond plus
//! et les evenements emis restent en file. Les commandes lentes sont donc des
//! `async fn` qui deleguent leur corps synchrone a `spawn_blocking`.
//!
//! `#[tauri::command(async)]` est ecarte : il lance le corps sur un worker Tokio,
//! et N commandes PowerShell lentes occuperaient les N workers. `spawn_blocking`
//! utilise le pool dedie aux appels bloquants.
//!
//! Le thread principal ne serialisant plus rien, les ecritures systeme prennent
//! `SYSTEM_WRITE_LOCK` (snapshot, restauration, apply/protect/optimize/remove/
//! clean/fix/disable/enable) : un snapshot ne capture jamais un etat a moitie
//! modifie et deux lots ne s'entrecroisent pas. Les lectures restent paralleles.
//! Le verrou n'est pris qu'ici, jamais dans les corps synchrones : un corps qui
//! en appelle un autre (profil -> protect_item) ne peut pas s'auto-bloquer.

use std::sync::{Mutex, MutexGuard};

use crate::utils::errors::CommandError;

static SYSTEM_WRITE_LOCK: Mutex<()> = Mutex::new(());

fn lock_system_writes() -> MutexGuard<'static, ()> {
    SYSTEM_WRITE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn panic_detail(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panic non textuel".to_string())
}

async fn spawn<T, F>(f: F) -> Result<T, tauri::Error>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f).await
}

fn join_error(join: tauri::Error) -> CommandError {
    let detail = match join {
        tauri::Error::JoinError(e) if e.is_panic() => panic_detail(&*e.into_panic()),
        other => other.to_string(),
    };
    crate::utils::logger::error("blocking", &format!("tache de commande interrompue: {}", detail));
    CommandError::Generic(format!("Tâche interrompue : {}", detail))
}

fn rethrow(join: tauri::Error) -> ! {
    match join {
        tauri::Error::JoinError(e) if e.is_panic() => std::panic::resume_unwind(e.into_panic()),
        other => panic!("tâche de commande interrompue: {}", other),
    }
}

/// Lecture : parallele avec toutes les autres commandes.
pub async fn read_result<T, F>(f: F) -> Result<T, CommandError>
where
    F: FnOnce() -> Result<T, CommandError> + Send + 'static,
    T: Send + 'static,
{
    spawn(f).await.map_err(join_error)?
}

pub async fn read_value<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match spawn(f).await {
        Ok(value) => value,
        Err(join) => rethrow(join),
    }
}

/// Ecriture systeme : exclusive avec toute autre ecriture ou snapshot.
pub async fn write_result<T, F>(f: F) -> Result<T, CommandError>
where
    F: FnOnce() -> Result<T, CommandError> + Send + 'static,
    T: Send + 'static,
{
    spawn(move || {
        let _guard = lock_system_writes();
        f()
    })
    .await
    .map_err(join_error)?
}

pub async fn write_value<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let joined = spawn(move || {
        let _guard = lock_system_writes();
        f()
    })
    .await;
    match joined {
        Ok(value) => value,
        Err(join) => rethrow(join),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::time::Duration;

    fn run<F: std::future::Future>(future: F) -> F::Output {
        tauri::async_runtime::block_on(future)
    }

    #[test]
    fn writes_never_overlap() {
        let live = Arc::new(AtomicUsize::new(0));
        let max_live = Arc::new(AtomicUsize::new(0));
        run(async {
            let tasks: Vec<_> = (0..8)
                .map(|_| {
                    let live = Arc::clone(&live);
                    let max_live = Arc::clone(&max_live);
                    tauri::async_runtime::spawn(async move {
                        write_value(move || {
                            let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                            max_live.fetch_max(now, Ordering::SeqCst);
                            std::thread::sleep(Duration::from_millis(15));
                            live.fetch_sub(1, Ordering::SeqCst);
                        })
                        .await
                    })
                })
                .collect();
            for task in tasks {
                task.await.unwrap();
            }
        });
        assert_eq!(max_live.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn reads_run_in_parallel() {
        let barrier = Arc::new(Barrier::new(4));
        run(async {
            let tasks: Vec<_> = (0..4)
                .map(|_| {
                    let barrier = Arc::clone(&barrier);
                    tauri::async_runtime::spawn(async move {
                        read_value(move || {
                            barrier.wait();
                        })
                        .await
                    })
                })
                .collect();
            for task in tasks {
                task.await.unwrap();
            }
        });
    }

    #[test]
    fn a_read_is_not_blocked_by_a_running_write() {
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
        let read_done = run(async {
            let writer = tauri::async_runtime::spawn(async move {
                write_value(move || {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                })
                .await
            });
            started_rx.recv().unwrap();
            let value = read_value(|| 7).await;
            release_tx.send(()).unwrap();
            writer.await.unwrap();
            value
        });
        assert_eq!(read_done, 7);
    }

    #[test]
    fn result_commands_pass_their_error_through() {
        let out: Result<(), CommandError> =
            run(read_result(|| Err(CommandError::NotFound("x".into()))));
        assert!(matches!(out, Err(CommandError::NotFound(_))));
    }

    #[test]
    fn panic_in_result_command_becomes_a_command_error() {
        let out: Result<u32, CommandError> = run(read_result(|| panic!("boom")));
        match out {
            Err(CommandError::Generic(msg)) => assert!(msg.contains("boom"), "{}", msg),
            other => panic!("attendu Generic, recu {:?}", other.is_ok()),
        }
    }

    #[test]
    fn panic_in_write_releases_the_lock() {
        let out: Result<u32, CommandError> = run(write_result(|| panic!("boom")));
        assert!(out.is_err());
        let after: Result<u32, CommandError> = run(write_result(|| Ok(1)));
        assert_eq!(after.unwrap(), 1);
    }

    #[test]
    fn panic_in_value_command_is_resumed_in_the_caller() {
        let outcome = std::panic::catch_unwind(|| run(read_value(|| -> u32 { panic!("boom-value") })));
        let payload = outcome.expect_err("la panique doit remonter");
        assert_eq!(panic_detail(&*payload), "boom-value");
    }
}
