use actix_web::{
    delete, get, patch, post, web::{scope, Data, Json, Path, Query, ServiceConfig}, HttpResponse, Responder
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    modules::post_model::{CreatePostRequest, PostResponse}, proto, AppState
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

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    limits: Option<usize>,
    page: Option<usize>,
}

#[get("/get_all_post")]
pub async fn get_all_post(
    data: Data<AppState>,
    query: Query<Pagination> 
) -> impl Responder {
    let limits = query.limits.unwrap_or(10); 
    let page = query.page.unwrap_or(1); 

    println!("Fetching posts with limits: {}, page: {}", limits, page);

    // Simulasi query ke database atau panggilan ke gRPC
    let request_data = crate::proto::GetAllPostRequest { limits: limits as i64, page: page as i64 };


    let response = {
        let mut client = data.post_client.lock().await;
        client.get_all_post(request_data).await
    };

    match response {
        Ok(response) => {
            let message = response.into_inner();
            let posts: Vec<PostResponse> = message.posts.into_iter()
            .map(|post| PostResponse {
                id: post.id.parse::<Uuid>().unwrap(),
                user_id: post.user_id.parse::<Uuid>().unwrap(),
                title: post.title,
                content: post.content,
                username:post.username
            })
            .collect();
        
            HttpResponse::Ok().json(json!({
                "message": "fetch all posts success",
                "limits": limits,
                "page": page,
                "data": posts
            }))
        },
        Err(error) => {
            HttpResponse::BadRequest().json(json!({
                "message": "fetch all posts failed",
                "error": format!("{}", error)
            }))
        }
    }
}

#[get("/get_post/{post_id}")]
pub async fn get_post_by_id(
    data: Data<AppState>,
    path: Path<Uuid>,  
) -> impl Responder {
    let post_id = path.into_inner(); 

    println!("Fetching post with ID: {}", post_id);

    // Membuat request ke gRPC
    let request_data = crate::proto::PostIdRequest {
        post_id: post_id.to_string(),
    };

    let response = {
        let mut client = data.post_client.lock().await;
        client.get_post_by_id(request_data).await
    };

    match response {
        Ok(response) => {
            let message = response.into_inner();
            let post = PostResponse {
                id: message.id.parse::<Uuid>().unwrap(),
                user_id: message.user_id.parse::<Uuid>().unwrap(),
                title: message.title,
                content: message.content,
                username: message.username
            };

            HttpResponse::Ok().json(json!({
                "message": "fetch post success",
                "data": post
            }))
        },
        Err(error) => {
            HttpResponse::NotFound().json(json!({
                "message": "post not found",
                "error": format!("{}", error)
            }))
        }
    }
}


pub fn post_config(config: &mut ServiceConfig){
    config.service(
        scope("/post")
        .service(get_all_post)
        .service(get_post_by_id)
    );
}