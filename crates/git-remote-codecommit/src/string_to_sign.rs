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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let s = StringToSign {
            timestamp: SystemTime::UNIX_EPOCH,
            credential_scope: CredentialScope {
                timestamp: SystemTime::UNIX_EPOCH,
                region: "us-east-1",
            },
            canonical_request: CanonicalRequest {
                repo: "my-repo",
                hostname: "git-codecommit.us-east-1.amazonaws.com",
            },
        }
        .to_string();

        assert_eq!(
            s,
            "AWS4-HMAC-SHA256\n19700101T000000\n19700101/us-east-1/codecommit/aws4_request\na1d3c427fe57dc90a0031cb03cef21be70874879bb17c5c2ab29dfda0f514c7a"
        );
    }
}
