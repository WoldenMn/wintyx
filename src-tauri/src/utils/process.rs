//! Execution de process externes et manipulation de chaines associee.
//!
//! Decompose (code de prod > 200L, dette V3) en deux responsabilites distinctes :
//! - [`exec`] : lancement de process avec timeout + drainage concurrent des pipes
//! - [`text`] : decoupage d'arguments, parsing pipe-separated, echappement PowerShell
//!
//! La surface publique (`run`, `run_silent`, `CmdResult`, `parse_pipe_fields`,
//! `escape_powershell`) est inchangee : les call-sites `process::xxx` continuent
//! de fonctionner via les re-exports ci-dessous.

use serde::Serialize;

mod decode;
mod exec;
mod system_path;
mod text;

pub use exec::{run, run_silent};
pub use system_path::{resolve_system_program, system_root};
pub use text::{escape_powershell, parse_pipe_fields};

/// Resultat d'une execution de process : statut de succes, sorties standard et
/// d'erreur, code de retour (-1 si timeout ou echec de spawn).
#[derive(Debug, Clone, Serialize)]
pub struct CmdResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
