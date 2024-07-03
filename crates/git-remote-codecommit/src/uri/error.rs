use uriparse::URIError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseUriError {
    InvalidUri(URIError),
    MissingAuthority,
    UnexpectedPath,
    UnexpectedQuery,
    UnexpectedFragment,
    UnexpectedPassword,
    UnexpectedPort,
    UnexpectedIpForRepositoryName,
}

impl From<URIError> for ParseUriError {
    fn from(value: URIError) -> Self {
        Self::InvalidUri(value)
    }
}

impl core::fmt::Display for ParseUriError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidUri(err) => write!(f, "invalid URI: {err}"),
            Self::MissingAuthority => f.write_str("missing authority"),
            Self::UnexpectedPath => f.write_str("unexpected path"),
            Self::UnexpectedQuery => f.write_str("unexpected query"),
            Self::UnexpectedFragment => f.write_str("unexpected fragment"),
            Self::UnexpectedPassword => f.write_str("unexpected password"),
            Self::UnexpectedPort => f.write_str("unexpected port"),
            Self::UnexpectedIpForRepositoryName => f.write_str("unexpected IP for repository name"),
        }
    }
}

impl std::error::Error for ParseUriError {}
