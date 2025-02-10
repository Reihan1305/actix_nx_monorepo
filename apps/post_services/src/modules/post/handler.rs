use std::sync::Arc;

use jwt_libs::types::AccessToken;
use sqlx::types::Uuid;
use tonic::{async_trait, Request, Response, Status};
use pgsql_libs::DbPool;
use super::{model::{CreatePost, UpdatePost}, query::PostQuery};
use proto_libs::post_proto::{
    post_server::Post, protected_post_server::ProtectedPost, CreatePostRequest, DeleteResponse, GetAllPostRequest, PostIdRequest, PostListResponse, PostResponse, UpdatePostRequest
};

pub struct PostService{
    dbpool: DbPool
}

pub struct AuthPostService{
    dbpool: DbPool
}

impl PostService{
    pub fn new(dbpool: DbPool) -> Self {
        Self { dbpool }
    }
}

#[async_trait]
impl Post for PostService{
    async fn get_all_post(
        &self,
        request: Request<GetAllPostRequest>
    )-> Result<Response<PostListResponse>,Status>{
        let data: &GetAllPostRequest = request.get_ref();

        let posts = match PostQuery::get_all_posts(&self.dbpool, data.page, data.limits).await{
            Ok(posts)=>posts,
            Err(error)=> return Err(Status::internal(error))
        };


        let response: PostListResponse = PostListResponse{
            posts
        };

        Ok(Response::new(response))
    }   


    async fn get_post_by_id(
        &self,
        request: Request<PostIdRequest>
    )-> Result<Response<PostResponse>,Status>{
        let data: &PostIdRequest = request.get_ref();
        
        let post = match PostQuery::get_post_by_id(data.post_id.parse::<Uuid>().expect("invalid id"), &self.dbpool).await{
            Ok(post)=>post,
            Err(error)=> return Err(Status::internal(error))
        };

        let response = PostResponse{
            id: post.id.to_string(),
            user_id: post.user_id.to_string(),
            username:post.username,
            title:post.title,
            content:post.content
        };

        Ok(Response::new(response))
    }
}

impl AuthPostService{
    pub fn new(dbpool: DbPool)-> Self {
        Self { dbpool }
    }

    pub fn user_validate<T>(
        req: &Request<T>
    )->Result<AccessToken,Status>{
        match req.extensions().get::<Arc<AccessToken>>(){
            Some(token)=>{
                Ok(token.as_ref().clone())
            },
            None=>{
                return Err(Status::unauthenticated(format!("invalid token")))
            }
        }
    }
}

#[async_trait]
impl ProtectedPost for AuthPostService{
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>
    )->Result<Response<PostResponse>,Status>{
        let user = self::AuthPostService::user_validate(&request)?;

        let input = request.get_ref();

        let data: CreatePost = CreatePost{
            user_id:user.id,
            content:input.content.clone(),
            title:input.title.clone(),
            username:user.username
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
            id: new_post.id.to_string(),
            user_id: new_post.user_id.to_string(),
            username:new_post.username,
            title:new_post.title,
            content:new_post.content
        }; 

        Ok(Response::new(response))
    }

    async fn update_post(
        &self,
        request: tonic::Request<UpdatePostRequest>
    )-> Result<Response<PostResponse>,Status>{
        let user =  self::AuthPostService::user_validate(&request)?;
        let data: &UpdatePostRequest= request.get_ref();

        let update_data: UpdatePost = UpdatePost{
            post_id: data.post_id.parse::<Uuid>().unwrap(),
            user_id: user.id,
            content: data.content.clone(),
            title: data.title.clone(),
        };
        match PostQuery::update_post(update_data, &self.dbpool).await{
            Ok(posts)=>{
                let response:PostResponse = PostResponse{
                    content:posts.content,
                    id:posts.id.to_string(),
                    title:posts.title,
                    user_id:posts.user_id.to_string(),
                    username:posts.username
                };

                Ok(Response::new(response))
            },
            Err(error)=>{
                Err(error)
            }
        }
    }


    async fn delete_post(
        &self,
        request:Request<PostIdRequest>
    )-> Result<Response<DeleteResponse>,Status>{
        let user =  self::AuthPostService::user_validate(&request)?;
        let data = request.get_ref();
        

        match PostQuery::delete_post(user.id, data.post_id.parse::<Uuid>().unwrap(), &self.dbpool).await{
            Ok(delete_response)=>{
                let response: DeleteResponse = DeleteResponse{
                    post_id: String::from(delete_response.post_id),
                    user_id: String::from(delete_response.user_id),
                    message: String::from("delete post successfully")
                };
                Ok(Response::new(response))
            },
            Err(error)=>Err(Status::internal(error))   
        }
    }
}