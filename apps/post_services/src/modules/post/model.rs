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

pub struct UpdatePost{
    pub post_id: Uuid,     
    pub user_id: Uuid,
    pub title: String,
    pub content : String,
}

pub struct UserPayload {
    pub id: Uuid,
    pub username: String,
    pub email: String
}

pub struct QueryDeleteResponse{
    pub post_id: Uuid,
    pub user_id: Uuid
}