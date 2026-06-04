use std::pin::Pin;

use actix_web::{
    FromRequest, HttpMessage, HttpRequest,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    error::ResponseError,
    http::StatusCode,
};
use utoipa_actix_web::scope::Scope;

use crate::models::Account;

pub mod health;
pub mod playlist_bookmarks;
pub mod playlists;
pub mod subscriptions;
pub mod user;
pub mod watch_history;

pub mod utils;

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("bookmark doesn't exists")]
    BookmarkNotExists,
    #[error("playlist doesn't exists")]
    PlaylistNotExists,
    #[error("account doesn't exists")]
    AccountNotExists,
    #[error("not the owner of the playlist")]
    PlaylistNotOwned,
    #[error("playlist already exists")]
    PlaylistExists,
    #[error("not subscribed to this channel")]
    NotSubscribed,
    #[error("subscription group doesn't exist or doesn't belong to this account")]
    SubscriptionGroupNotFound,
    #[error("channel has to be subscribed to before it can be added to a channel group")]
    SubscribeBeforeChannelGroup,
    #[error("registration is disabled on this server")]
    RegistrationDisabled,
    #[error("password too short (8 chars min)")]
    PasswordTooShort,
    #[error("accountname already taken")]
    AccountNameTaken,
    #[error("invalid accountname or password")]
    InvalidCredentials,
    #[error("invalid or missing authentication token")]
    InvalidToken,
    #[error("video not in watch history")]
    NotInWatchHistory,
    #[error("internal database error")]
    InternalDatabaseError,
    #[error("internal database error: {0}")]
    InternalDatabaseErrorWithContext(String),
    #[error("provided metadata seems to be wrong")]
    ValidationError,
    #[error("provided metadata seems to be wrong: {0}")]
    ValidationErrorWithContext(String),
    #[error("failed to load data from YouTube")]
    YouTubeConnectError,
}

impl ResponseError for HandlerError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::BookmarkNotExists => StatusCode::NOT_FOUND,
            Self::PlaylistNotExists => StatusCode::NOT_FOUND,
            Self::PlaylistNotOwned => StatusCode::FORBIDDEN,
            Self::PlaylistExists => StatusCode::CONFLICT,
            Self::NotSubscribed => StatusCode::BAD_REQUEST,
            Self::SubscriptionGroupNotFound => StatusCode::NOT_FOUND,
            Self::SubscribeBeforeChannelGroup => StatusCode::BAD_REQUEST,
            Self::RegistrationDisabled => StatusCode::METHOD_NOT_ALLOWED,
            Self::PasswordTooShort => StatusCode::BAD_REQUEST,
            Self::AccountNameTaken => StatusCode::CONFLICT,
            Self::AccountNotExists => StatusCode::NOT_FOUND,
            Self::InvalidCredentials => StatusCode::FORBIDDEN,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::NotInWatchHistory => StatusCode::NOT_FOUND,
            Self::InternalDatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalDatabaseErrorWithContext(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ValidationError => StatusCode::BAD_REQUEST,
            Self::ValidationErrorWithContext(_) => StatusCode::BAD_REQUEST,
            Self::YouTubeConnectError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type HandlerResult<T> = Result<T, HandlerError>;

// https://github.com/actix/actix-web/discussions/3074
pub trait ScopedHandler {
    fn get_service() -> Scope<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    >;
}

impl FromRequest for Account {
    type Error = actix_web::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let extensions = req.extensions();
        let account = extensions.get::<Account>().cloned();
        Box::pin(
            async move { account.ok_or(actix_web::error::ErrorForbidden("missing account info")) },
        )
    }
}

#[macro_export]
macro_rules! get_db_conn {
    ($pool:ident) => {
        $pool
            .get()
            .await
            .expect("Couldn't get db connection from the pool")
    };
}
