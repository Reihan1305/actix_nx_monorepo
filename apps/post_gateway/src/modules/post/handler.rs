use actix_web::{
    delete, get, patch, post, web::{scope, Data, Json, Path, Query, ServiceConfig}, HttpResponse, Responder
};
use kafka_libs::{send_message, Producer};
use logger_libs::Logger;
use serde_json::json;
use uuid::Uuid;
use proto_libs::post_proto;

use crate::{
    modules::post::model::{CreatePostRequest, Pagination, PostResponse}, AppState
};

pub async fn send_event(
    producer: &Producer,
    post_id: Uuid,
    user_id: Uuid,
    message: String,
) -> Result<(), HttpResponse>
{
    let key = format!("{}:{}",user_id,post_id);
    let topic = "post";

    let producer_guard = producer.lock().await;
    let producer_ref = &*producer_guard;

    match send_message(producer_ref, topic, &key, &message).await {
        Ok(_) => {
            Logger::info_logger("post_gateway.send_handler", &format!("{}",key), "send_event.send_message");
            Ok(())
        },
        Err(error) => {
            Logger::err_logger("post_gateway.send_handler", &format!("{}",key), "send_event.send_message",error);
            Err(HttpResponse::InternalServerError().json("Failed to send message to Kafka"))
        },
    }
}

#[post("/create_post")]
pub async fn create_post(
    data: Data<AppState>,
    content: Json<CreatePostRequest>
) -> impl Responder {
    let handler_name = "post_gateway.create_post";
    let req = &content.into_inner();
    let request_data = post_proto::CreatePostRequest{
            title: req.title.clone(),
            content: req.content.clone(),  
    };

    let log_id= &format!("{}.{}",handler_name,&request_data.title);

    let request = tonic::Request::new(request_data);
    
    let response = {
        let mut client = data.protected_post_client.lock().await;
        client.create_post(request).await
    };

    match response {
        Ok(message) => {
            let message = &message.into_inner();
            let response = PostResponse{
                id: message.id.parse::<Uuid>().unwrap(),
                title: message.title.clone(),
                content: message.content.clone(),
                user_id:message.user_id.parse::<Uuid>().unwrap(),
                username:message.username.clone()
            };
            let kafka_message = String::from("post created");
            let _ = send_event(&data.kafka_producer,response.id , response.user_id, kafka_message).await;
            Logger::info_logger(&handler_name, log_id,"post_gateway.create_post.insert_services");
        
            Logger::debug_logger(&handler_name, log_id, req, "post_gateway.create_post.insert_services", &response);
            HttpResponse::Created().json(json!({
                "message": "post created",
                "data": response
            }))
        },
        Err(error) => {
            Logger::err_logger(&handler_name, log_id, "post_gateway.create_post.insert_services", &error    );
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
    let handler_name = "post_gateway.update_post";
    let log_id = &format!("post_gateway.update_post.{}",post_id);

    let req = &content.into_inner();
    let request_data = post_proto::UpdatePostRequest{
            post_id: post_id.to_string(),
            title: req.title.clone(),
            content: req.content.clone(),  
    };

    let request = tonic::Request::new(request_data);

    let response = {
        let mut client = data.protected_post_client.lock().await;
        client.update_post(request).await
    };

    match response {
        Ok(message) => {
            let message = message.into_inner();
            let response = PostResponse{
                id: message.id.parse::<Uuid>().unwrap(),
                title: message.title,
                content: message.content,
                user_id:message.user_id.parse::<Uuid>().unwrap(),
                username:message.username
            };

            Logger::debug_logger(&handler_name, log_id, req, "post_gateway.update_in_services", &response);
            Logger::info_logger(&handler_name, log_id, "post_gateway.update_in_services");

            let kafka_message = String::from("post updated");
            let _ = send_event(&data.kafka_producer,response.id , response.user_id, kafka_message).await;
    
            HttpResponse::Created().json(json!({
                "message": "post created",
                "data": response
            }))
        },
        Err(error) => {
            Logger::err_logger(&handler_name, log_id, "post_gateway.update_in_services", &error);
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
    let handler_name = "post_gateway.delete_post";
    let log_id = &format!("post_gateway.delete_post.{}",post_id);

    let request_data = post_proto::PostIdRequest{
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
            let response_message = format!("post: {} successfully deleted",message.post_id);
            Logger::debug_logger(&handler_name, log_id, &post_id, "post_gateway.delete_post_in_services", &response_message);

            Logger::info_logger(&handler_name, log_id, "post_gateway.delete_post_in_services");
            let kafka_message = String::from("post created");
            let _ = send_event(&data.kafka_producer,message.post_id.parse::<Uuid>().unwrap() , message.user_id.parse::<Uuid>().unwrap(), kafka_message).await;

            HttpResponse::Created().json(json!({
                "message": response_message
            }))
        },
        Err(error) => {
            Logger::err_logger(&handler_name, log_id, "post_gateway.delete_post_in_services",&error);
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




#[get("/get_all_post")]
pub async fn get_all_post(
    data: Data<AppState>,
    query: Query<Pagination> 
) -> impl Responder {
    let limits = query.limits.unwrap_or(10); 
    let page = query.page.unwrap_or(1); 

    let request_data = post_proto::GetAllPostRequest { limits: limits as i64, page: page as i64 };

    let handler_name = "post_gateway.get_all_post";
    let log_id = &format!("post_gateway.get_all_post.{}",Uuid::new_v4());
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
            
            Logger::info_logger(&handler_name, log_id, "post_gateway.get_all_post_in_post_services");
            HttpResponse::Ok().json(json!({
                "message": "fetch all posts success",
                "limits": limits,
                "page": page,
                "data": posts
            }))
        },
        Err(error) => {
            Logger::warning_logger(&handler_name, log_id, "post_gateway.get_all_post_in_post_services",&format!("{}",error));
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
    let handler_name = "post_gateway.get_post_by_id";
    let log_id = &format!("post_gateway.get_post_by_id.{}",post_id);

    let request_data = post_proto::PostIdRequest {
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

            Logger::info_logger(&handler_name, log_id, "get_post_by_id_in_post_services");
            HttpResponse::Ok().json(json!({
                "message": "fetch post success",
                "data": post
            }))
        },
        Err(error) => {
            Logger::warning_logger(&handler_name, log_id, "get_post_by_id_in_post_services",&format!("{}",error));

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