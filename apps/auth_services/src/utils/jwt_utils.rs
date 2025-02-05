use chrono::{Utc, Duration};
use jsonwebtoken::{decode, encode, errors::Error as JwtError, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use super::types_utils::{AccessToken, RefreshToken, TokenClaims};


impl<T: Serialize> TokenClaims<T> {
    pub fn generate_token(data: T, duration: Duration) -> Result<String, String> {
        let iat = Utc::now().timestamp();
        let exp = (Utc::now() + duration).timestamp();

        let claims = TokenClaims { iat, exp, token: data };
        let jwt_secret = EncodingKey::from_secret("secret_key".as_bytes());
        encode(&Header::default(), &claims , &jwt_secret).map_err(|e| e.to_string())
    }
}

pub fn decode_token<T>(token: &str) -> Result<TokenData<TokenClaims<T>>, String>
where
    T: for<'de> Deserialize <'de>,
{
    decode::<TokenClaims<T>>(
        &token,
        &DecodingKey::from_secret("secret_key".as_bytes()),
        &Validation::default(),
    )
    .map_err(|e: JwtError| e.to_string())
}

// Usage examples:

pub fn generate_refresh_token(data: RefreshToken) -> Result<String, String> {
    TokenClaims::<RefreshToken>::generate_token(data, Duration::days(7))
}

pub fn generate_access_token(data: AccessToken) -> Result<String, String> {
    TokenClaims::<AccessToken>::generate_token(data, Duration::minutes(20))
}

pub fn decode_refresh_token(token: &str) -> Result<TokenData<TokenClaims<RefreshToken>>, String> {
    decode_token(token)
}

pub fn decode_access_token(token: &str) -> Result<TokenData<TokenClaims<AccessToken>>, String> {
    decode_token(token)
}
