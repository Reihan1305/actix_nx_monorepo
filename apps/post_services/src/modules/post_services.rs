use sqlx::types::Uuid;
use tonic::{async_trait, Request, Response, Status};
use pgsql_libs::DbPool;

use crate::proto::{post_server::Post, GetPostByIdRequest, PostRequest, PostResponse};

use super::{post_models::CreatePost, post_query::PostQuery};

#[derive(Debug)]
pub struct PostService{
    dbpool: DbPool
}

impl PostService{
    pub fn new(dbpool: DbPool) -> Self {
        Self { dbpool }
    }
}

#[async_trait]
impl Post for PostService{
    async fn create_post(
        &self,
        request: Request<PostRequest>
    )->Result<Response<PostResponse>,Status>{

        let input = request.get_ref();

        let data: CreatePost = CreatePost{
            user_id:input.user_id.parse::<Uuid>().unwrap(),
            content:input.content.clone(),
            title:input.title.clone(),
            username:input.content.clone()
        };

        let new_post = match PostQuery::create_post(
            data,
            &self.dbpool
        ).await{
            Ok(posts)=> posts,
            Err(error)=>{
                return Err(Status::internal(error))
            }
        };

        let response = PostResponse{
            post_id: new_post.id.to_string(),
            user_id: new_post.user_id.to_string(),
            username:new_post.username,
            title:new_post.title,
            content:new_post.content
        }; 

        Ok(Response::new(response))
    }

    async fn get_post_by_id(
        &self,
        request: Request<GetPostByIdRequest>
    )-> Result<Response<PostResponse>,Status>{
        let data: &GetPostByIdRequest = request.get_ref();
        
        let post = match PostQuery::get_post_by_id(data.post_id.parse::<Uuid>().expect("invalid id"), &self.dbpool).await{
            Ok(post)=>post,
            Err(error)=> return Err(Status::internal(error))
        };

        let response = PostResponse{
            post_id: post.id.to_string(),
            user_id: post.user_id.to_string(),
            username:post.username,
            title:post.title,
            content:post.content
        };

        Ok(Response::new(response))
    }
}
