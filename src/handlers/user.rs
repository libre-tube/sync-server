use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, error, post, web};
use diesel::result::DatabaseErrorKind;
use utoipa_actix_web::scope;
use uuid::Uuid;

use crate::database::user::{
    delete_existing_user, find_user_by_id, find_user_by_name_hash, insert_new_user,
};
use crate::dto::LoginResponse;
use crate::handlers::{ScopedHandler, get_user};
use crate::util::{generate_jwt, hash_password, hash_username, verify_jwt, verify_password};
use crate::{WebData, dto, get_db_conn, models};

const AUTH_HEADER_KEY: &str = "Authorization";

pub struct UserHandler {}
impl ScopedHandler for UserHandler {
    fn get_service() -> scope::Scope<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = actix_web::Error,
        >,
    > {
        scope::scope("/user")
            .service(register_user)
            .service(login_user)
            // services that require auth start here
            .service(
                scope::scope("")
                    .wrap(actix_web::middleware::from_fn(auth_middleware))
                    .service(delete_user),
            )
    }
}

#[utoipa::path(responses((status = OK, body = LoginResponse)))]
#[post("/register")]
async fn register_user(
    pool: WebData,
    form: web::Json<dto::RegisterUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let user = models::User {
        id: Uuid::now_v7().to_string(),
        name_hash: hash_username(&form.name),
        password_hash: hash_password(&form.password),
    };

    let user = insert_new_user(&mut conn, &user)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                error::ErrorConflict("username already taken")
            }
            _ => error::ErrorInternalServerError(err),
        })?;

    match generate_jwt(&user) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Created().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED, body = LoginResponse)))]
#[post("/login")]
async fn login_user(
    pool: WebData,
    form: web::Json<dto::LoginUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let name = hash_username(&form.name);
    let Some(user) = find_user_by_name_hash(&mut conn, &name)
        .await
        .ok()
        .flatten()
    else {
        return Err(error::ErrorForbidden("invalid username or password"));
    };

    if !verify_password(&form.password, &user.password_hash) {
        return Err(error::ErrorForbidden("invalid username or password"));
    }

    match generate_jwt(&user) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Ok().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)))]
#[delete("/delete")]
async fn delete_user(
    req: HttpRequest,
    pool: WebData,
    form: web::Json<dto::DeleteUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let user = get_user(&req);

    if !verify_password(&form.password, &user.password_hash) {
        return Err(error::ErrorForbidden("invalid username or password"));
    }

    match delete_existing_user(&mut conn, &user.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Middleware that ensures that the user is authenticated.
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let auth_header = req
        .headers()
        .get(AUTH_HEADER_KEY)
        .and_then(|header| header.to_str().ok())
        .map(|value| value.to_string());
    let auth_cookie = req
        .cookie(AUTH_HEADER_KEY)
        .map(|cookie| cookie.value().to_string());

    let Some(jwt) = auth_cookie.or(auth_header) else {
        return Err(error::ErrorUnauthorized("missing authentication token"));
    };
    let Ok(user_id) = verify_jwt(&jwt) else {
        return Err(error::ErrorUnauthorized("invalid authentication token"));
    };

    let pool: WebData = req.app_data().cloned().unwrap();
    let mut conn = get_db_conn!(pool);

    let Some(user) = find_user_by_id(&mut conn, &user_id).await.ok().flatten() else {
        return Err(error::ErrorBadRequest("user does not exist"));
    };

    // append user to request extensions so that it can be accessed with
    // `req.extensions().get::<User>()` by handlers
    req.extensions_mut().insert(user);

    next.call(req).await
}
