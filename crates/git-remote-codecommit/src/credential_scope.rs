use std::time::SystemTime;

use crate::TimestampExt;
use crate::SERVICE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CredentialScope<'a> {
    pub timestamp: SystemTime,
    pub region: &'a str,
}

impl core::fmt::Display for CredentialScope<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { timestamp, region } = self;
        let date = timestamp.sigv4_date();
        write!(f, "{date}/{region}/{SERVICE}/aws4_request")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let s = CredentialScope {
            timestamp: SystemTime::UNIX_EPOCH,
            region: "us-east-1",
        }
        .to_string();

        assert_eq!(s, "19700101/us-east-1/codecommit/aws4_request")
    }
}
