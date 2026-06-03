use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

/// Parse an Electron-style accelerator string into a Tauri `Shortcut`.
///
/// # Format
/// Tokens are joined by `+`. The last token is the key; all preceding tokens
/// are modifiers. Examples: `"CmdOrCtrl+T"`, `"Alt+Shift+F"`, `"Ctrl+Shift+5"`.
///
/// # Modifier tokens
/// | Token | Mapped to |
/// |---|---|
/// | `Ctrl` | `Modifiers::CONTROL` |
/// | `CmdOrCtrl` | `Modifiers::CONTROL` (Windows/Linux), `Modifiers::SUPER` (macOS) |
/// | `Alt` | `Modifiers::ALT` |
/// | `Shift` | `Modifiers::SHIFT` |
/// | `Meta` / `Super` / `Cmd` | `Modifiers::META` |
///
/// # Key tokens
/// Single alpha characters (`A`–`Z`, case-insensitive) → `Code::KeyA`…`Code::KeyZ`
/// Single digits (`0`–`9`) → `Code::Digit0`…`Code::Digit9`
///
/// # Errors
/// Returns `Err(String)` for empty input, missing modifiers, unknown modifiers,
/// unknown key codes, or input with no key token.
pub fn parse_hotkey(s: &str) -> Result<Shortcut, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("快捷键为空。".to_string());
    }

    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() {
        return Err("快捷键为空。".to_string());
    }

    // All tokens except the last are modifiers; the last is the key.
    let (modifier_tokens, key_tokens) = parts.split_at(parts.len() - 1);
    let key_token = key_tokens[0].trim();

    if modifier_tokens.is_empty() {
        return Err("快捷键至少需要包含一个修饰键。".to_string());
    }

    // Build modifier bitmask
    let mut modifiers = Modifiers::empty();
    for token in modifier_tokens {
        let token = token.trim();
        match token {
            "Ctrl" => modifiers |= Modifiers::CONTROL,
            "CmdOrCtrl" => {
                #[cfg(target_os = "macos")]
                {
                    modifiers |= Modifiers::SUPER;
                }
                #[cfg(not(target_os = "macos"))]
                {
                    modifiers |= Modifiers::CONTROL;
                }
            }
            "Alt" => modifiers |= Modifiers::ALT,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Meta" | "Super" | "Cmd" => modifiers |= Modifiers::SUPER,
            other => return Err(format!("未知修饰键：'{}'", other)),
        }
    }

    // Map key token to Code
    let code = parse_key_code(key_token)?;

    Ok(Shortcut::new(Some(modifiers), code))
}

fn parse_key_code(token: &str) -> Result<Code, String> {
    // Single character: alpha or digit
    let mut chars = token.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => match c.to_ascii_uppercase() {
            'A' => Ok(Code::KeyA),
            'B' => Ok(Code::KeyB),
            'C' => Ok(Code::KeyC),
            'D' => Ok(Code::KeyD),
            'E' => Ok(Code::KeyE),
            'F' => Ok(Code::KeyF),
            'G' => Ok(Code::KeyG),
            'H' => Ok(Code::KeyH),
            'I' => Ok(Code::KeyI),
            'J' => Ok(Code::KeyJ),
            'K' => Ok(Code::KeyK),
            'L' => Ok(Code::KeyL),
            'M' => Ok(Code::KeyM),
            'N' => Ok(Code::KeyN),
            'O' => Ok(Code::KeyO),
            'P' => Ok(Code::KeyP),
            'Q' => Ok(Code::KeyQ),
            'R' => Ok(Code::KeyR),
            'S' => Ok(Code::KeyS),
            'T' => Ok(Code::KeyT),
            'U' => Ok(Code::KeyU),
            'V' => Ok(Code::KeyV),
            'W' => Ok(Code::KeyW),
            'X' => Ok(Code::KeyX),
            'Y' => Ok(Code::KeyY),
            'Z' => Ok(Code::KeyZ),
            '0' => Ok(Code::Digit0),
            '1' => Ok(Code::Digit1),
            '2' => Ok(Code::Digit2),
            '3' => Ok(Code::Digit3),
            '4' => Ok(Code::Digit4),
            '5' => Ok(Code::Digit5),
            '6' => Ok(Code::Digit6),
            '7' => Ok(Code::Digit7),
            '8' => Ok(Code::Digit8),
            '9' => Ok(Code::Digit9),
            other => Err(format!("不支持的按键：'{}'", other)),
        },
        _ => Err(format!("不支持的按键片段：'{}'", token)),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_t_parses() {
        let s = parse_hotkey("CmdOrCtrl+T").expect("should parse");
        #[cfg(target_os = "macos")]
        assert_eq!(s.mods, Modifiers::SUPER);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(s.mods, Modifiers::CONTROL);
        assert_eq!(s.key, Code::KeyT);
    }

    #[test]
    fn alt_shift_f_parses() {
        let s = parse_hotkey("Alt+Shift+F").expect("should parse");
        let expected = Modifiers::ALT | Modifiers::SHIFT;
        assert_eq!(s.mods, expected);
        assert_eq!(s.key, Code::KeyF);
    }

    #[test]
    fn lowercase_key_parses() {
        let s = parse_hotkey("Ctrl+t").expect("should parse");
        assert_eq!(s.key, Code::KeyT);
    }

    #[test]
    fn digit_key_parses() {
        let s = parse_hotkey("Alt+5").expect("should parse");
        assert_eq!(s.key, Code::Digit5);
    }

    #[test]
    fn empty_string_errors() {
        assert!(parse_hotkey("").is_err());
    }

    #[test]
    fn whitespace_only_errors() {
        assert!(parse_hotkey("   ").is_err());
    }

    #[test]
    fn unknown_modifier_errors() {
        assert!(parse_hotkey("Win+T").is_err());
    }

    #[test]
    fn single_key_errors() {
        assert!(parse_hotkey("T").is_err());
    }

    #[test]
    fn unsupported_key_errors() {
        // Multi-char tokens that aren't handled
        assert!(parse_hotkey("Ctrl+F12").is_err());
    }

    #[test]
    fn meta_modifier_parses() {
        let s = parse_hotkey("Meta+Space").unwrap_or_else(|_| {
            // Space is not handled — just verify Meta parsing itself doesn't panic
            parse_hotkey("Meta+M").expect("Meta+M should parse")
        });
        assert_eq!(s.mods, Modifiers::SUPER);
    }
}
