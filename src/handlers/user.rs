use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, error, post, web};
use diesel::result::DatabaseErrorKind;
use utoipa_actix_web::scope;
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_accountname, hash_password, verify_jwt, verify_password};
use crate::database::account::{
    delete_existing_account, find_account_by_id, find_account_by_name_hash, insert_new_account,
};
use crate::dto::LoginResponse;
use crate::handlers::{ScopedHandler, get_account};
use crate::{SECRET_KEY, WebData, dto, get_db_conn, models};

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
        scope::scope("/account")
            .service(register_account)
            .service(login_account)
            // services that require auth start here
            .service(
                scope::scope("")
                    .wrap(actix_web::middleware::from_fn(auth_middleware))
                    .service(delete_account),
            )
    }
}

#[utoipa::path(responses((status = OK, body = LoginResponse)))]
#[post("/register")]
async fn register_account(
    pool: WebData,
    form: web::Json<dto::RegisterUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let password_length = form.password.len();
    if password_length < 8 {
        return Err(error::ErrorBadRequest("password too short (8 chars min)"));
    }

    let account = models::Account {
        id: Uuid::now_v7().to_string(),
        name_hash: hash_accountname(&form.name, SECRET_KEY.as_bytes()),
        password_hash: hash_password(&form.password),
    };

    let account = insert_new_account(&mut conn, &account)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                error::ErrorConflict("accountname already taken")
            }
            _ => error::ErrorInternalServerError(err),
        })?;

    match generate_jwt(&account, SECRET_KEY.as_bytes()) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Created().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = CREATED, body = LoginResponse)))]
#[post("/login")]
async fn login_account(
    pool: WebData,
    form: web::Json<dto::LoginUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let name = hash_accountname(&form.name, SECRET_KEY.as_bytes());
    let Some(account) = find_account_by_name_hash(&mut conn, &name)
        .await
        .ok()
        .flatten()
    else {
        return Err(error::ErrorForbidden("invalid accountname or password"));
    };

    if !verify_password(&form.password, &account.password_hash) {
        return Err(error::ErrorForbidden("invalid accountname or password"));
    }

    match generate_jwt(&account, SECRET_KEY.as_bytes()) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Ok().json(resp))
        }
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

#[utoipa::path(responses((status = OK)))]
#[delete("/delete")]
async fn delete_account(
    req: HttpRequest,
    pool: WebData,
    form: web::Json<dto::DeleteUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = get_db_conn!(pool);
    let account = get_account(&req);

    if !verify_password(&form.password, &account.password_hash) {
        return Err(error::ErrorForbidden("invalid accountname or password"));
    }

    match delete_existing_account(&mut conn, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(error::ErrorInternalServerError(err)),
    }
}

/// Middleware that ensures that the account is authenticated.
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
    let Ok(account_id) = verify_jwt(&jwt, SECRET_KEY.as_bytes()) else {
        return Err(error::ErrorUnauthorized("invalid authentication token"));
    };

    let pool: WebData = req.app_data().cloned().unwrap();
    let mut conn = get_db_conn!(pool);

    let Some(account) = find_account_by_id(&mut conn, &account_id)
        .await
        .ok()
        .flatten()
    else {
        return Err(error::ErrorBadRequest("account does not exist"));
    };

    // append account to request extensions so that it can be accessed with
    // `req.extensions().get::<User>()` by handlers
    req.extensions_mut().insert(account);

    next.call(req).await
}
