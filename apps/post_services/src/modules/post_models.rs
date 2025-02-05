use sqlx::types::Uuid;

pub struct CreatePost{
    pub username: String,
    pub user_id: Uuid,
    pub title: String,
    pub content: String
}

#[derive(sqlx::FromRow)]
pub struct  PostPayload{
    pub username: String,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub id: Uuid
}