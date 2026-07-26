use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::web::Redirect;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, web};
use diesel::result::DatabaseErrorKind;
use serde::Deserialize;
use utoipa_actix_web::scope;
use uuid::Uuid;

use crate::auth::{generate_jwt, hash_accountname, hash_password, verify_jwt, verify_password};
use crate::database::account::{
    delete_existing_account, delete_existing_account_by_oidc_sub, find_account_by_id,
    find_account_by_name_hash, insert_new_account,
};
use crate::dto::LoginResponse;
use crate::handlers::{HandlerError, HandlerResult, ScopedHandler};
use crate::models::Account;
use crate::oidc::check_oidc_auth_request;
use crate::{CONFIG, WebData, dto, get_db_conn, models, oidc};

const AUTH_HEADER_KEY: &str = "Authorization";
const MIN_PASSWORD_LENGTH: usize = 8;
const OIDC_ACCOUNT_PREFIX: &str = "OIDC-ACCOUNT-";

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
        let mut s = scope::scope("/account")
            .service(register_account)
            .service(login_account);

        if CONFIG.oidc.is_some() {
            s = s
                .service(authenticate_oidc_account)
                .service(authenticate_oidc_account_callback)
                .service(delete_oidc_account)
                .service(delete_oidc_account_callback)
        };

        // services that require auth start here
        s.service(
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
) -> HandlerResult<impl Responder> {
    if !CONFIG.allow_registration {
        return Err(HandlerError::RegistrationDisabled);
    }

    // usernames starting with OIDC_ACCOUNT_PREFIX are preserved for oidc users
    if form.name.starts_with(OIDC_ACCOUNT_PREFIX) {
        return Err(HandlerError::InvalidCredentials);
    }

    let mut conn = get_db_conn!(pool);

    let password_length = form.password.len();
    if password_length < MIN_PASSWORD_LENGTH {
        return Err(HandlerError::PasswordTooShort);
    }

    let account = models::Account {
        id: Uuid::now_v7().to_string(),
        name_hash: hash_accountname(&form.name, CONFIG.secret.as_bytes()),
        password_hash: Some(hash_password(&form.password)),
        oidc_sub: None,
    };

    let account = insert_new_account(&mut conn, &account)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                HandlerError::AccountNameTaken
            }
            _ => HandlerError::InternalDatabaseErrorWithContext(err.to_string()),
        })?;

    match generate_jwt(&account, CONFIG.secret.as_bytes()) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Created().json(resp))
        }
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}

#[utoipa::path(responses((status = CREATED, body = LoginResponse)))]
#[post("/login")]
async fn login_account(
    pool: WebData,
    form: web::Json<dto::LoginUser>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let name = hash_accountname(&form.name, CONFIG.secret.as_bytes());
    let Some(account) = find_account_by_name_hash(&mut conn, &name)
        .await
        .ok()
        .flatten()
    else {
        return Err(HandlerError::InvalidCredentials);
    };

    let Some(password_hash) = &account.password_hash else {
        return Err(HandlerError::PasswordLoginDisabledForAccount);
    };

    if !verify_password(&form.password, password_hash) {
        return Err(HandlerError::InvalidCredentials);
    }

    match generate_jwt(&account, CONFIG.secret.as_bytes()) {
        Ok(jwt) => {
            let resp = LoginResponse { jwt };
            Ok(HttpResponse::Ok().json(resp))
        }
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}

#[utoipa::path(responses((status = OK)), security(("api_jwt_token" = [])))]
#[delete("/delete")]
async fn delete_account(
    account: Account,
    pool: WebData,
    form: web::Json<dto::DeleteUser>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let Some(password_hash) = &account.password_hash else {
        return Err(HandlerError::PasswordLoginDisabledForAccount);
    };

    if !verify_password(&form.password, &password_hash) {
        return Err(HandlerError::InvalidCredentials);
    }

    match delete_existing_account(&mut conn, &account.id).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
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
        return Err(HandlerError::InvalidToken.into());
    };
    let Ok(account_id) = verify_jwt(&jwt, CONFIG.secret.as_bytes()) else {
        return Err(HandlerError::InvalidToken.into());
    };

    let pool: WebData = req.app_data().cloned().unwrap();
    let mut conn = get_db_conn!(pool);

    let Some(account) = find_account_by_id(&mut conn, &account_id)
        .await
        .ok()
        .flatten()
    else {
        return Err(HandlerError::AccountNotExists.into());
    };

    // append account to request extensions so that it can be accessed with
    // `req.extensions().get::<User>()` by handlers
    req.extensions_mut().insert(account);

    next.call(req).await
}

