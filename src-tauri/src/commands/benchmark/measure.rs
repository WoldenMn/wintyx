use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::utils::constants;

/// Duree minimale cumulee d'une mesure CPU ou memoire : en dessous, la resolution de
/// l'horloge (100 ns sous Windows) domine et le debit calcule n'a plus de sens.
const MIN_MEASURE: Duration = Duration::from_millis(150);
/// Duree sous laquelle un debit est refuse (mesure non fiable).
const MIN_ELAPSED: Duration = Duration::from_micros(500);

pub(super) const CPU_MOPS_RANGE: (f64, f64) = (0.5, 20_000.0);
pub(super) const MEMORY_MBS_RANGE: (f64, f64) = (50.0, 500_000.0);
pub(super) const DISK_MBS_RANGE: (f64, f64) = (1.0, 100_000.0);

/// Debit en Mo/s, borne au plafond. `None` si la duree est trop courte pour etre crue.
pub(super) fn throughput_mbs(bytes: u64, elapsed: Duration, range: (f64, f64)) -> Option<f64> {
    if elapsed < MIN_ELAPSED {
        return None;
    }
    let mbs = bytes as f64 / elapsed.as_secs_f64() / constants::MB;
    Some(mbs.clamp(range.0, range.1))
}

pub(super) fn clamp_score(value: f64, range: (f64, f64)) -> f64 {
    if value.is_finite() {
        value.clamp(range.0, range.1)
    } else {
        range.0
    }
}

pub(super) fn is_plausible(value: f64, range: (f64, f64)) -> bool {
    value.is_finite() && value >= range.0 && value <= range.1
}

/// Millions d'iterations par seconde d'une chaine de melange entiere dont chaque etape
/// depend de la precedente (non vectorisable, non eliminable : `black_box` en entree et
/// en sortie). Les iterations sont repetees jusqu'a MIN_MEASURE de calcul cumule.
pub(super) fn bench_cpu() -> f64 {
    const BATCH: u64 = 2_000_000;
    let mut state = black_box(0x9E37_79B9_7F4A_7C15u64);
    let mut iterations = 0u64;
    let start = Instant::now();
    while start.elapsed() < MIN_MEASURE {
        for _ in 0..BATCH {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        }
        iterations += BATCH;
    }
    black_box(state);
    let elapsed = start.elapsed().as_secs_f64();
    clamp_score(iterations as f64 / elapsed / 1_000_000.0, CPU_MOPS_RANGE)
}

/// Debit de copie memoire en Mo/s : 10 Mo copies en boucle jusqu'a MIN_MEASURE cumule,
/// source et destination passees par `black_box` pour que la copie reste reellement executee.
pub(super) fn bench_memory() -> f64 {
    const SIZE: usize = 10 * 1024 * 1024;
    let src = vec![42u8; SIZE];
    let mut dst = vec![0u8; SIZE];
    let mut copied = 0u64;
    let start = Instant::now();
    while start.elapsed() < MIN_MEASURE {
        black_box(&mut dst).copy_from_slice(black_box(&src));
        copied += SIZE as u64;
    }
    black_box(&dst);
    throughput_mbs(copied, start.elapsed(), MEMORY_MBS_RANGE).unwrap_or(MEMORY_MBS_RANGE.0)
}
