use std::panic::AssertUnwindSafe;
use std::sync::mpsc;
use std::time::Duration;

use wmi::{COMLibrary, Variant, WMIConnection};

use crate::utils::constants;
use crate::utils::errors::CommandError;

/// Execute une closure sur un thread dedie qui initialise COM lui-meme, avec un
/// delai maximum.
///
/// Pourquoi un thread dedie (fix v2.11.0) : WebView2 initialise COM en apartment
/// STA sur le thread principal ; `COMLibrary::new()` demande MTA et recevrait
/// `RPC_E_CHANGED_MODE`. Un thread frais n'a aucun COM preexistant, donc l'init
/// MTA reussit. Depuis le run 7 (v2.18), les commandes WMI tournent deja hors du
/// thread principal (`utils::blocking`) : ce thread dedie reste necessaire car
/// les threads du pool ne sont pas garantis vierges de COM, et il est le seul
/// moyen d'abandonner une requete WMI bloquee (Winmgmt fige) : au-dela du delai
/// l'appelant recoit `CommandError::Timeout`, le thread fautif est laisse a
/// lui-meme et l'interface n'attend plus.
pub fn run_on_com_thread<R, F>(f: F) -> Result<R, CommandError>
where
    F: FnOnce() -> Result<R, CommandError> + Send + 'static,
    R: Send + 'static,
{
    run_on_com_thread_within(f, Duration::from_millis(constants::TIMEOUT_WMI))
}

fn run_on_com_thread_within<R, F>(f: F, timeout: Duration) -> Result<R, CommandError>
where
    F: FnOnce() -> Result<R, CommandError> + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("wintyx-wmi".into())
        .spawn(move || {
            let outcome = std::panic::catch_unwind(AssertUnwindSafe(f));
            let _ = tx.send(outcome);
        })
        .map_err(|e| CommandError::WmiUnavailable(format!("thread WMI: {}", e)))?;

    match rx.recv_timeout(timeout) {
        Ok(Ok(result)) => result,
        Ok(Err(payload)) => {
            let detail = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panic non textuel".to_string());
            crate::utils::logger::error("wmi", &format!("thread WMI: panic ({})", detail));
            Err(CommandError::WmiUnavailable(format!("thread WMI: panic ({})", detail)))
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            crate::utils::logger::error(
                "wmi",
                &format!("requête WMI sans réponse après {} ms, abandon", timeout.as_millis()),
            );
            Err(CommandError::Timeout {
                command: "WMI".into(),
                timeout_ms: timeout.as_millis() as u64,
            })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(CommandError::WmiUnavailable("thread WMI terminé sans résultat".into()))
        }
    }
}

/// Initialise COM + WMIConnection sur un thread dedie et passe la connexion a la closure.
///
/// Usage :
/// ```ignore
/// let drivers = with_wmi_conn(|wmi| { let results = wmi.raw_query(...)?; Ok(results) })?;
/// ```
///
/// L'init est centralisee une seule fois par appel (gain ~200-400ms sur les commandes
/// qui chainent plusieurs requetes WMI, cf. `get_memory_summary`, `get_driver_summary`)
/// et isolee sur un thread frais pour eviter le conflit d'apartment COM (cf.
/// [`run_on_com_thread`]).
pub fn with_wmi_conn<R, F>(f: F) -> Result<R, CommandError>
where
    F: Fn(&WMIConnection) -> Result<R, CommandError> + Send + 'static,
    R: Send + 'static,
{
    run_on_com_thread(move || {
        retry_transient(WMI_RETRIES, WMI_RETRY_DELAY, || {
            let com = COMLibrary::new().map_err(|e| {
                crate::utils::logger::error("wmi", &format!("COMLibrary init echec: {}", e));
                CommandError::WmiUnavailable(e.to_string())
            })?;
            let wmi = WMIConnection::new(com).map_err(|e| {
                crate::utils::logger::error("wmi", &format!("WMIConnection echec (Winmgmt indisponible ?): {}", e));
                CommandError::WmiUnavailable(e.to_string())
            })?;
            f(&wmi)
        })
    })
}

const WMI_RETRIES: u32 = 2;
const WMI_RETRY_DELAY: Duration = Duration::from_millis(400);

/// Codes WMI documentes comme lies a la memoire du fournisseur, donc passagers quand
/// plusieurs requetes partagent le meme hote wmiprvse (quota par hote) :
/// WBEM_E_OUT_OF_MEMORY 0x80041006 ("Not enough memory for the operation") et
/// WBEM_E_QUOTA_VIOLATION 0x8004106C ("WMI is taking up too much memory"). Sources :
/// learn.microsoft.com/windows/win32/wmisdk/wmi-error-constants et
/// techcommunity.microsoft.com "Memory and Handle Quotas in the WMI Provider Service".
const TRANSIENT_WMI_CODES: [&str; 2] = ["80041006", "8004106c"];

