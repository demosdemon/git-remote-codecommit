use std::num::NonZeroU16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InferredHostname<'a> {
    region: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CliHostname {
    host: String,
    port: Option<NonZeroU16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Hostname<'a> {
    Inferred(InferredHostname<'a>),
    Cli(CliHostname),
}

impl<'a> InferredHostname<'a> {
    pub fn new(region: &'a str) -> Self {
        Self { region }
    }
}

impl core::fmt::Display for InferredHostname<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let &Self { region } = self;
        let partition = if region.starts_with("cn-") {
            "amazonaws.com.cn"
        } else {
            "amazonaws.com"
        };
        write!(f, "git-codecommit.{region}.{partition}")
    }
}

impl core::fmt::Display for CliHostname {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}:{}", self.host, port)
        } else {
            write!(f, "{}", self.host)
        }
    }
}

impl core::fmt::Display for Hostname<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Hostname::Inferred(inferred) => inferred.fmt(f),
            Hostname::Cli(cli) => cli.fmt(f),
        }
    }
}

impl core::str::FromStr for CliHostname {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (host, port) = extract_hostname(value).ok_or("invalid hostname")?;
        let host = host.to_owned();
        Ok(Self { host, port })
    }
}

#[inline]
fn extract_hostname(hostname: &str) -> Option<(&str, Option<NonZeroU16>)> {
    #[inline]
    fn parse_port(mut bytes: std::iter::Copied<std::slice::Iter<'_, u8>>) -> Option<NonZeroU16> {
        #[inline]
        fn add_digit(port: u16, digit: u8) -> Option<u16> {
            let digit = digit.is_ascii_digit().then_some(digit)?;
            let port = port.checked_mul(10)?;
            port.checked_add(u16::from(digit - b'0'))
        }
        bytes.try_fold(0_u16, add_digit).and_then(NonZeroU16::new)
    }

    let mut bytes = hostname.as_bytes().iter().copied();
    let mut host_len = 0;
    let mut label_len = 0;
    let mut last_was_hyphen = false;

    let port = loop {
        match bytes.next() {
            None | Some(b'.' | b':') if label_len == 0 || last_was_hyphen || host_len + 1 > 255 => {
                return None;
            }
            Some(b'-') if label_len == 0 => return None,
            Some(b'-' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')
                if label_len + 1 > 63 || host_len + 1 > 255 =>
            {
                return None;
            }
            Some(b'.') => {
                host_len += 1;
                label_len = 0;
                last_was_hyphen = false;
            }
            Some(b @ (b'-' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z')) => {
                host_len += 1;
                label_len += 1;
                last_was_hyphen = b == b'-';
            }
            Some(b':') => break Some(parse_port(bytes)?),
            Some(_) => return None,
            None => break None,
        }
    };

    // SAFETY: We have validated that `host_len` is a valid character boundary in
    // `hostname`. This removes the need for a bounds check when slicing `hostname`
    // to get the hostname.
    unsafe { core::hint::assert_unchecked(hostname.is_char_boundary(host_len)) };
    Some((&hostname[..host_len], port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cn() {
        let s = InferredHostname::new("cn-north-1").to_string();
        assert_eq!(s, "git-codecommit.cn-north-1.amazonaws.com.cn");
    }

    #[test]
    fn test_iad() {
        let s = InferredHostname::new("us-east-1").to_string();
        assert_eq!(s, "git-codecommit.us-east-1.amazonaws.com");
    }

    // CliHostname::from_str — valid inputs

    #[test]
    fn test_cli_hostname_simple() {
        let h: CliHostname = "localhost".parse().unwrap();
        assert_eq!(h.host, "localhost");
        assert_eq!(h.port, None);
    }

    #[test]
    fn test_cli_hostname_with_port() {
        let h: CliHostname = "localhost:8443".parse().unwrap();
        assert_eq!(h.host, "localhost");
        assert_eq!(h.port, Some(NonZeroU16::new(8443).unwrap()));
    }

    #[test]
    fn test_cli_hostname_fqdn() {
        let h: CliHostname = "git-codecommit.us-east-1.amazonaws.com".parse().unwrap();
        assert_eq!(h.host, "git-codecommit.us-east-1.amazonaws.com");
        assert_eq!(h.port, None);
    }

    #[test]
    fn test_cli_hostname_fqdn_with_port() {
        let h: CliHostname = "example.com:443".parse().unwrap();
        assert_eq!(h.host, "example.com");
        assert_eq!(h.port, Some(NonZeroU16::new(443).unwrap()));
    }

    #[test]
    fn test_cli_hostname_single_char_labels() {
        let h: CliHostname = "a.b.c".parse().unwrap();
        assert_eq!(h.host, "a.b.c");
        assert_eq!(h.port, None);
    }

    #[test]
    fn test_cli_hostname_numeric_label() {
        let h: CliHostname = "123.456".parse().unwrap();
        assert_eq!(h.host, "123.456");
        assert_eq!(h.port, None);
    }

    #[test]
    fn test_cli_hostname_max_port() {
        let h: CliHostname = "host:65535".parse().unwrap();
        assert_eq!(h.host, "host");
        assert_eq!(h.port, Some(NonZeroU16::new(65535).unwrap()));
    }

    #[test]
    fn test_cli_hostname_min_port() {
        let h: CliHostname = "host:1".parse().unwrap();
        assert_eq!(h.host, "host");
        assert_eq!(h.port, Some(NonZeroU16::new(1).unwrap()));
    }

    #[test]
    fn test_cli_hostname_hyphen_middle() {
        let h: CliHostname = "my-host".parse().unwrap();
        assert_eq!(h.host, "my-host");
        assert_eq!(h.port, None);
    }

    // CliHostname::from_str — invalid inputs

    #[test]
    fn test_cli_hostname_empty() {
        assert!("".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_leading_dot() {
        assert!(".example.com".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_trailing_dot() {
        assert!("example.".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_double_dot() {
        assert!("example..com".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_leading_hyphen() {
        assert!("-host".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_trailing_hyphen() {
        assert!("host-".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_label_trailing_hyphen_dot() {
        assert!("host-.com".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_port_zero() {
        assert!("host:0".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_port_overflow() {
        assert!("host:65536".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_port_non_numeric() {
        assert!("host:abc".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_port_empty() {
        assert!("host:".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_invalid_char() {
        assert!("host_name".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_space() {
        assert!("host name".parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_label_too_long() {
        let label = "a".repeat(64);
        assert!(label.parse::<CliHostname>().is_err());
    }

    #[test]
    fn test_cli_hostname_total_too_long() {
        // Build a hostname of 259 chars: 65 labels of 3 chars separated by dots
        // 65*3 + 64 = 259
        let hostname = (0..65).map(|_| "aaa").collect::<Vec<_>>().join(".");
        assert!(hostname.len() > 255);
        assert!(hostname.parse::<CliHostname>().is_err());
    }

    // CliHostname Display

    #[test]
    fn test_cli_hostname_display_with_port() {
        let h: CliHostname = "host:443".parse().unwrap();
        assert_eq!(h.to_string(), "host:443");
    }

    #[test]
    fn test_cli_hostname_display_without_port() {
        let h: CliHostname = "host".parse().unwrap();
        assert_eq!(h.to_string(), "host");
    }

    // Hostname enum Display delegation

    #[test]
    fn test_hostname_display_inferred() {
        let h = Hostname::Inferred(InferredHostname::new("us-east-1"));
        assert_eq!(h.to_string(), "git-codecommit.us-east-1.amazonaws.com");
    }

    #[test]
    fn test_hostname_display_cli() {
        let h = Hostname::Cli("example.com:8080".parse().unwrap());
        assert_eq!(h.to_string(), "example.com:8080");
    }
}
