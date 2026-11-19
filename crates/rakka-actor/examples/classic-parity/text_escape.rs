//! The oracle's textual escaping, shared by every case family.

/// Every UTF-16 unit outside printable ASCII becomes `\uXXXX` (upper-case hex), like the oracle's `esc`.
pub fn escape_non_ascii_units(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for unit in text.encode_utf16() {
        match u8::try_from(unit) {
            Ok(ascii) if (0x20..=0x7e).contains(&ascii) => escaped.push(char::from(ascii)),
            _ => escaped.push_str(&format!("\\u{unit:04X}")),
        }
    }
    escaped
}
