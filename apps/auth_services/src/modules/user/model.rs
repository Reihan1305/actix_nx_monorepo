use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug,Serialize, Deserialize, Validate,Clone)]
pub struct RegisterData{
    #[validate(email(message="invalid format"))]
    pub email: String,
    #[validate(length(min=5, message="too short"))]
    pub username: String,
    #[validate(length(min=5, message="too short"))]
    pub password: String
}

#[derive(Deserialize,Serialize,Debug)]
pub struct RegisterPayload{
    pub id:Uuid,
    pub email: String,
    pub username: String,
}

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct LoginData{
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String
}

#[derive(Debug,Deserialize,Serialize)]
pub struct LoginQueryPayload{
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String
}

#[derive(Debug,Deserialize,Serialize)]
pub struct LoginPayload{
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub refresh_token: String,
    pub access_token: String
}