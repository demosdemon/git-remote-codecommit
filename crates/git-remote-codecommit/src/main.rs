mod canonical_request;
mod credential_scope;
mod datetime;
mod hex;
mod hostname;
mod logging;
mod sdk_context;
mod string_to_sign;
mod uri;
mod urlsafe;
mod username;

use std::time::SystemTime;

use anyhow::Context;
use clap::Parser;
use hmac::digest::FixedOutput;
use hmac::Mac;
use tracing::debug;
use tracing::trace;

use self::canonical_request::CanonicalRequest;
use self::credential_scope::CredentialScope;
use self::datetime::TimestampExt;
use self::hex::HexDisplayExt;
use self::hostname::InferredHostname;
use self::sdk_context::SdkContext;
use self::string_to_sign::StringToSign;
use self::uri::ParsedUri;
use self::urlsafe::UrlSafeQuote;
use self::username::Username;

const SERVICE: &str = "codecommit";

const URL_PATH_PREFIX: &str = "v1/repos";

#[derive(Debug, Clone, Parser)]
#[command(name = "git-remote-codecommit", version, about)]
/// A Git remote helper for AWS CodeCommit.
///
/// This is normally invoked by git any time it needs to interact with a remote
/// with the `codecommit://` scheme.
///
/// https://git-scm.com/docs/gitremote-helpers
///
/// Git invokes the helper with one or two arguments; however, this helper
/// requires both arguments to be present. See the url above for more details;
/// but briefly:
///
/// - The first argument is the name of the remote. In most cases, this is the
///   name of the remote configured in the git repo. However, this can also be
///   the URL to the remote if URL was encountered on the command line.
///
/// - The second argument is the url of the remote. Git will not provide this if
///   the remote is configured in the config as `remote.<name>.vcs = codecommit`
///   and `remote.<name>.url` is not set. This is not supported.
///
/// ## URL format
///
/// This helper accepts the following URLs:
///
/// - `codecommit://[<profile>@]<repository>`: Use the default AWS region. Use
///   the specified profile otherwise use the default.
///
/// - `codecommit::<region>://[<profile>@]<repository>`: Override the AWS
///   region.
///
///   - Note: Git strips the `codecommit::` prefix when invoking the helper and
///     the remote uses the region form.
struct Cli {
    /// Override the default AWS endpoint for CodeCommit.
    ///
    /// If not provided, the default is
    /// `git-codecommit.${region}.${aws-partition}`.
    ///
    /// Where `${region}` is taken from the environment or profile and
    /// `${aws-partition}` is `amazonaws.com` for AWS regions and
    /// `amazonaws.cn` for AWS China regions.
    #[arg(long, env, value_name = "URL")]
    code_commit_endpoint: Option<String>,

    /// The first argument to the git-remote helper.
    remote_name: String,

    /// The second argument to the git-remote helper.
    remote_uri: String,
}

fn main() -> anyhow::Result<()> {
    crate::logging::init_logging();
    trace!("initialized logging");

    let Cli {
        code_commit_endpoint,
        remote_name,
        remote_uri,
    } = Cli::parse();
    debug!(
        ?code_commit_endpoint,
        ?remote_name,
        ?remote_uri,
        "parsed cli arguments"
    );

    let parsed_uri = ParsedUri::try_from(&remote_uri).context("failed to parse uri")?;
    debug!(?parsed_uri, "parsed uri");

    let sdk_context = SdkContext::load_context_sync(parsed_uri.region(), parsed_uri.profile())?;
    debug!(?sdk_context, "loaded sdk context");

    let url = generate_url(
        SystemTime::now(),
        parsed_uri,
        code_commit_endpoint,
        sdk_context,
    );
    debug!(?url, "generated url");

    let err = exec::execvp("git", ["git", "remote-https", &remote_name, &url]);
    anyhow::bail!("failed to execute git: {err}")
}

fn generate_url(
    timestamp: SystemTime,
    parsed_uri: ParsedUri<'_>,
    override_endpoint: Option<String>,
    sdk_context: SdkContext,
) -> String {
    let hostname = override_endpoint
        .unwrap_or_else(|| InferredHostname::new(sdk_context.region().as_ref()).to_string());
    debug!(?hostname, "using hostname for codecommit endpoint");

    let username = Username {
        access_key_id: sdk_context.credentials().access_key_id(),
        session_token: sdk_context.credentials().session_token(),
    }
    .to_string();
    debug!(?username, "generated username");

    let signature = generate_signature(timestamp, &hostname, parsed_uri.repository(), &sdk_context);
    debug!(?signature, "generated signature");

    format!(
        "https://{username}:{signature}@{hostname}/{URL_PATH_PREFIX}/{repo}",
        username = UrlSafeQuote(&username),
        repo = parsed_uri.repository(),
    )
}

