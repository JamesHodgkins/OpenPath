//! Parse shortcut text such as `"Ctrl+Shift+Z"` into egui shortcuts.

use egui::{Key, KeyboardShortcut, Modifiers};

/// Parse `"Ctrl+Shift+Z"`, `"Delete"`, `"Alt+F4"` and similar. `Ctrl` maps to
/// egui's `COMMAND` (Ctrl on Windows/Linux, Cmd on macOS). Returns `None` if
/// a part is not recognized.
pub fn parse_shortcut(text: &str) -> Option<KeyboardShortcut> {
    let mut modifiers = Modifiers::NONE;
    let mut key = None;
    for part in text.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "cmd" | "command" => modifiers.command = true,
            "shift" => modifiers.shift = true,
            "alt" => modifiers.alt = true,
            _ => {
                if key.is_some() {
                    return None;
                }
                key = Some(parse_key(part)?);
            }
        }
    }
    Some(KeyboardShortcut::new(modifiers, key?))
}

/// `Key::from_name` is case-sensitive ("Delete", "Z"); accept any casing.
fn parse_key(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();
    let mut capitalized = lower.clone();
    if let Some(first) = capitalized.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    [name.to_owned(), capitalized, lower.to_ascii_uppercase()]
        .iter()
        .find_map(|candidate| Key::from_name(candidate))
}

/// Number of modifier keys in a shortcut. Check shortcuts with more modifiers
/// first so `Ctrl+Shift+Z` is not swallowed by `Ctrl+Z`.
pub fn modifier_count(shortcut: &KeyboardShortcut) -> usize {
    let m = shortcut.modifiers;
    [m.command, m.ctrl, m.shift, m.alt]
        .into_iter()
        .filter(|&on| on)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_and_key() {
        let s = parse_shortcut("Ctrl+Shift+Z").unwrap();
        assert_eq!(s.logical_key, Key::Z);
        assert!(s.modifiers.command && s.modifiers.shift && !s.modifiers.alt);
        assert_eq!(modifier_count(&s), 2);
    }

    #[test]
    fn parses_bare_key_and_is_case_insensitive() {
        let s = parse_shortcut("delete").unwrap();
        assert_eq!(s.logical_key, Key::Delete);
        assert_eq!(s.modifiers, Modifiers::NONE);
        assert!(parse_shortcut("ctrl+n").is_some());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_shortcut("Ctrl+").is_none());
        assert!(parse_shortcut("Ctrl+Banana").is_none());
        assert!(parse_shortcut("Z+Y").is_none());
        assert!(parse_shortcut("").is_none());
    }
}
