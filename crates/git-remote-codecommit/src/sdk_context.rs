use anyhow::Context;
use aws_config::meta::region::ProvideRegion;
use aws_config::meta::region::RegionProviderChain;
use aws_config::AppName;
use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_config::SdkConfig;
use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::Credentials;

const APP_NAME: &str = "git-remote-codecommit";

#[derive(Debug)]
pub struct SdkContext {
    region: Region,
    credentials: Credentials,
}

impl SdkContext {
    pub fn load_context_sync(
        override_region: Option<&str>,
        override_profile: Option<&str>,
    ) -> anyhow::Result<Self> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build tokio runtime")?
            .block_on(Self::load_context(override_region, override_profile))
    }

    pub async fn load_context(
        override_region: Option<&str>,
        override_profile: Option<&str>,
    ) -> anyhow::Result<Self> {
        let mut config_loader = aws_config::ConfigLoader::default()
            .behavior_version(BehaviorVersion::v2024_03_28())
            .region(region_provider(override_region))
            .app_name(app_name());

        if let Some(profile) = override_profile {
            config_loader = config_loader.profile_name(profile);
        }

        let sdk_config = config_loader.load().await;
        Self::from_sdk_config(sdk_config).await
    }

    pub async fn from_sdk_config(sdk_config: SdkConfig) -> anyhow::Result<Self> {
        let credentials = sdk_config
            .credentials_provider()
            .context("credentials not set")?
            .provide_credentials()
            .await
            .context("failed to resolve credentials")?;

        let region = sdk_config.region().context("region not set")?.clone();

        Ok(Self {
            region,
            credentials,
        })
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

fn app_name() -> AppName {
    AppName::new(APP_NAME).expect("constant app name to be valid")
}

fn region_provider(maybe_region: Option<&str>) -> RegionProviderChain {
    RegionProviderChain::first_try(MaybeRegion::from(maybe_region)).or_default_provider()
}

#[derive(Debug)]
struct MaybeRegion(Option<Region>);

impl ProvideRegion for MaybeRegion {
    fn region(&self) -> aws_config::meta::region::future::ProvideRegion<'_> {
        aws_config::meta::region::future::ProvideRegion::ready(self.0.clone())
    }
}

impl From<Option<&str>> for MaybeRegion {
    fn from(value: Option<&str>) -> Self {
        Self(value.map(str::to_owned).map(Region::new))
    }
}
