use std::time::{Duration, Instant};

use super::controller::StopToken;
use super::sample::{metrics_between, Sample};
use super::worker::{SampleError, Sampler};
use super::SystemMetrics;

pub(super) fn supervise<W: Sampler>(
    token: &StopToken,
    mut spawn: impl FnMut() -> Result<W, SampleError>,
    sample_timeout: Duration,
    restart_backoff: Duration,
    interval: impl Fn() -> Duration,
    mut emit: impl FnMut(&SystemMetrics),
) {
    let open = |spawn: &mut dyn FnMut() -> Result<W, SampleError>| match spawn() {
        Ok(worker) => Some(worker),
        Err(e) => {
            crate::utils::logger::warn("monitor", &format!("prelevement WMI impossible, monitoring arrete: {:?}", e));
            None
        }
    };
    let Some(mut worker) = open(&mut spawn) else {
        return;
    };

    let mut previous: Option<Sample> = None;
    while !token.is_stopped() {
        let start = Instant::now();
        match worker.sample(sample_timeout) {
            Ok(current) => {
                if let Some(before) = &previous {
                    emit(&metrics_between(before, &current));
                }
                let first = previous.is_none();
                previous = Some(current);
                if first {
                    continue;
                }
            }
            Err(SampleError::TimedOut) => {
                crate::utils::logger::warn(
                    "monitor",
                    &format!(
                        "requete WMI sans reponse apres {} ms, prelevement abandonne puis relance dans {} s",
                        sample_timeout.as_millis(),
                        restart_backoff.as_secs()
                    ),
                );
                if token.sleep(restart_backoff) {
                    break;
                }
                let Some(fresh) = open(&mut spawn) else {
                    return;
                };
                worker = fresh;
                previous = None;
                continue;
            }
            Err(SampleError::Unavailable(detail)) => {
                crate::utils::logger::warn("monitor", &format!("prelevement WMI interrompu, monitoring arrete: {}", detail));
                return;
            }
        }
        let remaining = interval().saturating_sub(start.elapsed());
        if token.sleep(remaining) {
            break;
        }
    }
}

#[cfg(test)]
mod tests;
