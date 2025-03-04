use serde::{Deserialize,Serialize};
use uuid::Uuid;


#[derive(Deserialize,Serialize, Debug)]
pub struct CreatePostRequest{
    pub title: String,
    pub content: String,
}

#[derive(Deserialize,Serialize, Debug)]
pub struct PostResponse{
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub user_id: Uuid,
    pub username: String
}

#[derive(Deserialize)]
pub struct Pagination {
    pub limits: Option<usize>,
    pub page: Option<usize>,
}