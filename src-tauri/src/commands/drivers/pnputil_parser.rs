#[derive(Debug, Clone)]
pub(super) struct PnputilDriver {
    pub oem_inf: String,
    pub original_inf: String,
    pub provider: String,
    pub class: String,
    pub version: String,
    pub date: String,
}

/// Parse une ligne CSV en respectant le quoting RFC 4180 : un champ entre
/// guillemets peut contenir des virgules ; `""` represente un guillemet litteral.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => fields.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    fields.push(field);
    fields
}

/// `DriverVersion` vaut « MM/JJ/AAAA version » : renvoie (date ISO, version).
fn split_date_version(raw: &str) -> Option<(String, String)> {
    let (date, version) = raw.trim().split_once(' ')?;
    let version = version.trim();
    if version.is_empty() {
        return None;
    }
    let date = chrono::NaiveDate::parse_from_str(date, "%m/%d/%Y").ok()?;
    Some((date.format("%Y-%m-%d").to_string(), version.to_string()))
}

fn column_index(header: &[String], name: &str) -> Result<usize, String> {
    header
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("colonne {} absente de la sortie pnputil", name))
}

/// Parse la sortie de `pnputil /enum-drivers /format csv`, colonnes lues par nom.
/// Une sortie sans l'en-tete attendu (texte d'aide, format change) est une erreur :
/// elle ne doit jamais produire une liste qui pilote une suppression. Les lignes
/// dont la version ou la date est illisible sont ignorees.
pub(super) fn parse_pnputil_csv(csv: &str) -> Result<Vec<PnputilDriver>, String> {
    let mut lines = csv.lines().map(str::trim).filter(|l| !l.is_empty());
    let Some(header_line) = lines.next() else {
        return Ok(Vec::new());
    };
    let header = parse_csv_line(header_line);
    let i_oem = column_index(&header, "DriverName")?;
    let i_original = column_index(&header, "OriginalName")?;
    let i_provider = column_index(&header, "ProviderName")?;
    let i_class = column_index(&header, "ClassName")?;
    let i_version = column_index(&header, "DriverVersion")?;
    let needed = [i_oem, i_original, i_provider, i_class, i_version]
        .into_iter()
        .max()
        .unwrap_or(0)
        + 1;

    Ok(lines
        .filter_map(|line| {
            let fields = parse_csv_line(line);
            if fields.len() < needed {
                return None;
            }
            let (date, version) = split_date_version(&fields[i_version])?;
            Some(PnputilDriver {
                oem_inf: fields[i_oem].trim().to_string(),
                original_inf: fields[i_original].trim().to_string(),
                provider: fields[i_provider].trim().to_string(),
                class: fields[i_class].trim().to_string(),
                version,
                date,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/enum_drivers.csv");

    #[test]
    fn parse_real_pnputil_output_by_column_name() {
        let parsed = parse_pnputil_csv(FIXTURE).unwrap();
        assert_eq!(parsed.len(), 136);
        let amd = parsed.iter().find(|d| d.oem_inf == "oem53.inf").unwrap();
        assert_eq!(amd.original_inf, "paquet001.inf");
        assert_eq!(amd.provider, "Fournisseur 01");
        assert_eq!(amd.class, "System");
        assert_eq!(amd.version, "1.0.0.12");
        assert_eq!(amd.date, "2025-09-26");
    }

    #[test]
    fn parse_real_pnputil_output_splits_date_and_version() {
        let parsed = parse_pnputil_csv(FIXTURE).unwrap();
        let nv = parsed.iter().find(|d| d.oem_inf == "oem118.inf").unwrap();
        assert_eq!(nv.version, "1.4.6.3");
        assert_eq!(nv.date, "2026-07-23");
        assert!(parsed.iter().all(|d| !d.version.contains(' ') && !d.version.contains('/')));
    }

    #[test]
    fn parse_pnputil_csv_is_independent_of_column_order() {
        let csv = "DriverVersion,ProviderName,ClassName,OriginalName,DriverName\n\
\"03/17/2025 9.8.30.0\",\"Broadcom Inc.\",\"System\",\"vmci.inf\",\"oem47.inf\"\n";
        let parsed = parse_pnputil_csv(csv).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].oem_inf, "oem47.inf");
        assert_eq!(parsed[0].original_inf, "vmci.inf");
        assert_eq!(parsed[0].provider, "Broadcom Inc.");
        assert_eq!(parsed[0].class, "System");
        assert_eq!(parsed[0].version, "9.8.30.0");
        assert_eq!(parsed[0].date, "2025-03-17");
    }

    #[test]
    fn parse_pnputil_csv_rejects_usage_text() {
        let usage = "PNPUTIL [/add-driver <...> | /delete-driver <...> |\n         /export-driver <...> | /enum-drivers [<...>] |\n";
        assert!(parse_pnputil_csv(usage).is_err());
    }

    #[test]
    fn parse_pnputil_csv_rejects_missing_required_column() {
        let csv = "DriverName,OriginalName,ProviderName,ClassName\n\"oem1.inf\",\"a.inf\",\"P\",\"Net\"\n";
        assert!(parse_pnputil_csv(csv).is_err());
    }

    #[test]
    fn parse_pnputil_csv_empty_output_is_empty_list() {
        assert!(parse_pnputil_csv("").unwrap().is_empty());
    }

    #[test]
    fn parse_pnputil_csv_skips_rows_without_parsable_version() {
        let csv = "DriverName,OriginalName,ProviderName,ClassName,DriverVersion\n\
\"oem1.inf\",\"a.inf\",\"P\",\"Net\",\"10.0\"\n\
\"oem2.inf\",\"b.inf\",\"P\",\"Net\",\"13/45/2024 1.0.0.0\"\n\
\"oem3.inf\",\"c.inf\",\"P\",\"Net\",\"01/10/2025 1.0.0.0\"\n\
short\n";
        let parsed = parse_pnputil_csv(csv).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].oem_inf, "oem3.inf");
    }

    #[test]
    fn parse_pnputil_csv_tolerates_crlf_and_blank_lines() {
        let csv = "DriverName,OriginalName,ProviderName,ClassName,DriverVersion\r\n\
\"oem1.inf\",\"a.inf\",\"P\",\"Net\",\"01/10/2025 1.0.0.0\"\r\n\r\n";
        assert_eq!(parse_pnputil_csv(csv).unwrap().len(), 1);
    }

    #[test]
    fn parse_csv_line_handles_quotes_and_escapes() {
        assert_eq!(parse_csv_line("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(parse_csv_line("a,\"b,c\",d"), vec!["a", "b,c", "d"]);
        assert_eq!(parse_csv_line("\"x\"\"y\""), vec!["x\"y"]);
    }
}
