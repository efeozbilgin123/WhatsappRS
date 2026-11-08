//! Sayfa başlığından okunmamış mesaj sayısını çıkarır.

/// `"(3) WhatsApp"` → 3, `"(99+) WhatsApp"` → 99; başka her biçim → 0.
pub fn parse(title: &str) -> u32 {
    let Some(rest) = title.trim_start().strip_prefix('(') else { return 0 };
    let Some(end) = rest.find(')') else { return 0 };
    let inner = &rest[..end];
    let digits = inner.strip_suffix('+').unwrap_or(inner);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return 0;
    }
    digits.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn parse_count() { assert_eq!(parse("(3) WhatsApp"), 3); }
    #[test] fn parse_no_count() { assert_eq!(parse("WhatsApp"), 0); }
    #[test] fn parse_plus() { assert_eq!(parse("(99+) WhatsApp"), 99); }
    #[test] fn parse_garbage() {
        for t in ["", "(", "()", "(abc) WhatsApp", "WhatsApp (3)", "(-1) WhatsApp"] { assert_eq!(parse(t), 0, "{t}"); }
    }
    #[test] fn parse_overflow() { assert_eq!(parse("(99999999999) WhatsApp"), 0); }
    #[test] fn parse_back_to_zero() { assert_eq!(parse("(2) WhatsApp"), 2); assert_eq!(parse("WhatsApp"), 0); }
    #[test] fn parse_leading_space() { assert_eq!(parse("  (4) WhatsApp"), 4); }
}