#[derive(Deserialize)]
struct OidcAuthenticationRequest {
    /// Url to redirect to once authentication succeeded.
    /// Passes a `token` query parameter to the URL, which is a valid JWT for the authenticated account.
    redirect_url: String,
}

#[utoipa::path]
#[get("/oidc/authenticate")]
async fn authenticate_oidc_account(
    req: HttpRequest,
    query: web::Query<OidcAuthenticationRequest>,
) -> HandlerResult<impl Responder> {
    let callback_route = req
        .url_for::<&[_; 0], &String>("authenticate_oidc_account_callback", &[])
        .unwrap();

    let redirect_url = oidc::authenticate_oidc_user_request(
        &CONFIG.oidc.clone().unwrap(),
        callback_route.path(),
        query.redirect_url.clone(),
    )
    .await
    .map_err(HandlerError::OidcError)?;

    Ok(Redirect::to(redirect_url))
}

fn oidc_username_hash(oidc_sub: &str) -> String {
    // the name is getting hashed anyways, so its actual value isn't important because the user
    // never sees it
    // it only is important that the username never changes and doesn't conflict with the normally created ones
    let username = format!("{OIDC_ACCOUNT_PREFIX}{oidc_sub}");
    hash_accountname(&username, CONFIG.secret.as_bytes())
}

#[derive(Deserialize)]
struct OidcCallbackData {
    code: String,
    state: String,
}

#[utoipa::path(responses((status = CREATED, body = LoginResponse)))]
#[get("/oidc/authenticate/callback")]
async fn authenticate_oidc_account_callback(
    pool: WebData,
    query: web::Query<OidcCallbackData>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let (user_claims, redirect_url) = check_oidc_auth_request(&query.state, query.code.clone())
        .await
        .map_err(HandlerError::OidcError)?;

    let oidc_sub = user_claims.subject().as_str();

    let name_hash = oidc_username_hash(oidc_sub);
    let account = if let Some(existing_account) = find_account_by_name_hash(&mut conn, &name_hash)
        .await
        .ok()
        .flatten()
    {
        existing_account
    } else {
        let account = Account {
            id: Uuid::now_v7().to_string(),
            name_hash,
            // the password_hash field should be nullable instead of using an empty string here,
            // but unfortunately SQLite doesn't have a statement to alter table columns...
            password_hash: None,
            oidc_sub: Some(oidc_sub.to_string()),
        };
        insert_new_account(&mut conn, &account)
            .await
            .map_err(|err| HandlerError::InternalDatabaseErrorWithContext(err.to_string()))?;

        account
    };

    match generate_jwt(&account, CONFIG.secret.as_bytes()) {
        Ok(jwt) => Ok(Redirect::to(format!("{redirect_url}?token={jwt}"))),
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}

#[utoipa::path]
#[get("/oidc/delete")]
async fn delete_oidc_account(
    req: HttpRequest,
    query: web::Query<OidcAuthenticationRequest>,
) -> HandlerResult<impl Responder> {
    let callback_route = req
        .url_for::<&[_; 0], &String>("delete_oidc_account_callback", &[])
        .unwrap();

    let redirect_url = oidc::authenticate_oidc_user_request(
        &CONFIG.oidc.clone().unwrap(),
        callback_route.path(),
        query.redirect_url.clone(),
    )
    .await
    .map_err(HandlerError::OidcError)?;

    Ok(Redirect::to(redirect_url))
}

#[utoipa::path(responses((status = CREATED, body = LoginResponse)))]
#[get("/oidc/delete/callback")]
async fn delete_oidc_account_callback(
    pool: WebData,
    query: web::Query<OidcCallbackData>,
) -> HandlerResult<impl Responder> {
    let mut conn = get_db_conn!(pool);

    let (user_claims, redirect_url) = check_oidc_auth_request(&query.state, query.code.clone())
        .await
        .map_err(HandlerError::OidcError)?;

    let oidc_sub = user_claims.subject().as_str();

    match delete_existing_account_by_oidc_sub(&mut conn, oidc_sub).await {
        Ok(deleted) => {
            if deleted {
                Ok(Redirect::to(redirect_url))
            } else {
                Err(HandlerError::AccountNotExists)
            }
        }
        Err(err) => Err(HandlerError::InternalDatabaseErrorWithContext(
            err.to_string(),
        )),
    }
}
