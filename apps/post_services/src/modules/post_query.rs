use pgsql_libs::DbPool;
use sqlx::{query_as, types::Uuid};
use crate::proto::PostResponse;

use super::post_models::{CreatePost, UpdatePost, UserPayload};
pub struct PostQuery;

impl PostQuery{
    pub async fn create_post(
        data: CreatePost,
        db_pool: &DbPool
    )->Result<PostResponse,String>{
        let new_post = query_as!(
            PostResponse,  
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
    )->Result<PostResponse,String>{
        let new_post = query_as!(
            PostResponse,  
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
    pub async fn get_all_posts(
        db_pool: &DbPool,
        page: i64,
        limit: i64
    ) -> Result<Vec<PostResponse>, String> {
        let offset = (page - 1) * limit;
    
        let posts = query_as!(
            PostResponse,
            r#"
            SELECT username, user_id, title, content, id 
            FROM "posts"
            LIMIT $1 OFFSET $2;
            "#,
            limit,
            offset
        )
        .fetch_all(db_pool)
        .await;
    
        match posts {
            Ok(post_list) => Ok(post_list),
            Err(error) => Err(format!("Error database: {}", error)),
        }
    }
        pub async fn find_user_by_id(
        id: Uuid,
        db_pool: &DbPool
    )-> Result<UserPayload,String>{
        match query_as!(
            UserPayload,
            r#"
            SELECT id, username, email FROM "user" where id = $1;
            "#,
            id
        ).fetch_one(db_pool).await{
            Ok(user)=>Ok(user),
            Err(error)=> Err(format!("db error: {}", error))
        }
    }    

    pub async fn update_post(
        update_data: UpdatePost,
        db_pool: &DbPool,
    ) -> Result<PostResponse, String> {
        // Find the user by ID
        let user: UserPayload = match Self::find_user_by_id(update_data.user_id, db_pool).await {
            Ok(user) => user,
            Err(error) => return Err(error),
        };
    
        // Find the post by ID
        let post = match Self::get_post_by_id(update_data.post_id, db_pool).await {
            Ok(post) => post,
            Err(error) => return Err(error),
        };
    
        // Update the post in the database
        let updated_post = query_as!(
            PostResponse,
            r#"
            UPDATE posts
            SET title = $1, content = $2
            WHERE user_id = $3 AND id = $4
            RETURNING id, username, user_id, title, content
            "#,
            update_data.title,
            update_data.content,
            user.id,
            post.id.parse::<Uuid>().unwrap()
        )
        .fetch_one(db_pool)
        .await;
    
        match updated_post {
            Ok(post) => Ok(post),
            Err(error) => Err(error.to_string()),
        }
    }
}