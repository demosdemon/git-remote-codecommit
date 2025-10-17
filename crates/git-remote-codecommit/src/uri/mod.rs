#![cfg_attr(not(bool_to_result), allow(unstable_name_collisions))]

mod error;

use std::ops::Not;

use uriparse::Host;
use uriparse::RegisteredName;
use uriparse::Scheme;
use uriparse::URI;
use uriparse::Username;

pub use self::error::ParseUriError;
#[cfg(not(bool_to_result))]
use crate::nightly::BoolExt;

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

    #[cfg_attr(not(test), expect(dead_code))]
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
            .is_some_and(|only| only.is_empty() || only == "/")
            .ok_or(ParseUriError::UnexpectedPath)?;

        query.is_none().ok_or(ParseUriError::UnexpectedQuery)?;

        fragment
            .is_none()
            .ok_or(ParseUriError::UnexpectedFragment)?;

        password
            .is_none()
            .ok_or(ParseUriError::UnexpectedPassword)?;

        port.is_none().ok_or(ParseUriError::UnexpectedPort)?;

        let Host::RegisteredName(repository) = host else {
            return Err(ParseUriError::UnexpectedIpForRepositoryName);
        };

        repository
            .is_empty()
            .not()
            .ok_or(ParseUriError::EmptyRepositoryName)?;

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
        // note: https://doc.rust-lang.org/nightly/edition-guide/rust-2024/temporary-tail-expr-scope.html
        if iter.next().is_none() {
            Some(first)
        } else {
            None
        }
    }
}

impl<T: IntoIterator> SingleExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_example() {
        let parsed_uri = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");
        assert_eq!(None, parsed_uri.region());
        assert_eq!(None, parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_base_example_with_profile() {
        let parsed_uri = ParsedUri::try_from("codecommit://my-profile@my-repo").expect("valid URI");
        assert_eq!(None, parsed_uri.region());
        assert_eq!(Some("my-profile"), parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_region_example() {
        let parsed_uri = ParsedUri::try_from("codecommit::us-east-1://my-repo").expect("valid URI");
        assert_eq!(Some("us-east-1"), parsed_uri.region());
        assert_eq!(None, parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_region_example_with_profile() {
        let parsed_uri =
            ParsedUri::try_from("codecommit::us-east-1://my-profile@my-repo").expect("valid URI");
        assert_eq!(Some("us-east-1"), parsed_uri.region());
        assert_eq!(Some("my-profile"), parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_region_from_git_example() {
        let parsed_uri = ParsedUri::try_from("us-east-1://my-repo").expect("valid URI");
        assert_eq!(Some("us-east-1"), parsed_uri.region());
        assert_eq!(None, parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_region_from_git_example_with_profile() {
        let parsed_uri = ParsedUri::try_from("us-east-1://my-profile@my-repo").expect("valid URI");
        assert_eq!(Some("us-east-1"), parsed_uri.region());
        assert_eq!(Some("my-profile"), parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_invalid_uri() {
        assert!(matches!(
            ParsedUri::try_from("codecommit::my-repo"),
            Err(ParseUriError::InvalidUri(_))
        ));
    }

    #[test]
    fn test_missing_authority() {
        assert_eq!(
            Err(ParseUriError::MissingAuthority),
            ParsedUri::try_from("codecommit:"),
        );
    }

    #[test]
    fn test_unexpected_path() {
        assert_eq!(
            Err(ParseUriError::UnexpectedPath),
            ParsedUri::try_from("codecommit:///my-repo"),
        );
    }

    #[test]
    fn test_unexpected_query() {
        assert_eq!(
            Err(ParseUriError::UnexpectedQuery),
            ParsedUri::try_from("codecommit://my-repo?query"),
        );
    }

    #[test]
    fn test_unexpected_fragment() {
        assert_eq!(
            Err(ParseUriError::UnexpectedFragment),
            ParsedUri::try_from("codecommit://my-repo#fragment"),
        );
    }

    #[test]
    fn test_unexpected_password() {
        assert_eq!(
            Err(ParseUriError::UnexpectedPassword),
            ParsedUri::try_from("codecommit://user:pass@my-repo"),
        );
    }

    #[test]
    fn test_unexpected_port() {
        assert_eq!(
            Err(ParseUriError::UnexpectedPort),
            ParsedUri::try_from("codecommit://my-repo:1234"),
        );
    }

    #[test]
    fn test_unexpected_ipv4_for_repo() {
        assert_eq!(
            Err(ParseUriError::UnexpectedIpForRepositoryName),
            ParsedUri::try_from("codecommit://127.0.0.1"),
        );
    }

    #[test]
    fn test_unexpected_ipv6_for_repo() {
        assert_eq!(
            Err(ParseUriError::UnexpectedIpForRepositoryName),
            ParsedUri::try_from("codecommit://[::1]"),
        );
    }

    #[test]
    fn test_empty_repo_name() {
        assert_eq!(
            Err(ParseUriError::EmptyRepositoryName),
            ParsedUri::try_from("codecommit://"),
        );
    }

    #[test]
    fn test_to_owned() {
        let parsed_uri = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");
        let owned = parsed_uri.into_owned();
        assert_eq!(None, owned.region());
        assert_eq!(None, owned.profile());
        assert_eq!("my-repo", owned.repository());
    }

    #[test]
    fn test_try_from_owned() {
        let s = "codecommit://my-repo".to_owned();
        let parsed_uri = ParsedUri::try_from(&s).expect("valid URI");
        assert_eq!(None, parsed_uri.region());
        assert_eq!(None, parsed_uri.profile());
        assert_eq!("my-repo", parsed_uri.repository());
    }

    #[test]
    fn test_to_string() {
        let parsed_uri = ParsedUri::try_from("codecommit://my-repo")
            .expect("valid URI")
            .to_string();
        assert_eq!("codecommit://my-repo", parsed_uri);
    }

    #[test]
    fn test_to_string_with_profile() {
        let parsed_uri = ParsedUri::try_from("codecommit://my-profile@my-repo")
            .expect("valid URI")
            .to_string();
        assert_eq!("codecommit://my-profile@my-repo", parsed_uri);
    }

    #[test]
    fn test_to_string_with_region() {
        let parsed_uri = ParsedUri::try_from("codecommit::us-west-2://my-repo")
            .expect("valid URI")
            .to_string();
        assert_eq!("codecommit::us-west-2://my-repo", parsed_uri);
    }

    #[test]
    fn test_to_string_with_profile_and_region() {
        let parsed_uri = ParsedUri::try_from("codecommit::us-west-2://my-profile@my-repo")
            .expect("valid URI")
            .to_string();
        assert_eq!("codecommit::us-west-2://my-profile@my-repo", parsed_uri);
    }
}
