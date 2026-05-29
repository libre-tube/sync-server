use std::pin::Pin;

use actix_web::{
    FromRequest, HttpMessage, HttpRequest,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    error,
};
use utoipa_actix_web::scope::Scope;

use crate::models::Account;

pub mod health;
pub mod playlist_bookmarks;
pub mod playlists;
pub mod subscriptions;
pub mod user;
pub mod watch_history;

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
    RegistrationIsDisabled,
    #[error("password too short (8 chars min)")]
    PasswordTooShort,
    #[error("accountname already taken")]
    AccountnameTaken,
    #[error("invalid accountname or password")]
    InvalidCredentials,
    #[error("invalid or missing authentication token")]
    InvalidToken,
    #[error("video not in watch history")]
    NotInWatchHistory,
}

impl Into<actix_web::Error> for HandlerError {
    fn into(self) -> actix_web::Error {
        match self {
            Self::BookmarkNotExists => error::ErrorNotFound(self),
            Self::PlaylistNotExists => error::ErrorNotFound(self),
            Self::PlaylistNotOwned => error::ErrorForbidden(self),
            Self::PlaylistExists => error::ErrorConflict(self),
            Self::NotSubscribed => error::ErrorBadRequest(self),
            Self::SubscriptionGroupNotFound => error::ErrorNotFound(self),
            Self::SubscribeBeforeChannelGroup => error::ErrorBadRequest(self),
            Self::RegistrationIsDisabled => error::ErrorMethodNotAllowed(self),
            Self::PasswordTooShort => error::ErrorBadRequest(self),
            Self::AccountnameTaken => error::ErrorConflict(self),
            Self::AccountNotExists => error::ErrorNotFound(self),
            Self::InvalidCredentials => error::ErrorForbidden(self),
            Self::InvalidToken => error::ErrorUnauthorized(self),
            Self::NotInWatchHistory => error::ErrorNotFound(self),
        }
    }
}

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
