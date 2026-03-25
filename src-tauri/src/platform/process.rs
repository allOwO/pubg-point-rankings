#[cfg(windows)]
use std::process::Command;

pub fn is_pubg_running() -> bool {
    let process_names = list_process_names();
    process_names
        .iter()
        .any(|process_name| is_pubg_process_name(process_name))
}

#[cfg(windows)]
fn list_process_names() -> Vec<String> {
    let output = match Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_tasklist_csv_names(&stdout)
}

#[cfg(not(windows))]
fn list_process_names() -> Vec<String> {
    parse_tasklist_csv_names("")
}

fn is_pubg_process_name(name: &str) -> bool {
    let normalized = name.trim().to_ascii_lowercase();
    normalized == "tslgame.exe" || normalized == "pubg.exe"
}

fn parse_tasklist_csv_names(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            trimmed
                .split(",")
                .next()
                .map(|name| name.trim().trim_matches('"').to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{is_pubg_process_name, parse_tasklist_csv_names};

    #[test]
    fn recognizes_pubg_process_names_case_insensitively() {
        assert!(is_pubg_process_name("TslGame.exe"));
        assert!(is_pubg_process_name("PUBG.exe"));
        assert!(!is_pubg_process_name("explorer.exe"));
    }

    #[test]
    fn parses_first_csv_column_as_process_name() {
        let output = concat!(
            "\"explorer.exe\",\"1244\",\"Console\",\"1\",\"12,344 K\"\n",
            "\"TslGame.exe\",\"8892\",\"Console\",\"1\",\"1,234,567 K\"\n"
        );

        let names = parse_tasklist_csv_names(output);

        assert_eq!(names, vec!["explorer.exe", "TslGame.exe"]);
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_fails_safe_as_not_running() {
        assert!(!super::is_pubg_running());
    }
}
