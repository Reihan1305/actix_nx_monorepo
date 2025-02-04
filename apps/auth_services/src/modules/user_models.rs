use serde::{Serialize,Deserialize};
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
    pub id:String,
    pub email: String,
    pub username: String,
}