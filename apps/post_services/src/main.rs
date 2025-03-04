use std::{env::var, error::Error, process::exit, sync::Arc};

use config_libs::libs_config;
use config_type::PostAppConfig;
use dotenv::dotenv;
use modules::{post::middleware::AuthMiddleware, post::handler::{AuthPostService, PostService}};
use pgsql_libs::{create_db_pool, DbPool};
use proto_libs::post_proto::{post_server::PostServer, protected_post_server::ProtectedPostServer};
use redis_libs::{redis_connect, RedisPool};
use tonic::{transport::Server, Request};
use logger_libs::Logger as service_logger;
pub mod modules;
pub mod config_type;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok(); 
    env_logger::init();
    let handler_name = "post_services.main";
    
    let config_path = match var("CONFIG_PATH") {
        Ok(path)=>path,
        Err(error)=>{
            service_logger::err_logger(handler_name,"main", "main.config", &error);
            exit(1)
        }
    };
    
    let config: PostAppConfig = match libs_config(&config_path, "POST"){
        Ok(config)=>{
            service_logger::info_logger(handler_name, "main", "main.config_validate");
            config
        },
        Err(error)=>{
            service_logger::err_logger(handler_name, "main", "main.config_validate", error);
            exit(1)
        }
    };
    let address = match config.apps.address.parse(){
        Ok(data)=>data,
        Err(error)=>{
            service_logger::err_logger(&handler_name, "main", "main.config_get_address", error);
            exit(1)
        }
    };
    
    let db_url = config.database.url;
    let (db_min,db_max) = (config.database.min_pool_connection,config.database.max_pool_connection);
    let db_pool: DbPool = match create_db_pool(db_url, db_min, db_max).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("error db_pool: {}", error);
            std::process::exit(1);
        }
    };

    let (redis_min,redis_max) = (config.redis.min_pool_connection,config.redis.max_pool_connection);
    let redis_host = config.redis.host;
    let redis_connect: RedisPool = match redis_connect(redis_host, None,redis_min,redis_max){   
        Ok(redis_pool)=> {
            service_logger::info_logger(handler_name, "main", "main.get_redis_pool");
            redis_pool},
        Err(err)=>{
            service_logger::err_logger(&handler_name, "main","main.get_redis_pool", err);
            exit(1)
        }

    };

    let redis_arc = Arc::new(redis_connect);
    let auth_middleware = AuthMiddleware::new(redis_arc.clone());

    let post = PostService::new(db_pool.clone());
    let protected_post = AuthPostService::new(db_pool.clone());

    let interceptor = move |req: Request<()>| auth_middleware.auth_check(req);

    let services = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto_libs::POST_FILE_DESCRIPTOR_SET)
        .build_v1()?;

    Server::builder()
        .add_service(services)
        .add_service(ProtectedPostServer::with_interceptor(protected_post, interceptor))
        .add_service(PostServer::new(post))
        .serve(address)
        .await?;

    Ok(())
}
