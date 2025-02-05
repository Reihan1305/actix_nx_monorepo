use std::error::Error;

use proto::post_server::{Post,PostServer};
use tonic::{async_trait, transport::Server, Request, Response, Status};

pub mod proto {
    tonic::include_proto!("post");
}

#[derive(Debug,Default)]
pub struct PostService;

#[async_trait]
impl Post for PostService{
    async fn create_post(
        &self,
        request: Request<proto::PostRequest>
    )->Result<Response<proto::PostResponse>,Status>{
        println!("request: {:?}",request);
        
        let input = request.get_ref();

        let response = proto::PostResponse{
            user_id: input.user_id.clone(),
            username: input.username.clone(),
            title: input.title.clone(),
            content: input.content.clone()        
        };

        Ok(Response::new(response))
    }
}


#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>>{
    let address = "[::1]:50501".parse()?;

    let post = PostService::default();

    Server::builder()
    .add_service(PostServer::new(post))
    .serve(address)
    .await?;

    Ok(())
}