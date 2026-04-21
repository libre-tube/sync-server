use actix_web::{HttpResponse, Responder, error, get, post, web};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use hmac::{Hmac, KeyInit, Mac as _};
use sha2::Sha256;
use uuid::Uuid;

use crate::database::user::{find_user_by_id, insert_new_user};
use crate::util::bytes_to_hex_string;
use crate::{DbPool, models};

// TODO: make configurable
const SECRET_KEY: &str = "secret";

#[get("/user/{user_id}")]
async fn get_user(
    pool: web::Data<DbPool>,
    item_id: web::Path<Uuid>,
) -> actix_web::Result<impl Responder> {
    let user_id = item_id.into_inner();

    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    let user = find_user_by_id(&mut conn, user_id)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(match user {
        Some(item) => HttpResponse::Ok().json(item),
        None => HttpResponse::NotFound().body(format!("No item found with UID: {user_id}")),
    })
}

#[post("/user/register")]
async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> actix_web::Result<impl Responder> {
    let mut conn = pool
        .get()
        .await
        .expect("Couldn't get db connection from the pool");

    let user = models::User {
        id: Uuid::now_v7().to_string(),
        name_hash: hash_username(&form.name),
        password_hash: hash_password(&form.password),
    };

    let user = insert_new_user(&mut conn, &user)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(user))
}

fn argon2_instance<'a>() -> Argon2<'a> {
    Argon2::default()
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(password_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    argon2_instance()
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok()
}

fn hash_username(username: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(username.as_bytes()).unwrap();
    mac.update(SECRET_KEY.as_bytes());

    let result = &mac.finalize().into_bytes();
    bytes_to_hex_string(result)
}
