use super::ScheduledTaskInfo;

/// Indices forts : détournement d'un interpréteur ou d'un outil de téléchargement.
/// Un auteur de confiance ne les excuse pas (le champ Author se forge).
const STRONG_PATTERNS: &[&str] = &[
    "wscript", "cscript", "mshta", "regsvr32", "certutil", "bitsadmin",
    "http://", "https://", "ftp://",
    "-enc", "-encodedcommand", "-noprofile -windowstyle hidden",
    "invoke-expression", "iex ", "downloadstring",
];

/// Indices faibles : courants chez les logiciels tiers. Informatifs, pour un auteur inconnu.
const WEAK_PATTERNS: &[&str] = &[
    "rundll32", "%temp%", "%appdata%", "\\appdata\\", "\\temp\\", "\\users\\public", "\\programdata\\", "c:\\temp\\",
];

// Stockes en minuscules : compares directement a author_lower, sans realloc (perf-rust-03).
const TRUSTED_AUTHORS: &[&str] = &[
    "microsoft", "microsoft corporation", "intel", "nvidia",
    "amd", "realtek", "google", "adobe", "system",
];

pub(super) fn is_trusted_author(author: &str) -> bool {
    if author.is_empty() {
        return false;
    }
    let author_lower = author.to_lowercase();
    TRUSTED_AUTHORS.iter().any(|a| author_lower.contains(a))
}

/// Teste la presence d'un pattern suspect dans `haystack` (deja en minuscules).
///
/// Cas particulier `-enc` (abreviation de `-EncodedCommand`) : un simple
/// `contains` matcherait `-encoding`, parametre legitime tres courant
/// (`Out-File -Encoding utf8`) -> faux positif (rust-02). On exige une frontiere
/// droite non alphabetique pour ce flag. Les autres patterns restent en contains.
fn pattern_matches(haystack: &str, pattern: &str) -> bool {
    if pattern == "-enc" {
        return flag_present(haystack, "-enc");
    }
    haystack.contains(pattern)
}

/// `flag` present comme token : suivi d'un caractere non alphabetique (ou fin de
/// chaine). Distingue `-enc <base64>` de `-encoding`.
fn flag_present(haystack: &str, flag: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(flag) {
        let idx = start + pos;
        match haystack[idx + flag.len()..].chars().next() {
            None => return true,
            Some(c) if !c.is_ascii_alphabetic() => return true,
            _ => {}
        }
        start = idx + flag.len();
    }
    false
}

/// Deux niveaux. `is_suspicious` : indice fort, quel que soit l'auteur. `needs_review` :
/// indice faible, informatif, réservé aux auteurs non reconnus. Les indices forts sont
/// évalués avant les faibles : un PowerShell encodé reste suspect même sans auteur.
pub(crate) fn analyze_suspicious(task: &mut ScheduledTaskInfo) {
    task.is_suspicious = false;
    task.needs_review = false;
    task.suspicious_reason = String::new();

    let check_str = format!("{} {} {}", task.action_path, task.action_arguments, task.path).to_lowercase();

    if let Some(pattern) = STRONG_PATTERNS.iter().find(|p| pattern_matches(&check_str, p)) {
        task.is_suspicious = true;
        task.suspicious_reason = format!("Motif suspect : {}", pattern);
        return;
    }

    if is_trusted_author(&task.author) {
        return;
    }

    if let Some(pattern) = ["powershell", "cmd.exe"].iter().find(|p| check_str.contains(**p)) {
        task.needs_review = true;
        task.suspicious_reason = format!("Auteur non reconnu, lance {}", pattern);
        return;
    }

    if let Some(pattern) = WEAK_PATTERNS.iter().find(|p| check_str.contains(**p)) {
        task.needs_review = true;
        task.suspicious_reason = format!("Auteur non reconnu, motif courant : {}", pattern);
        return;
    }

    if !task.action_path.is_empty() && task.author.trim().is_empty() {
        task.needs_review = true;
        task.suspicious_reason = "Aucun auteur déclaré".to_string();
    }
}

/// Gravité d'une tâche signalée par un indice fort, d'après son motif : commande encodée
/// ou mshta critiques, outils de téléchargement élevés, le reste moyen.
pub(crate) fn suspicious_severity(task: &ScheduledTaskInfo) -> &'static str {
    let reason = task.suspicious_reason.to_lowercase();
    if ["-enc", "-encodedcommand", "mshta"].iter().any(|p| reason.ends_with(p)) {
        "critical"
    } else if ["certutil", "bitsadmin", "invoke-expression", "iex ", "downloadstring"].iter().any(|p| reason.ends_with(p)) {
        "high"
    } else {
        "medium"
    }
}
