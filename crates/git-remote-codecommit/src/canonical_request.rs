use std::io::Write;

use hmac::digest::FixedOutput;
use sha2::Digest;

use crate::IntoU256Hex;
use crate::URL_PATH_PREFIX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalRequest<'a> {
    pub repo: &'a str,
    pub hostname: &'a str,
}

impl CanonicalRequest<'_> {
    pub fn sha256(&self) -> impl core::fmt::Display + use<'_> {
        let mut hasher = sha2::Sha256::new();
        write!(hasher, "{self}").expect("writing to hasher cannot fail");
        hasher.finalize_fixed().into_u256_hex()
    }
}

impl core::fmt::Display for CanonicalRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { repo, hostname } = self;
        write!(
            f,
            "GIT\n/{URL_PATH_PREFIX}/{repo}\n\nhost:{hostname}\n\nhost\n"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let s = CanonicalRequest {
            repo: "my-repo",
            hostname: "git-codecommit.us-east-1.amazonaws.com",
        }
        .to_string();

        assert_eq!(
            s,
            "GIT\n/v1/repos/my-repo\n\nhost:git-codecommit.us-east-1.amazonaws.com\n\nhost\n"
        );
    }

    #[test]
    fn test_sha256() {
        let s = CanonicalRequest {
            repo: "my-repo",
            hostname: "git-codecommit.us-east-1.amazonaws.com",
        }
        .sha256()
        .to_string();

        assert_eq!(
            s,
            "a1d3c427fe57dc90a0031cb03cef21be70874879bb17c5c2ab29dfda0f514c7a"
        );
    }
}