pub(crate) fn is_transient_wmi_error(err: &CommandError) -> bool {
    match err {
        CommandError::WmiUnavailable(detail) => {
            let lower = detail.to_ascii_lowercase();
            TRANSIENT_WMI_CODES.iter().any(|code| lower.contains(code))
        }
        _ => false,
    }
}

/// Rejoue `op` jusqu'a `retries` fois apres un echec WMI passager, avec `delay` entre
/// les essais. Toute autre erreur, ou l'echec du dernier essai, est rendue telle quelle.
pub(crate) fn retry_transient<R>(
    retries: u32,
    delay: Duration,
    mut op: impl FnMut() -> Result<R, CommandError>,
) -> Result<R, CommandError> {
    let mut attempt = 0;
    loop {
        match op() {
            Err(e) if attempt < retries && is_transient_wmi_error(&e) => {
                attempt += 1;
                crate::utils::logger::warn(
                    "wmi",
                    &format!("échec WMI passager ({}), nouvel essai {}/{}", e.friendly_message(), attempt, retries),
                );
                std::thread::sleep(delay);
            }
            other => return other,
        }
    }
}

fn as_integer(v: &Variant) -> Option<i128> {
    match v {
        Variant::I1(n) => Some(i128::from(*n)),
        Variant::I2(n) => Some(i128::from(*n)),
        Variant::I4(n) => Some(i128::from(*n)),
        Variant::I8(n) => Some(i128::from(*n)),
        Variant::UI1(n) => Some(i128::from(*n)),
        Variant::UI2(n) => Some(i128::from(*n)),
        Variant::UI4(n) => Some(i128::from(*n)),
        Variant::UI8(n) => Some(i128::from(*n)),
        _ => None,
    }
}

fn narrow<T: TryFrom<i128> + Default>(helper: &str, v: Option<&Variant>) -> T {
    let Some(v) = v else { return T::default() };
    let parsed = match v {
        Variant::Null | Variant::Empty => return T::default(),
        Variant::String(s) => s.trim().parse::<i128>().ok(),
        other => as_integer(other),
    };
    match parsed.and_then(|n| T::try_from(n).ok()) {
        Some(n) => n,
        None => {
            crate::utils::logger::warn(
                "wmi",
                &format!("{}: valeur WMI inexploitable ({}), repli 0", helper, describe(v)),
            );
            T::default()
        }
    }
}

fn describe(v: &Variant) -> String {
    match v {
        Variant::String(s) => format!("texte '{}'", s),
        other => as_integer(other)
            .map(|n| format!("entier {}", n))
            .unwrap_or_else(|| "type non numérique".to_string()),
    }
}

pub fn v_str(v: Option<&Variant>) -> String {
    match v {
        Some(Variant::String(s)) => s.clone(),
        Some(Variant::Bool(b)) => b.to_string(),
        Some(Variant::Null) | None => String::new(),
        Some(other) => match as_integer(other) {
            Some(n) => n.to_string(),
            None => format!("{:?}", other),
        },
    }
}

pub fn v_f64(v: Option<&Variant>) -> f64 {
    match v {
        Some(Variant::R8(n)) => *n,
        Some(Variant::R4(n)) => f64::from(*n),
        Some(Variant::String(s)) => s.parse().unwrap_or_else(|_| {
            crate::utils::logger::warn("wmi", &format!("v_f64: valeur WMI non numerique '{}', repli 0.0", s));
            0.0
        }),
        Some(other) => as_integer(other).map(|n| n as f64).unwrap_or(0.0),
        None => 0.0,
    }
}

pub fn v_i32(v: Option<&Variant>) -> i32 {
    narrow("v_i32", v)
}

pub fn v_u32(v: Option<&Variant>) -> u32 {
    narrow("v_u32", v)
}

pub fn kb_to_gb_rounded(kb: f64) -> f64 {
    (kb / super::constants::KB_TO_GB * 100.0).round() / 100.0
}

pub fn v_bool(v: Option<&Variant>) -> bool {
    match v {
        Some(Variant::Bool(b)) => *b,
        Some(Variant::String(s)) => s.eq_ignore_ascii_case("true"),
        Some(other) => as_integer(other).is_some_and(|n| n != 0),
        None => false,
    }
}

#[cfg(test)]
mod tests;
