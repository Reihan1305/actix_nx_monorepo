use actix_web::{
    delete, patch, post, web::{scope, Data, Json, Path, ServiceConfig}, HttpResponse, Responder
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    modules::post_model::{CreatePostRequest, PostResponse}, proto::{self, post_client::PostClient, protected_post_client::ProtectedPostClient}, AppState
};

#[post("/create_post")]
pub async fn create_post(
    data: Data<AppState>,
    content: Json<CreatePostRequest>
) -> impl Responder {
    let req = content.into_inner();
    let request_data = proto::CreatePostRequest{
            title: req.title,
            content: req.content,  
    };

    let request = tonic::Request::new(request_data);

    let response = {
        let mut client = data.protected_post_client.lock().await;
        client.create_post(request).await
    };

    match response {
        Ok(message) => {
            let message = message.into_inner();
            println!("ini message: {:?}",message);
            let response = PostResponse{
                id: message.id.parse::<Uuid>().unwrap(),
                title: message.title,
                content: message.content,
                user_id:message.user_id.parse::<Uuid>().unwrap(),
                username:message.username
            };
    
            HttpResponse::Created().json(json!({
                "message": "post created",
                "data": response
            }))
        },
        Err(error) => {
            HttpResponse::BadRequest().json(json!({
                "message": "create post failed",
                "error": format!("{}", error)
            }))
        }
    }
    
}

#[patch("/update_post/{post_id}")]
pub async fn update_post(
    data: Data<AppState>,
    path: Path<Uuid>,
    content: Json<CreatePostRequest>
) -> impl Responder {
    let post_id = path.into_inner();

    let req = content.into_inner();
    let request_data = proto::UpdatePostRequest{
            post_id: post_id.to_string(),
            title: req.title,
            content: req.content,  
    };

    let request = tonic::Request::new(request_data);

    let response = {
        let mut client = data.protected_post_client.lock().await;
        client.update_post(request).await
    };

    match response {
        Ok(message) => {
            let message = message.into_inner();
            println!("ini message: {:?}",message);
            let response = PostResponse{
                id: message.id.parse::<Uuid>().unwrap(),
                title: message.title,
                content: message.content,
                user_id:message.user_id.parse::<Uuid>().unwrap(),
                username:message.username
            };
    
            HttpResponse::Created().json(json!({
                "message": "post created",
                "data": response
            }))
        },
        Err(error) => {
            HttpResponse::BadRequest().json(json!({
                "message": "create post failed",
                "error": format!("{}", error)
            }))
        }
    }
    
}

#[delete("/delete_post/{post_id}")]
pub async fn delete_post(
    data: Data<AppState>,
    path: Path<Uuid>
) -> impl Responder {
    let post_id: Uuid = path.into_inner();

    let request_data = proto::PostIdRequest{
            post_id: post_id.to_string()
    };

    let request = tonic::Request::new(request_data);

    let response = {
        let mut client = data.protected_post_client.lock().await;
        client.delete_post(request).await
    };

    match response {
        Ok(message) => {
            let message = message.into_inner();
            println!("ini message: {:?}",message);
            let response_message = format!("post: {} successfully deleted",message.post_id);
    
            HttpResponse::Created().json(json!({
                "message": response_message
            }))
        },
        Err(error) => {
            HttpResponse::BadRequest().json(json!({
                "message": "create post failed",
                "error": format!("{}", error)
            }))
        }
    }
    
}


pub fn protected_post_config(config: &mut ServiceConfig) {
    config.service(
        scope("/protected_post")
            .service(create_post)
            .service(update_post)
            .service(delete_post)
    );
}
