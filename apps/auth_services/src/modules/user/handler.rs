use actix_web::{get, post, web::{scope, Data, Json, ServiceConfig}, HttpMessage, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use serde::Serialize;
use serde_json::json;
use validator::Validate;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, time::Instant};
use jwt_libs::types::AccessToken;
use crate::{middlewares::{access_token_middleware::AccessTokenMW, refresh_token_middleware::RefreshTokenMW},AppState};

use logger_libs::{debug_logger, info_logger, warning_logger};
use super::{model::{LoginData, RegisterData}, service::UserServices};

// Validation function
fn json_validate<T:Validate>(
    json_data: Json<T>,
    log_id: &str,
    handler:&str
) -> Result<T, HttpResponse>
where 
    T: Clone + Serialize + Debug
{
    let data = json_data.into_inner();
    let method_name = "Validate";
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

        warning_logger(&log_id, handler,method_name, &format!("{:?}",error_map));
        return Err(HttpResponse::BadRequest().json(json!({
            "status": "failed",
            "message": error_map
        })));
    }

    info_logger(&log_id, handler, method_name);
    Ok(data)
}

#[post("/register")]
async fn register_handlers(
    register_body: Json<RegisterData>,
    app_data: Data<AppState>
) -> impl Responder {
    let start = Instant::now();
    info!("Registration process started at: {:?}", start);
    
    let log_id = format!("{}-{}",uuid::Uuid::new_v4(),register_body.username);
    // Validate incoming JSON data
    let req_log = register_body.clone();
    let register_data = match json_validate(
        register_body,
        &log_id,
        "register"
    ) {
        Ok(validated_data) => validated_data,
        Err(err_response) => return err_response,
    };

    // Attempt to register user
    match UserServices::register(
        &log_id,
        register_data,
        &app_data.db,
        &app_data.rabbit).await {
        Ok(user_payload) => {
            debug_logger(&log_id, "register", "Service", &req_log, &user_payload);
            
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

#[get("/refresh_token")]
async fn refresh_token_handler(
    req:HttpRequest,
    app_state:Data<AppState>,
) -> impl Responder{
    let start = Instant::now();
    let token = req.extensions().get::<String>().cloned().expect("token not found");

    match UserServices::refresh_token(token, &app_state.db,&app_state.redis).await{
        Ok(access_token)=>{
            info!("proccess token: {}",access_token);

            let end = Instant::now();
            info!("proccess refresh_token success, request time: {:?}",end - start);
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"get token success",
                "data":{
                    "access_token":format!("{}",access_token)
                }
            }))
        },
        Err(error)=>{
            info!("proccess token failed: {}",error);
            HttpResponse::BadGateway().json(json!({
                "status":"failed",
                "message":format!("server error: {}",error)
            }))
        }
    }
}

#[get("/user_profile")]
async fn user_profile_handler(
    req: HttpRequest,
    app_state: Data<AppState>
)-> impl Responder{
    let start = Instant::now();
    let token = req.extensions().get::<AccessToken>().cloned().expect("token not found");

    info!("find user proccess, token: {:?}",token);
    println!("token :{:?}",token);
    match UserServices::access_token(token, &app_state.db).await{
        Ok(user)=>{
            info!("find user, data:{:?}",user);
            let end = Instant::now();

            info!("get user success,response time: {:?}",end - start);
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"get user profile success",
                "data":user
            }))
        },
        Err(error)=>{
            HttpResponse::BadGateway().json(json!({
                "status":"failed",
                "message": format!("server error: {}",error)
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
pub fn token_config(config:&mut ServiceConfig){
    config.service(
        scope("/token")
        .wrap(RefreshTokenMW)
        .service(refresh_token_handler)
    );
}

pub fn user_config(config:&mut ServiceConfig){
    config.service(
        scope("/user")
        .wrap(AccessTokenMW)
        .service(user_profile_handler)
    );
}