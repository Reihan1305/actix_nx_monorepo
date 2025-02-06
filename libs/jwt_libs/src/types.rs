use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct TokenClaims<T> {
    pub iat: i64,
    pub exp: i64,
    pub token: T,
}

#[derive(Deserialize, Serialize,Clone,Debug)]
pub struct AccessToken {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct RefreshToken {
    pub id: Uuid,
}
