mod error;

use uriparse::Host;
use uriparse::RegisteredName;
use uriparse::Scheme;
use uriparse::Username;
use uriparse::URI;

pub use self::error::ParseUriError;

const SCHEME: &str = "codecommit";

// Note the double colon. It is not a typo.
const PREFIX_WITH_REGION: &str = "codecommit::";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedUri<'a> {
    region: Option<Scheme<'a>>,
    profile: Option<Username<'a>>,
    repository: RegisteredName<'a>,
}

impl ParsedUri<'_> {
    pub fn region(&self) -> Option<&str> {
        self.region.as_ref().map(Scheme::as_str)
    }

    pub fn profile(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    pub fn repository(&self) -> &str {
        self.repository.as_str()
    }

    pub fn into_owned(self) -> ParsedUri<'static> {
        ParsedUri {
            region: self.region.map(Scheme::into_owned),
            profile: self.profile.map(Username::into_owned),
            repository: self.repository.into_owned(),
        }
    }
}

impl<'a> TryFrom<&'a String> for ParsedUri<'a> {
    type Error = ParseUriError;

    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl<'a> TryFrom<&'a str> for ParsedUri<'a> {
    type Error = ParseUriError;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        // Git removes this prefix before invoking the helper; but, we're checking for
        // it anyways to be safe as otherwise it would be an invalid URI.
        let value = input.strip_prefix(PREFIX_WITH_REGION).unwrap_or(input);

        let (scheme, authority, path, query, fragment) = URI::try_from(value)?.into_parts();

        let (profile, password, host, port) = authority
            .ok_or(ParseUriError::MissingAuthority)?
            .into_parts();

        path.segments()
            .single()
            .map_or(false, |only| only.is_empty() || only == "/")
            .ok_or(ParseUriError::UnexpectedPath)?;

        query.is_none().ok_or(ParseUriError::UnexpectedQuery)?;

        fragment
            .is_none()
            .ok_or(ParseUriError::UnexpectedFragment)?;

        password
            .is_none()
            .ok_or(ParseUriError::UnexpectedPassword)?;

        port.is_none().ok_or(ParseUriError::UnexpectedPort)?;

        let repository = match host {
            Host::RegisteredName(rn) => rn,
            _ => return Err(ParseUriError::UnexpectedIpForRepositoryName),
        };

        // TODO: should this be validated? The original code validates it to
        // `\w{2}-\w*.*-\d` which is a bit too strict.
        let region = if scheme == SCHEME { None } else { Some(scheme) };

        Ok(Self {
            region,
            profile,
            repository,
        })
    }
}

impl core::fmt::Display for ParsedUri<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(region) = self.region() {
            f.write_str(PREFIX_WITH_REGION)?;
            f.write_str(region)?;
        } else {
            f.write_str(SCHEME)?;
        }

        f.write_str("://")?;

        if let Some(profile) = self.profile() {
            f.write_str(profile)?;
            f.write_str("@")?;
        }

        f.write_str(self.repository())
    }
}

trait SingleExt: IntoIterator {
    fn single(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        let mut iter = self.into_iter();
        let first = iter.next()?;
        if iter.next().is_some() {
            None
        } else {
            Some(first)
        }
    }
}

impl<T: IntoIterator> SingleExt for T {}

trait BoolExt {
    fn ok_or<E>(self, or: E) -> Result<(), E>;
}

impl BoolExt for bool {
    fn ok_or<E>(self, or: E) -> Result<(), E> {
        if self {
            Ok(())
        } else {
            Err(or)
        }
    }
}
