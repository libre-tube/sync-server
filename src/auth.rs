use std::time::{Duration, SystemTime, UNIX_EPOCH};

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use hmac::{Hmac, KeyInit, Mac as _};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sha2::Sha256;

use crate::dto::JwtClaims;
use crate::models::Account;

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    String::from("0x")
        + &bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("")
}

pub fn generate_jwt(account: &Account, secret_key: &[u8]) -> jsonwebtoken::errors::Result<String> {
    let key = EncodingKey::from_secret(secret_key);
    // tokens are valid for one year, should be enough in most cases
    let expiration_date = SystemTime::now()
        .checked_add(Duration::from_hours(365 * 24))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let claims = JwtClaims {
        sub: account.id.clone(),
        exp: expiration_date as usize,
    };
    encode(&Header::default(), &claims, &key)
}

/// Returns the User ID on success.
pub fn verify_jwt(encoded_jwt: &str, secret_key: &[u8]) -> jsonwebtoken::errors::Result<String> {
    let key = DecodingKey::from_secret(secret_key);
    let claims: JwtClaims = decode(encoded_jwt.as_bytes(), &key, &Validation::default())?.claims;
    Ok(claims.sub)
}

fn argon2_instance<'a>() -> Argon2<'a> {
    Argon2::default()
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    argon2_instance()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(password_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    argon2_instance()
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok()
}

/// Generate HMAC of accountname. Usernames are not stored in plaintext for better anonymity.
pub fn hash_accountname(accountname: &str, secret_key: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(accountname.as_bytes()).unwrap();
    mac.update(secret_key);

    let result = &mac.finalize().into_bytes();
    bytes_to_hex_string(result)
}
