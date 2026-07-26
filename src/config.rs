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

pub fn build_config() -> Result<Config, ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::with_convert_case(config::Case::Snake))
        .build()?
        .try_deserialize()
}
