use config::ConfigError;

const fn default_true() -> bool {
    true
}

#[derive(serde::Deserialize, Clone)]
pub struct OidcConfig {
    pub provider_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub app_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "secret_key")]
    pub secret: String,
    #[serde(default = "default_true")]
    pub allow_registration: bool,
    #[serde(default = "default_true")]
    pub validate_submitted_metadata: bool,
    pub database_url: String,
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
}

impl Config {
    /// The default value for secrets in the config.
    const DEFAULT_SECRET_KEY: &str = "changeme";

    /// Whether the config is using default values for secrets.
    pub fn is_using_default_secrets(&self) -> bool {
        if self.secret == Self::DEFAULT_SECRET_KEY {
            log::error!("`secret_key` is using the default value, please change it");
            return true;
        }

        if let Some(oidc) = &self.oidc {
            if oidc.client_id == Self::DEFAULT_SECRET_KEY {
                log::error!("`oidc.client_id` is using the default value, please change it");
                return true;
            }

            if oidc.client_secret == Self::DEFAULT_SECRET_KEY {
                log::error!("`oidc.client_secret` is using the default value, please change it");
                return true;
            }
        }

        false
    }
}

pub fn build_config() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::with_convert_case(config::Case::Snake))
        .build()?
        .try_deserialize()
}
