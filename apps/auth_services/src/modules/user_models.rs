use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct RegisterData{
    #[validate(email(message="invalid format"))]
    pub email: String,
    #[validate(length(min=5, message="too short"))]
    pub username: String,
    #[validate(length(min=5, message="too short"))]
    pub password: String
}

#[derive(Deserialize,Serialize)]
pub struct RegisterPayload{
    pub id:Uuid,
    pub email: String,
    pub username: String,
}

#[derive(Deserialize,Serialize)]
pub struct LoginData{
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String
}

#[derive(Deserialize,Serialize)]
pub struct LoginQueryPayload{
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String
}

#[derive(Deserialize,Serialize)]
pub struct LoginPayload{
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub refresh_token: String,
    pub access_token: String
}