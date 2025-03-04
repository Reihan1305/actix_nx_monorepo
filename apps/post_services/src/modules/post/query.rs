use pgsql_libs::DbPool;
use sqlx::{query, query_as, types::Uuid};
use tonic::Status;

use proto_libs::post_proto::PostResponse;

use super::model::{CreatePost, PostPayload, QueryDeleteResponse, UpdatePost, UserPayload};
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
    )->Result<Option<PostPayload>,String>{
        let new_post = query_as!(
            PostPayload,  
            r#"
            SELECT username, user_id, title, content, id FROM "posts" 
            where id = $1
            "#,
            post_id,
        )
        .fetch_optional(db_pool)  
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
    ) -> Result<PostPayload, Status> {
        // Find the user by ID
        let user: UserPayload = match Self::find_user_by_id(update_data.user_id, db_pool).await {
            Ok(user) => user,
            Err(error) => return Err(Status::invalid_argument(format!("user not found: {}",error))),
        };
    
        // Find the post by ID
        let post = match Self::get_post_by_id(update_data.post_id, db_pool).await {
            Ok(Some(post)) => post,
            Ok(None)=> return Err(Status::invalid_argument(format!("post not found: "))),
            Err(error) => return Err(Status::internal(format!("error database: {}",error))),
        };
        
        if user.id != post.user_id {
            return Err(Status::unauthenticated("you are not the owner"))
        }

        // Update the post in the database
        let updated_post = query_as!(
            PostPayload,
            r#"
            UPDATE posts
            SET title = $1, content = $2
            WHERE user_id = $3 AND id = $4
            RETURNING id, username, user_id, title, content
            "#,
            update_data.title,
            update_data.content,
            user.id,
            post.id
        )
        .fetch_one(db_pool)
        .await;
    
        match updated_post {
            Ok(post) => Ok(post),
            Err(error) => Err(Status::aborted(format!("{}",error))),
        }
    }

    pub async fn delete_post(
        user_id: Uuid,
        post_id: Uuid,
        db_pool: &DbPool
    ) -> Result<QueryDeleteResponse, String> {

        let post = query!(
            r#"
            SELECT id, user_id FROM posts WHERE id = $1;
            "#,
            post_id
        )
        .fetch_optional(db_pool)
        .await;
    
        let post = match post {
            Ok(Some(post)) => post,
            Ok(None) => return Err("Post not found".to_string()),
            Err(error) => return Err(format!("Database error: {}", error)),
        };
    
        if post.user_id != user_id {
            return Err("You do not have permission to delete this post".to_string());
        }
    
        let result = query!(
            r#"
            DELETE FROM posts WHERE id = $1;
            "#,
            post_id
        )
        .execute(db_pool)
        .await;
    
        match result {
            Ok(_) => Ok(
                QueryDeleteResponse{
                    post_id,
                    user_id
                }
            ),
            Err(error) => Err(format!("Failed to delete post: {}", error)),
        }
    }
    
}