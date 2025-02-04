use actix_web::{post, web::{scope, Data, Json, ServiceConfig}, HttpResponse, Responder};
use log::{error, info};
use serde_json::json;
use validator::Validate;
use std::{borrow::Cow, collections::HashMap, time::Instant};

use crate::AppState;

use super::{user_models::{LoginData, RegisterData}, user_services::UserServices};

// Validation function
fn json_validate<T:Validate>(data: Json<T>) -> Result<T, HttpResponse> {
    let data = data.into_inner();
    
    let mut error_map: HashMap<String,Cow<'static,str>> = HashMap::new();

    if let Err(errors) = data.validate() {
        for (field, error) in errors.field_errors() {
            let error_messages = error.into_iter().map(|e| {
                e.message.clone()
            });
            
            for error_message in error_messages {
                    error_map.insert(field.to_string(), error_message.clone().unwrap());
            }
        }
        return Err(HttpResponse::BadRequest().json(json!({
            "status": "failed",
            "message": error_map
        })));
    }
    Ok(data)
}

#[post("/register")]
async fn register_handlers(
    register_body: Json<RegisterData>,
    app_data: Data<AppState>
) -> impl Responder {
    let start = Instant::now();
    info!("Registration process started: {:?}", start);

    // Validate incoming JSON data
    let register_data = match json_validate(register_body) {
        Ok(validated_data) => validated_data,
        Err(err_response) => return err_response,
    };

    // Attempt to register user
    match UserServices::register(register_data, &app_data.db).await {
        Ok(user_payload) => {
            info!("User registered successfully with ID: {}", user_payload.id);
            let end:Instant = Instant::now();
            info!("success response take: {:?}",end - start);
            HttpResponse::Created().json(json!({
                "status": "success",
                "message": "Registration successful",
                "data": user_payload
            }))
        },
        Err(error) => {
            log::error!("Registration failed: {}", error);
            HttpResponse::BadRequest().json(json!({
                "status": "failed",
                "message": format!("Server error: {}", error)
            }))
        }
    }
}

#[post("/login")]
async fn login_handlers(
    login_body: Json<LoginData>,
    app_data: Data<AppState>
) -> impl Responder{
    let start = Instant::now();

    info!("login proccess started: {:?}",start);
    
    let login_data = login_body.into_inner();

    match UserServices::login(login_data, &app_data.db).await{
        Ok(payload)=>{
            let end = Instant::now();
            info!("login success, response time: {:?}", end - start);
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"login successfull",
                "data":payload
            }))
        },
        Err(errors)=>{
            let end = Instant::now();
            error!("server error: {}, response time: {:?}",errors, end - start);
            HttpResponse::BadGateway().json(json!({
                "status":"failed",
                "message":format!("server Error: {}",errors)
            }))
        }
    }
}

pub fn auth_config(config:&mut ServiceConfig){
    config.service(
        scope("/auth")
        .service(register_handlers)
        .service(login_handlers)
    );
}