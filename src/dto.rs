use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RegisterUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub jwt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct DeleteUser {
    pub password: String,
}

/// Claims to store inside the JWT Token
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User ID.
    pub sub: String,
    pub exp: usize,
}
