//! Manipulation de chaines pour l'execution de process : decoupage d'arguments
//! (respect des guillemets), parsing de sortie pipe-separated, echappement
//! PowerShell.

/// Decoupe une chaine d'arguments en respectant les guillemets doubles.
///
/// Regles :
/// - separe sur les espaces hors guillemets ;
/// - un guillemet double `"` ouvre/ferme une section citee et n'est **pas conserve**
///   dans le token (les espaces internes sont preserves) ;
/// - `\"` est un guillemet **litteral** insere dans l'argument (permet d'imbriquer
///   des chaines PowerShell : `-Command "... \"$x\" ..."`) ;
/// - un guillemet non ferme termine le dernier token sans panic.
///
/// Indispensable pour PowerShell : `-Command "<script>"` doit transmettre `<script>`
/// comme un seul argument **sans** les guillemets externes. Sinon PowerShell recoit
/// une chaine litterale (`"..."`) et l'**affiche** au lieu de l'executer (bug runtime
/// v2.11.0 : IP publique, adaptateurs reseau, taches planifiees remontaient vides).
/// `std::process::Command::args` se charge ensuite de re-quoter proprement chaque
/// argument pour la ligne de commande Windows.
pub(super) fn split_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_token = false;
    let mut chars = args.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'"') => {
                chars.next();
                current.push('"');
                has_token = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                has_token = true;
            }
            ' ' if !in_quotes => {
                if has_token {
                    result.push(std::mem::take(&mut current));
                    has_token = false;
                }
            }
            other => {
                current.push(other);
                has_token = true;
            }
        }
    }

    if has_token {
        result.push(current);
    }

    result
}

pub fn parse_pipe_fields(line: &str, expected: usize) -> Option<Vec<&str>> {
    let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
    if parts.len() >= expected {
        Some(parts)
    } else {
        None
    }
}

/// Echappe une chaine pour insertion dans une chaine PowerShell entre **single-quotes**.
///
/// Reservee a ce contexte : entre guillemets doubles, `$` et le backtick sont
/// interpretes et cette fonction ne les neutralise pas. En quotes simples ils sont
/// litteraux, donc aucun echappement n'est ajoute (un backtick y corromprait le texte).
///
/// - `'` et les guillemets typographiques U+2018 a U+201B sont doubles : PowerShell
///   les traite tous comme des delimiteurs de chaine, le doublement les rend litteraux ;
/// - les caracteres de controle et les separateurs de ligne U+2028/U+2029 sont supprimes.
///
/// Usage : `format!("-Command \"... '{}' ...\"", escape_powershell(user_input))`.
pub fn escape_powershell(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{2028}' | '\u{2029}'))
        .flat_map(|c| match c {
            '\'' | '\u{2018}'..='\u{201B}' => vec![c, c],
            other => vec![other],
        })
        .collect()
}

#[cfg(test)]
mod tests;
