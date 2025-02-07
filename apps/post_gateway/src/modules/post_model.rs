use serde::{Deserialize,Serialize};
use uuid::Uuid;


#[derive(Deserialize,Serialize)]
pub struct CreatePostRequest{
    pub title: String,
    pub content: String,
}

#[derive(Deserialize,Serialize)]
pub struct PostResponse{
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub user_id: Uuid,
    pub username: String
}

// #[derive(Deserialize,Serialize)]
// pub 