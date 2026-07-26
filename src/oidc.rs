//! To quickly setup a test server, run
//! ```sh
//! docker run -p 9400:9400 ghcr.io/geigerzaehler/oidc-provider-mock
//! ```
//! Then in `config.toml`, set `provider_url` to `http://localhost:9400` and
//! `app_url` to `http://localhost:8080`. The other OIDC values don't matter for testing.
//!
//! Library docs: <https://docs.rs/openidconnect/latest/openidconnect/>
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::DerefMut,
    sync::{LazyLock, Mutex, OnceLock},
};

use openidconnect::{
    AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims,
    EndpointMaybeSet, EndpointNotSet, EndpointSet, IdTokenClaims, IssuerUrl, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, StandardErrorResponse, TokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType,
        CoreGenderClaim, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreProviderMetadata,
        CoreRevocableToken, CoreRevocationErrorResponse, CoreTokenIntrospectionResponse,
        CoreTokenResponse,
    },
};

use crate::config::OidcConfig;

#[allow(clippy::type_complexity)]
static CHALLENGES_STORE: LazyLock<
    Mutex<HashMap<String, (PkceCodeVerifier, Nonce, RedirectUrl, String)>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

static CLIENT: OnceLock<ApplicationOidcClient> = OnceLock::new();

// very ugly, copied from https://github.com/ramosbugs/openidconnect-rs/issues/193#issuecomment-2739072936
pub type ApplicationOidcClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointMaybeSet,
    HasUserInfoUrl = EndpointMaybeSet,
> = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    CoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
>;

#[derive(thiserror::Error, Debug)]
pub enum OidcError {
    #[error("failed to discover oidc endpoint of configured provider_url: {0}")]
    DiscoveryFailed(String),
    #[error("oidc client not initialized, please check the debug logs")]
    OidcNotInitialized,
    #[error("malformed provider uri: {0}")]
    MalformedProviderUri(String),
    #[error("malformed redirect uri: {0}")]
    MalformedRedirectUri(String),
    #[error("invalid token response: {0}")]
    InvalidTokenResponse(String),
    #[error("provided session does not exist")]
    InvalidSession,
}

pub async fn init_oidc(cfg: &OidcConfig) {
    match build_oidc_client(cfg).await {
        Ok(client) => CLIENT.set(client).unwrap(),
        Err(err) => eprintln!("{err}"),
    }
}

async fn build_oidc_client(cfg: &OidcConfig) -> Result<ApplicationOidcClient, OidcError> {
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(cfg.provider_url.clone())
            .map_err(|err| OidcError::MalformedProviderUri(err.to_string()))?,
        &http_client,
    )
    .await
    .map_err(|err| OidcError::DiscoveryFailed(err.to_string()))?;

    Ok(CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(cfg.client_id.clone()),
        Some(ClientSecret::new(cfg.client_secret.clone())),
    ))
}

pub async fn authenticate_oidc_user_request(
    cfg: &OidcConfig,
    callback_path: &str,
    app_state_extra: String,
) -> Result<String, OidcError> {
    let Some(client) = CLIENT.get() else {
        return Err(OidcError::OidcNotInitialized);
    };

    let redirect_uri = RedirectUrl::new(format!("{}{}", cfg.app_url, callback_path))
        .map_err(|err| OidcError::MalformedRedirectUri(err.to_string()))?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, state, nonce) = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .set_redirect_uri(Cow::Borrowed(&redirect_uri))
        .url();

    CHALLENGES_STORE.lock().unwrap().deref_mut().insert(
        state.secret().to_string(),
        // we store extra state for the app here that we can restore at
        // the callback from the OIDC provider and forwarded back to the app
        (pkce_verifier, nonce, redirect_uri, app_state_extra),
    );

    Ok(auth_url.to_string())
}

pub async fn check_oidc_auth_request(
    state: &str,
    code: String,
) -> Result<
    (
        IdTokenClaims<EmptyAdditionalClaims, CoreGenderClaim>,
        String,
    ),
    OidcError,
> {
    let Some(client) = CLIENT.get() else {
        return Err(OidcError::OidcNotInitialized);
    };

    let Some((pkce_verifier, nonce, redirect_uri, app_state_extra)) =
        CHALLENGES_STORE.lock().unwrap().remove(state)
    else {
        return Err(OidcError::InvalidSession);
    };

    let token_response = client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|err| OidcError::InvalidTokenResponse(err.to_string()))?
        .set_pkce_verifier(pkce_verifier)
        // required as per https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3 although
        // the openidconnect-rs example code doesn't contain it...
        .set_redirect_uri(Cow::Owned(redirect_uri))
        .request_async(&reqwest::Client::builder().build().unwrap())
        .await
        .map_err(|err| OidcError::InvalidTokenResponse(err.to_string()))?;

    let id_token = token_response.id_token().unwrap();
    let claims = id_token
        .claims(&client.id_token_verifier(), &nonce)
        .map_err(|err| OidcError::InvalidTokenResponse(err.to_string()))?;

    Ok((claims.clone(), app_state_extra))
}
