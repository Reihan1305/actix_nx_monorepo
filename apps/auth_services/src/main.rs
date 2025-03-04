
use std::process::exit;

use actix_web::{
    get, middleware::Logger, web::{self, scope}, App, HttpResponse, HttpServer, Responder
};
use config_type::UserAppConfig;
use lapin::{options::{BasicPublishOptions, QueueDeclareOptions}, types::FieldTable, BasicProperties};

use modules::user::handler::{auth_config, token_config, user_config};
use pgsql_libs::{create_db_pool, DbPool};
use r2d2_redis::redis::{Commands, RedisError};
use serde_json::json;
use dotenv::{dotenv, var};
use env_logger;
use redis_libs::{RedisPool,redis_connect};
mod middlewares;
mod modules;
use logger_libs::Logger as service_logger;
mod config_type;
use rabbitmq_libs::{RabbitMqPool,rabbit_connect};
pub struct AppState {
    db: DbPool,
    redis: RedisPool ,
    rabbit: RabbitMqPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); 
    env_logger::init();
    let handler_name = "main_auth_services";
    let config_path = match var("CONFIG_PATH") {
        Ok(path)=>path,
        Err(error)=>{
            service_logger::err_logger(handler_name,"main", "main.config", &error);
            exit(1)
        }
    };
    
    let config: UserAppConfig= match config_libs::libs_config(&config_path,"USER"){
        Ok(data_config) => {
            service_logger::info_logger(handler_name,"main", "main.config");
            data_config
        },
        Err(err)=>{
            service_logger::err_logger(handler_name,"main", "main.config", &err);
            exit(1)
        }
    };

    let db_url: String= config.database.url;

    let db_pool: DbPool= match create_db_pool(db_url, 5, 50).await {
        Ok(pool) => {
            service_logger::info_logger(handler_name,"main","main.connectDb");
            pool
        }
        Err(err) => {
            service_logger::err_logger(handler_name,"main", "main.cconnectdb",&err);
            std::process::exit(1);
        }
    };
    let redis_host: String = config.redis.host;

    let redis_pool: RedisPool= match redis_connect(redis_host,None,10,50){
        Ok(connections)=>{
            service_logger::info_logger(handler_name,"main", "main.redis_connection");
            connections
        },
        Err(error)=>{
            service_logger::err_logger(handler_name,"main", "main.redis_connection", &error);
            exit(1)
        }
    };

    let rabbit_url: String= config.rabbitmq.url;

    let rabbit_pool: RabbitMqPool= match rabbit_connect(rabbit_url,10){
        Ok(rb_pool)=>{
            service_logger::info_logger(handler_name,"main", "main.rabbitmq_connections");
            rb_pool
        },
        Err(error)=>{
            service_logger::err_logger(handler_name,"main", "main.rabbitmq_connections", &error);
            exit(1)
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState
                {
                    db: db_pool.clone(), 
                    redis: redis_pool.clone(), 
                    rabbit:rabbit_pool.clone()
                }
            ))
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .service(api_health_check)
                    .configure(auth_config)
                    .configure(token_config)
                    .configure(user_config)
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/healthcheck")]
async fn api_health_check(data: web::Data<AppState>) -> impl Responder {
    let log_id = "healtcheck";
    let handler_name = "health_check";
    let mut error_messages = vec![];

    if let Err(err) = sqlx::query("SELECT 1;").fetch_one(&data.db).await {
        service_logger::err_logger(handler_name,log_id, "healtcheck.database_check", &err);
        error_messages.push(json!({
            "database": format!("❌ Cannot connect to database: {}", err)
        }));
    }

    service_logger::info_logger(&handler_name,log_id, "healtcheck.database_check");

    match data.redis.get() {
        Ok(mut conn) => {
            let _: () = match conn.set::<&str,&str,String>("testing_redis", "yoo"){
                Ok(_)=>(),
                Err(error)=>{
                    service_logger::err_logger(handler_name,log_id,"healtcheck.redischeck", &error);
                    error_messages.push(json!({
                        "database": format!("❌ Cannot connect to redis: {}", error)
                    }));
                    ()
                }
            };
            let redis_value: Result<String,RedisError> = match conn.get("testing_redis") {
                Ok(value) => {
                    service_logger::info_logger(handler_name,log_id,"healthcheck.redischeck");
                    Ok(value)
                },
                Err(error)=>{
                    Err(error)
                }
            };
            match redis_value {
                Ok(_)=>{
                    service_logger::info_logger(handler_name,log_id, "healthcheck.redischeck");
                    let _ : () = conn.del("testing_redis").expect("failed to delete redis");
                },
                Err(error)=>{
                    service_logger::err_logger(handler_name,log_id, "healthcheck.redischeck", &error);
                    error_messages.push(json!({
                        "database": format!("❌ Cannot connect to redis: {}", error)
                    }));
                }
            }
        }
        Err(err) => {
            service_logger::err_logger(handler_name,log_id, "healthcheck.redischeck", &err);
            error_messages.push(json!({
                "redis": format!("❌ Cannot connect to redis: {}", err)
            }));
        }
    }

    service_logger::info_logger(handler_name,log_id,"healthcheck.redischeck");

    match data.rabbit.get().await {
        Ok(conn) => {
            match conn.create_channel().await{
                Ok(channel)=>{
                    match channel
                    .queue_declare("test_queue", QueueDeclareOptions::default(), FieldTable::default())
                    .await {
                        Ok(_)=>(),
                        Err(error)=>{
                            error_messages.push(json!({
                                "rabbit": format!("❌ Cannot connect to rabbit: {}", error)
                            }));
                        }
                    }

                    match channel
                    .basic_publish(
                        "",
                        "test_queue",
                        BasicPublishOptions::default(),
                        b"Hello, RabbitMQ!",
                        BasicProperties::default(),
                    )
                    .await {
                        Ok(_)=>(),
                        Err(error)=>{
                            service_logger::err_logger(handler_name,log_id, "healthcheck.rabbitmq_check", &error);
                            error_messages.push(json!({
                                "rabbit": format!("❌ Cannot connect to rabbit: {}", error)
                            }));
                        }   
                    }
                },
                Err(error)=>{
                    service_logger::err_logger(handler_name,log_id, "healthcheck.rabbitmq_check", &error);
                    error_messages.push(json!({
                        "rabbit": format!("❌ Cannot connect to rabbit: {}", error)
                    }));
                }
            };
              service_logger::info_logger(handler_name,log_id, "healthcheck.rabbitmq_check");
        }
        Err(err) => {
            service_logger::err_logger(handler_name,log_id, "healthcheck.rabbitmq_check", &err);
            error_messages.push(json!({
                "rabbit": format!("❌ Cannot connect to rabbit: {}", err)
            }));
        }
    }

    if !error_messages.is_empty() {
        return HttpResponse::BadRequest().json(json!({ "error": error_messages }));
    }

    HttpResponse::Ok().json(json!({ "status": "success", "message": "🚀 API healthy and ready to go!" }))
}
