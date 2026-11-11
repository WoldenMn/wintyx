//! Decodage de la sortie des process : UTF-8 strict d'abord (PowerShell force en
//! UTF-8 par `exec`), sinon page de codes OEM de la console (sc, reg, netsh...).

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub(super) fn decode_output(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => decode_oem(bytes),
    }
}

#[cfg(windows)]
fn decode_oem(bytes: &[u8]) -> String {
    decode_codepage(bytes, oem_codepage())
}

#[cfg(not(windows))]
fn decode_oem(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetOEMCP() -> u32;
    fn MultiByteToWideChar(
        code_page: u32,
        flags: u32,
        src: *const u8,
        src_len: i32,
        dst: *mut u16,
        dst_len: i32,
    ) -> i32;
}

#[cfg(windows)]
fn oem_codepage() -> u32 {
    // SAFETY : GetOEMCP ne prend aucun argument et ne touche aucune memoire.
    unsafe { GetOEMCP() }
}

#[cfg(windows)]
pub(super) fn decode_codepage(bytes: &[u8], code_page: u32) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let Ok(len) = i32::try_from(bytes.len()) else {
        return String::from_utf8_lossy(bytes).into_owned();
    };
    // SAFETY : premier appel sans tampon de sortie pour obtenir la taille ; `bytes`
    // est valide sur `len` octets pendant tout l'appel.
    let wide_len =
        unsafe { MultiByteToWideChar(code_page, 0, bytes.as_ptr(), len, std::ptr::null_mut(), 0) };
    if wide_len <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut wide = vec![0u16; wide_len as usize];
    // SAFETY : `wide` fait exactement `wide_len` elements, taille demandee au premier appel.
    let written = unsafe {
        MultiByteToWideChar(code_page, 0, bytes.as_ptr(), len, wide.as_mut_ptr(), wide_len)
    };
    if written <= 0 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    String::from_utf16_lossy(&wide[..written as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_is_kept_and_bom_stripped() {
        assert_eq!(decode_output("a \u{201C}b\u{201D}".as_bytes()), "a \u{201C}b\u{201D}");
        assert_eq!(decode_output(b"\xEF\xBB\xBF{\"a\":1}"), "{\"a\":1}");
    }

    #[test]
    fn empty_input_gives_empty_string() {
        assert_eq!(decode_output(b""), "");
    }

    #[cfg(windows)]
    #[test]
    fn cp850_bytes_decode_to_accents() {
        assert_eq!(decode_codepage(b"Invit\x82", 850), "Invit\u{E9}");
        assert_eq!(decode_codepage(b"Fran\x87ais \x85", 850), "Fran\u{E7}ais \u{E0}");
    }

    #[cfg(windows)]
    #[test]
    fn invalid_utf8_falls_back_to_a_non_lossy_oem_decode() {
        let out = decode_output(b"Invit\x82");
        assert!(!out.contains('\u{FFFD}'), "{:?}", out);
    }
}