fn generate_signature(
    timestamp: SystemTime,
    hostname: &str,
    repo: &str,
    context: &SdkContext,
) -> String {
    let region = context.region().as_ref();

    let string_to_sign = StringToSign {
        timestamp,
        credential_scope: CredentialScope { timestamp, region },
        canonical_request: CanonicalRequest { repo, hostname },
    };

    if tracing::enabled!(tracing::Level::DEBUG) {
        let canonical_request = string_to_sign.canonical_request.to_string();
        debug!(?canonical_request, "canonical request for signature");
    }

    let string_to_sign = string_to_sign.to_string();
    debug!(?string_to_sign, "string to sign");

    let signing_key = aws_sigv4::sign::v4::generate_signing_key(
        context.credentials().secret_access_key(),
        timestamp,
        region,
        SERVICE,
    );

    let signature = hmac::Hmac::<sha2::Sha256>::new_from_slice(signing_key.as_ref())
        .expect("HMAC can take key of any size")
        .chain_update(string_to_sign.as_bytes())
        .finalize_fixed();

    format!(
        "{}Z{}",
        timestamp.sigv4_timestamp(),
        signature.hex_display()
    )
}

#[cfg(test)]
mod tests {
    use aws_config::BehaviorVersion;
    use aws_config::SdkConfig;
    use aws_credential_types::Credentials;

    use super::*;

    async fn load_test_sdk_config() -> SdkConfig {
        aws_config::ConfigLoader::default()
            .behavior_version(BehaviorVersion::v2024_03_28())
            .region("us-east-1")
            .credentials_provider(Credentials::for_tests())
            .load()
            .await
    }

    async fn load_test_sdk_config_with_session_token() -> SdkConfig {
        aws_config::ConfigLoader::default()
            .behavior_version(BehaviorVersion::v2024_03_28())
            .region("us-east-1")
            .credentials_provider(Credentials::for_tests_with_session_token())
            .load()
            .await
    }

    #[test]
    fn test_generate_url() {
        let sdk_context = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(async {
                let config = load_test_sdk_config().await;
                SdkContext::from_sdk_config(config).await
            })
            .expect("failed to load context");

        let parsed_url = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");

        let url = generate_url(SystemTime::UNIX_EPOCH, parsed_url, None, sdk_context);

        assert_eq!(url, "https://ANOTREAL:19700101T000000Zf840ae3ff903ddb92c450d0e3567fe97ef4aa98bd6636905df48c3beee97d21d@git-codecommit.us-east-1.amazonaws.com/v1/repos/my-repo");
    }

    #[test]
    fn test_generate_url_with_override() {
        let sdk_context = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(async {
                let config = load_test_sdk_config().await;
                SdkContext::from_sdk_config(config).await
            })
            .expect("failed to load context");

        let parsed_url = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");

        let url = generate_url(
            SystemTime::UNIX_EPOCH,
            parsed_url,
            Some("localhost:8443".to_owned()),
            sdk_context,
        );

        assert_eq!(url, "https://ANOTREAL:19700101T000000Za305b3ce69941e8f0773a2257d9059df41dfc3a4d2563a42948e84ec4825ec06@localhost:8443/v1/repos/my-repo");
    }

    #[test]
    fn test_generate_url_with_session_token() {
        let sdk_context = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(async {
                let config = load_test_sdk_config_with_session_token().await;
                SdkContext::from_sdk_config(config).await
            })
            .expect("failed to load context");

        let parsed_url = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");

        let url = generate_url(SystemTime::UNIX_EPOCH, parsed_url, None, sdk_context);

        assert_eq!(url, "https://ANOTREAL%25notarealsessiontoken:19700101T000000Zf840ae3ff903ddb92c450d0e3567fe97ef4aa98bd6636905df48c3beee97d21d@git-codecommit.us-east-1.amazonaws.com/v1/repos/my-repo");
    }

    #[test]
    fn test_generate_url_with_session_token_with_override() {
        let sdk_context = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime")
            .block_on(async {
                let config = load_test_sdk_config_with_session_token().await;
                SdkContext::from_sdk_config(config).await
            })
            .expect("failed to load context");

        let parsed_url = ParsedUri::try_from("codecommit://my-repo").expect("valid URI");

        let url = generate_url(
            SystemTime::UNIX_EPOCH,
            parsed_url,
            Some("localhost:8443".to_owned()),
            sdk_context,
        );

        assert_eq!(url, "https://ANOTREAL%25notarealsessiontoken:19700101T000000Za305b3ce69941e8f0773a2257d9059df41dfc3a4d2563a42948e84ec4825ec06@localhost:8443/v1/repos/my-repo");
    }
}
