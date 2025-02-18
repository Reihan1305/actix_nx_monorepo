use actix_web::{get, post, web::{scope, Data, Json, ServiceConfig}, HttpMessage, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use serde::Serialize;
use serde_json::json;
use validator::Validate;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, time::Instant};
use jwt_libs::types::AccessToken;
use crate::{middlewares::{access_token_middleware::AccessTokenMW, refresh_token_middleware::RefreshTokenMW},AppState};

use logger_libs::{debug_logger, error_logger, info_logger, warning_logger};
use super::{model::{LoginData, RegisterData}, service::UserServices};

// Validation function
fn json_validate<T>(
    json_data: Json<T>
) -> Result<T, HttpResponse>
where 
    T: Clone + Serialize + Debug + Validate
{
    let data = json_data.into_inner();
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
    let log_id = format!("{}-{}",chrono::Utc::now(),uuid::Uuid::new_v4());
    
    let req_log = register_body.clone();
    let register_data = match json_validate(
        register_body
    ) {
        Ok(validated_data) => {
            info_logger(&log_id, "register", "Validate");
            validated_data}
            ,
        Err(err_response) => {
            warning_logger(&log_id, "register", "Validate", &format!("{:?}",err_response.body()));
            return err_response
        }
    };

    // Attempt to register user
    match UserServices::register(
        &log_id,
        register_data,
        &app_data.db,
        &app_data.rabbit
    ).await {
        Ok(user_payload) => {
            debug_logger(&log_id, "register", "Services", &req_log, &user_payload);
            info_logger(&log_id, "register", "Services");
            let end:Instant = Instant::now();
            info!("success response take: {:?}",end - start);
            HttpResponse::Created().json(json!({
                "status": "success",
                "message": "Registration successful",
                "data": user_payload
            }))
        },
        Err(error) => {
            if error.contains("input error"){
                warning_logger(&log_id, "register", "Services", &error);
                HttpResponse::BadRequest().json(json!({
                    "status": "failed",
                    "message": format!("{}", error)
                }))
            }else {
                warning_logger(&log_id, "register", "Services", &error);
                HttpResponse::BadGateway().json(json!({
                    "status": "failed",
                    "message": format!("{}", error)
                }))
            }
        }
    }
}

#[post("/login")]
async fn login_handlers(
    login_body: Json<LoginData>,
    app_data: Data<AppState>
) -> impl Responder{
    let start = Instant::now();
    let log_id = format!("{}-{}",chrono::Utc::now(),uuid::Uuid::new_v4());

    let login_data = login_body.into_inner();

    match UserServices::login(
        &log_id,
        login_data.clone(),
        &app_data.db, 
        &app_data.redis
    ).await{
        Ok(payload)=>{
            debug_logger(&log_id, "login", "Services", &login_data, &payload);
            info_logger(&log_id, "login", "Services");
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
            warning_logger(&log_id, "login", "Services", &errors);
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
    let log_id = format!("{}-{}",chrono::Utc::now(),uuid::Uuid::new_v4());

    let token = req.extensions().get::<String>().cloned().expect("token not found");

    match UserServices::refresh_token(
        token, 
        &app_state.db,
        &app_state.redis
    ).await{
        Ok(access_token)=>{
            info_logger(&log_id,"refresh_token","Services");
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
            let end = Instant::now();
            error_logger("refresh_token Services", &error);

            info!("proccess refresh_token failed, request time: {:?}",end - start);
            HttpResponse::BadGateway().json(json!({
                "status":"failed",
                "message":format!("{}",error)
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