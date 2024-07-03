use std::time::SystemTime;

use crate::CanonicalRequest;
use crate::CredentialScope;
use crate::TimestampExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringToSign<'a> {
    pub timestamp: SystemTime,
    pub credential_scope: CredentialScope<'a>,
    pub canonical_request: CanonicalRequest<'a>,
}

impl core::fmt::Display for StringToSign<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            timestamp,
            credential_scope,
            canonical_request,
        } = self;
        let timestamp = timestamp.sigv4_timestamp();
        let canonical_request = canonical_request.sha256();
        write!(
            f,
            "AWS4-HMAC-SHA256\n{timestamp}\n{credential_scope}\n{canonical_request}"
        )
    }
}
