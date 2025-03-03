use actix_web::{get, post, web::{scope, Data, Json, ServiceConfig}, HttpMessage, HttpRequest, HttpResponse, Responder};
use logger_libs::Logger;
use serde::Serialize;
use serde_json::json;
use validator::Validate;
use std::{borrow::Cow, collections::HashMap, fmt::Debug, time::Instant};
use jwt_libs::types::AccessToken;
use crate::{middlewares::{access_token_middleware::AccessTokenMW, refresh_token_middleware::RefreshTokenMW},AppState};

use super::{model::{LoginData, RegisterData}, service::UserServices};

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
    let handler_name= "register_handler";

    let register_data = match json_validate(register_body) {
        Ok(validated_data) => {
            validated_data
        },
        Err(err_response) => {
            return err_response
        }
    };

    let log_id = format!("{}",register_data.request_id);

    match UserServices::register(
        &log_id,
        register_data,
        &app_data.db,
        &app_data.rabbit
    ).await {
        Ok(user_payload) => {
            let end:Instant = Instant::now();
            Logger::info_logger(handler_name,&log_id, &format!("user_register.{:?}",start - end));
            HttpResponse::Created().json(json!({
                "status": "success",
                "message": "Registration successful",
                "data": user_payload
            }))
        },
        Err(error) => {
            if error.contains("input error"){
                Logger::warning_logger(handler_name, &log_id, "register.db_user_input",&error);
                HttpResponse::BadRequest().json(json!({
                    "status": "failed",
                    "message": format!("{}", error)
                }))
            }else {
                Logger::warning_logger(handler_name, &log_id, "register.db_user_input",&error);
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
    let handler_name= "login_handler";
    let start = Instant::now();
    let log_id = format!("{}",login_body.request_id);

    let login_data = login_body.into_inner();

    match UserServices::login(
        &log_id,
        login_data.clone(),
        &app_data.db, 
        &app_data.redis
    ).await{
        Ok(payload)=>{
            let end = Instant::now();
            Logger::info_logger(handler_name,&log_id, &format!("login_handler.{:?}", end - start));
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"login successfull",
                "data":payload
            }))
        },
        Err(errors)=>{
            Logger::warning_logger(handler_name, &log_id, "login_handler.failed", &errors);
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
    let handler_name= "refresh_token";
    let start = Instant::now();
    let log_id = format!("{} User.Refresh_token",chrono::Utc::now());

    let token = match req.extensions().get::<String>().clone(){
        Some(token)=>token.to_string(),
        None=>{
            let error_message = "token not found";
            Logger::warning_logger(handler_name, &log_id, "refresh_token.get_token_midleware", error_message);
            return HttpResponse::BadRequest().json(json!({
                "status":"error",
                "message": error_message
            }))
        }
    };
    

    match UserServices::refresh_token(
        &log_id,
        token,
        &app_state.db,
        &app_state.redis
    ).await{
        Ok(access_token)=>{
            let end = Instant::now();
            Logger::info_logger(handler_name,&log_id,&format!("access token create, request time : {:?}",end - start));
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"get token success",
                "data":{
                    "access_token":format!("{}",access_token)
                }
            }))
        },
        Err(error)=>{
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
    let log_id = format!("{} User.Refresh_token",chrono::Utc::now());
    let token = req.extensions().get::<AccessToken>().cloned().expect("token not found");
    let handler_name = "find_user_handler";

    match UserServices::find_user_login(&log_id,token, &app_state.db).await{
        Ok(user)=>{
            let end = Instant::now();
            Logger::info_logger(handler_name, &log_id, &format!("get_user_login.{:?}",start-end));
            HttpResponse::Ok().json(json!({
                "status":"success",
                "message":"get user profile success",
                "data":user
            }))
        },
        Err(error)=>{
            Logger::warning_logger(handler_name, &log_id, "get_user_login.query_db", &error);
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