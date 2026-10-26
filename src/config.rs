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

    /// Constructs the config using the `config.toml` file and environment variables.
    pub fn build() -> Result<Config, ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_convert_case(config::Case::Snake))
            .build()?
            .try_deserialize()
    }

    /// Validates the config.
    pub fn validate(&self) -> Result<(), Vec<ConfigValidationError>> {
        let mut errors = Vec::new();

        if self.secret == Self::DEFAULT_SECRET_KEY {
            errors.push(ConfigValidationError::DefaultSecret("secret_key"));
        }

        if let Some(oidc) = &self.oidc {
            if oidc.client_id == Self::DEFAULT_SECRET_KEY {
                errors.push(ConfigValidationError::DefaultSecret("oidc.client_id"));
            }

            if oidc.client_secret == Self::DEFAULT_SECRET_KEY {
                errors.push(ConfigValidationError::DefaultSecret("oidc.client_secret"));
            }

            if oidc.provider_url.is_empty() {
                errors.push(ConfigValidationError::InvalidUrl("oidc.provider_url"));
            }

            if oidc.app_url.is_empty() {
                errors.push(ConfigValidationError::InvalidUrl("oidc.app_url"));
            }
        }

        if self.database_url.is_empty() {
            errors.push(ConfigValidationError::InvalidUrl("database_url"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Invalid config errors.
#[derive(thiserror::Error, Debug)]
pub enum ConfigValidationError {
    /// A default value has been used for a secret.
    #[error("`{0}` is using the default value, please change it")]
    DefaultSecret(&'static str),
    /// The set URL is not valid.
    #[error("`{0}` is not a valid URL")]
    InvalidUrl(&'static str),
}
