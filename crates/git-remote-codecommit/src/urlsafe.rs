pub struct UrlSafeQuote<'a>(pub &'a str);

impl core::fmt::Display for UrlSafeQuote<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut cur = 0;

        for (idx, c) in self.0.char_indices() {
            if is_urlsafe(c) {
                continue;
            }

            let slice = &self.0[cur..idx];
            f.write_str(slice)?;

            for b in c.encode_utf8(&mut [0; 4]).as_bytes() {
                write!(f, "%{b:02X}")?;
            }

            cur = idx + c.len_utf8();
        }

        let slice = &self.0[cur..];
        f.write_str(slice)
    }
}

const fn is_urlsafe(c: char) -> bool {
    // "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-~";
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_safe() {
        const SAFE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-~";
        let s = UrlSafeQuote(SAFE).to_string();
        assert_eq!(s, SAFE);
    }

    #[test]
    fn test_with_unsafe() {
        let s = UrlSafeQuote("abc/123").to_string();
        assert_eq!(s, "abc%2F123");
    }

    #[test]
    fn test_with_emoji() {
        let s = UrlSafeQuote("abc/123/🦀").to_string();
        assert_eq!(s, "abc%2F123%2F%F0%9F%A6%80");
    }
}
