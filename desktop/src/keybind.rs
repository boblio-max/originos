pub fn normalize(key: &str) -> String {
    let new_key = key.to_ascii_lowercase();
    let mut parts: Vec<&str> = new_key.split('+').map(|s| s.trim()).collect();
    parts.sort();
    let normalized = parts.join("+");
    normalized
}

pub struct Keybind {
    pub action: String,
    pub command: String,
}

pub fn parse_line(line: &str) -> Option<Keybind> {
    let t = line.trim();
    if t.is_empty() || t.starts_with('#') {
        return None;
    }
    let (left, right) = t.split_once('=')?;
    let action = left.trim().to_string();
    let command_raw = right.trim().trim_matches('"').trim().to_string();
    if command_raw.is_empty() {
        return None;
    }
    Some(Keybind { action, command: normalize(&command_raw) })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("Ctrl+Alt+T"), "alt+ctrl+t");
        assert_eq!(normalize("Alt+Ctrl+T"), "alt+ctrl+t");
        assert_eq!(normalize("ctrl + alt + t"), "alt+ctrl+t");
    }

    #[test]
    fn test_parse_valid() {
        let kb = parse_line(r#"launch_terminal = "ctrl+alt+t""#).unwrap();
        assert_eq!(kb.action, "launch_terminal");
        assert_eq!(kb.command, "alt+ctrl+t");
    }

    #[test]
    fn test_parse_comment_empty_bad() {
        assert!(parse_line("# launcher = \"super+space\"").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("badline").is_none());
        assert!(parse_line("action = \"\"").is_none());
    }
}