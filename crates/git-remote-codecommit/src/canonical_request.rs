use std::io::Write;

use hmac::digest::FixedOutput;
use sha2::Digest;

use crate::HexDisplayExt;
use crate::URL_PATH_PREFIX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanonicalRequest<'a> {
    pub repo: &'a str,
    pub hostname: &'a str,
}

impl CanonicalRequest<'_> {
    pub fn sha256(&self) -> impl core::fmt::Display {
        let mut hasher = sha2::Sha256::new();
        write!(hasher, "{self}").expect("writing to hasher cannot fail");
        hasher.finalize_fixed().hex_display()
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
