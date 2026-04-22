use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
};
use utoipa_actix_web::scope::Scope;

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
