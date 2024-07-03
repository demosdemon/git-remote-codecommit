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
