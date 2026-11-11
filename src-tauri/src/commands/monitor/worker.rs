use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use wmi::{COMLibrary, WMIConnection};

use super::collect::take_sample;
use super::sample::Sample;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum SampleError {
    TimedOut,
    Unavailable(String),
}

pub(super) trait Sampler {
    fn sample(&mut self, timeout: Duration) -> Result<Sample, SampleError>;
}

pub(super) type TakeSample = Box<dyn FnMut() -> Sample>;

pub(super) struct SampleWorker {
    requests: Sender<()>,
    results: Receiver<Result<Sample, String>>,
}

fn open_wmi() -> Result<TakeSample, String> {
    let com = COMLibrary::new().map_err(|e| format!("init COM impossible: {}", e))?;
    let wmi = WMIConnection::new(com).map_err(|e| format!("connexion WMI impossible: {}", e))?;
    Ok(Box::new(move || take_sample(&wmi)))
}

impl SampleWorker {
    pub(super) fn spawn_wmi() -> Result<Self, SampleError> {
        Self::spawn_with(open_wmi)
    }

    pub(super) fn spawn_with(
        open: impl FnOnce() -> Result<TakeSample, String> + Send + 'static,
    ) -> Result<Self, SampleError> {
        let (request_tx, request_rx) = mpsc::channel::<()>();
        let (result_tx, result_rx) = mpsc::channel();
        thread::Builder::new()
            .name("wintyx-monitor-wmi".into())
            .spawn(move || {
                let mut take = match open() {
                    Ok(take) => take,
                    Err(detail) => {
                        if result_tx.send(Err(detail.clone())).is_err() {
                            crate::utils::logger::warn(
                                "monitor",
                                &format!("prelevement abandonne avant la fin de l'init: {}", detail),
                            );
                        }
                        return;
                    }
                };
                while request_rx.recv().is_ok() {
                    if result_tx.send(Ok(take())).is_err() {
                        break;
                    }
                }
            })
            .map_err(|e| SampleError::Unavailable(format!("thread de prelevement: {}", e)))?;
        Ok(Self {
            requests: request_tx,
            results: result_rx,
        })
    }
}

impl Sampler for SampleWorker {
    fn sample(&mut self, timeout: Duration) -> Result<Sample, SampleError> {
        let gone = || SampleError::Unavailable("thread de prelevement termine".into());
        if self.requests.send(()).is_err() {
            return match self.results.recv_timeout(timeout) {
                Ok(Err(detail)) => Err(SampleError::Unavailable(detail)),
                _ => Err(gone()),
            };
        }
        match self.results.recv_timeout(timeout) {
            Ok(Ok(sample)) => Ok(sample),
            Ok(Err(detail)) => Err(SampleError::Unavailable(detail)),
            Err(RecvTimeoutError::Timeout) => Err(SampleError::TimedOut),
            Err(RecvTimeoutError::Disconnected) => Err(gone()),
        }
    }
}

#[cfg(test)]
mod tests;
