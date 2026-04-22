use actix_web::{
    HttpMessage, HttpRequest,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
};
use utoipa_actix_web::scope::Scope;

use crate::models::User;

pub mod playlists;
pub mod subscriptions;
pub mod user;

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

#[macro_export]
macro_rules! get_db_conn {
    ($pool:ident) => {
        $pool
            .get()
            .await
            .expect("Couldn't get db connection from the pool")
    };
}

pub(crate) fn get_user(req: &HttpRequest) -> User {
    let extensions = req.extensions();
    let user = extensions.get::<User>().unwrap();
    user.clone()
}
