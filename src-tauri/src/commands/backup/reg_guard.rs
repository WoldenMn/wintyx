use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use crate::utils::errors::CommandError;
use crate::utils::registry::is_same_or_subkey;

use super::registry_export::CRITICAL_KEYS;

const MAX_BACKUP_BYTES: u64 = 16 * 1024 * 1024;
const FILE_SHARE_READ: u32 = 1;
const ACCEPTED_HEADERS: [&str; 2] = ["windows registry editor version 5.00", "regedit4"];

pub(super) struct VerifiedBackup {
    _locked: File,
}

pub(super) fn open_verified(path: &Path) -> Result<VerifiedBackup, CommandError> {
    let mut file = lock_read_shared(path)
        .map_err(|e| CommandError::Generic(format!("Lecture de la sauvegarde impossible : {e}")))?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_BACKUP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| CommandError::Generic(format!("Lecture de la sauvegarde impossible : {e}")))?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err(CommandError::InvalidInput("Sauvegarde trop volumineuse".into()));
    }
    let text = decode(&bytes).map_err(CommandError::InvalidInput)?;
    verify(&text).map_err(CommandError::InvalidInput)?;
    Ok(VerifiedBackup { _locked: file })
}

fn lock_read_shared(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).share_mode(FILE_SHARE_READ).open(path)
}

pub(super) fn decode(bytes: &[u8]) -> Result<String, String> {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => {
            let (pairs, remainder) = rest.as_chunks::<2>();
            if !remainder.is_empty() {
                return Err("Sauvegarde UTF-16 tronquée".into());
            }
            let units: Vec<u16> = pairs.iter().map(|c| u16::from_le_bytes(*c)).collect();
            String::from_utf16(&units).map_err(|_| "Séquence UTF-16 invalide".to_string())
        }
        _ if bytes.is_ascii() && !bytes.contains(&0) => Ok(String::from_utf8_lossy(bytes).into_owned()),
        _ => Err("Encodage non pris en charge : UTF-16 LE avec BOM ou ASCII attendu".into()),
    }
}

pub(super) fn verify(text: &str) -> Result<(), String> {
    if text.contains('\0') {
        return Err("Caractère nul dans la sauvegarde".into());
    }
    let mut lines = text.split(['\r', '\n']).map(str::trim).filter(|l| !l.is_empty());
    let header = lines.next().ok_or_else(|| "Sauvegarde vide".to_string())?;
    if !ACCEPTED_HEADERS.iter().any(|h| header.eq_ignore_ascii_case(h)) {
        return Err(format!("En-tête .reg non reconnu : {header}"));
    }
    for line in lines {
        if line.starts_with('[') {
            check_key_header(line)?;
        } else if deletes_value(line) {
            return Err(format!("Suppression de valeur refusée : {line}"));
        }
    }
    Ok(())
}

fn check_key_header(line: &str) -> Result<(), String> {
    let inner = line
        .strip_prefix('[')
        .and_then(|l| l.strip_suffix(']'))
        .ok_or_else(|| format!("En-tête de clé malformé : {line}"))?
        .trim_start();
    if inner.starts_with('-') {
        return Err(format!("Suppression de clé refusée : {line}"));
    }
    let key = canonical_key(inner).ok_or_else(|| format!("Ruche non prise en charge : {inner}"))?;
    if key.split('\\').any(str::is_empty) {
        return Err(format!("Chemin de clé malformé : {inner}"));
    }
    if CRITICAL_KEYS.iter().any(|root| is_same_or_subkey(&key, root)) {
        Ok(())
    } else {
        Err(format!("Clé hors de celles que Wintyx sauvegarde : {inner}"))
    }
}

fn canonical_key(key: &str) -> Option<String> {
    let (hive, rest) = match key.split_once('\\') {
        Some((hive, rest)) => (hive, Some(rest)),
        None => (key, None),
    };
    let hive = match hive.to_ascii_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" | "HKLM" => "HKLM",
        "HKEY_CURRENT_USER" | "HKCU" => "HKCU",
        _ => return None,
    };
    Some(match rest {
        Some(rest) => format!("{hive}\\{rest}"),
        None => hive.to_string(),
    })
}

fn deletes_value(line: &str) -> bool {
    line.ends_with("=-") || assigned_value(line).is_some_and(|value| value.starts_with('-'))
}

fn assigned_value(line: &str) -> Option<&str> {
    let after_name = match line.strip_prefix('@') {
        Some(rest) => rest,
        None => after_quoted_name(line.strip_prefix('"')?)?,
    };
    after_name
        .trim_start()
        .strip_prefix('=')
        .map(str::trim_start)
}

fn after_quoted_name(body: &str) -> Option<&str> {
    let mut chars = body.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '"' => return Some(&body[i + 1..]),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests;
