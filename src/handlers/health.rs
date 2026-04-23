use actix_web::{Responder, routes};
use utoipa_actix_web::scope;

use crate::handlers::ScopedHandler;

pub struct HealthHandler {}
impl ScopedHandler for HealthHandler {
    fn get_service() -> utoipa_actix_web::scope::Scope<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope::scope("").service(health_state)
    }
}

#[utoipa::path(responses((status = OK, body = String)))]
#[routes]
#[get("/")]
#[get("/health")]
#[get("/healthz")]
async fn health_state() -> impl Responder {
    "OK"
}
