use pgsql_libs::DbPool;
use sqlx::{query_as, types::Uuid};

use super::post_models::{CreatePost, PostPayload};
pub struct PostQuery;

impl PostQuery{
    pub async fn create_post(
        data: CreatePost,
        db_pool: &DbPool
    )->Result<PostPayload,String>{
        let new_post = query_as!(
            PostPayload,  
            r#"
            INSERT INTO "posts" (username, user_id, title, content)
            VALUES ($1, $2, $3, $4)
            RETURNING username, user_id, title, content, id
            "#,
            data.username,
            data.user_id,
            data.title,
            data.content
        )
        .fetch_one(db_pool)  
        .await;
        
        match new_post {
            Ok(post) => Ok(post),
            Err(error) => Err(format!("Error database: {}", error)),
        }
        
    }

    pub async fn get_post_by_id(
        post_id:Uuid,
        db_pool: &DbPool
    )->Result<PostPayload,String>{
        let new_post = query_as!(
            PostPayload,  
            r#"
            SELECT username, user_id, title, content, id FROM "posts" 
            where id = $1
            "#,
            post_id,
        )
        .fetch_one(db_pool)  
        .await;
        
        match new_post {
            Ok(post) => Ok(post),
            Err(error) => Err(format!("Error database: {}", error)),
        }
        
    }
}