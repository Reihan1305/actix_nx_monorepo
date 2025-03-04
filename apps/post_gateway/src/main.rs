use actix_web::{
    middleware::Logger,
    web::{scope, Data},
    App, HttpServer
};
use config_type::PostGatewayAppConfig;
use modules::post::handler::{post_config, protected_post_config};
use dotenv::dotenv;
use env_logger; 
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use kafka_libs::{Producer,configure_kafka};
mod config_type;
mod modules;
use logger_libs::Logger as ServiceLogger;

use proto_libs::post_proto::{post_client::PostClient, protected_post_client::ProtectedPostClient};


// Define AppState with both PostClient and ProtectedPostClient
pub struct AppState {
    post_client: Arc<Mutex<PostClient<Channel>>>,
    protected_post_client: Arc<Mutex<ProtectedPostClient<Channel>>>,
    kafka_producer: Producer
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let handler_name = "post_gateway.main";
    
    let config_path = match dotenv::var("CONFIG_PATH"){
        Ok(path)=>path,
        Err(err)=>{
            ServiceLogger::err_logger(&handler_name, "main", "post_gateway.get_path", &err);
            panic!("{}",err)
        }
    };

    let config:PostGatewayAppConfig = match config_libs::libs_config(&config_path, "POST-GATEWAY"){
        Ok(config)=>config,
        Err(error)=>{
            ServiceLogger::err_logger(&handler_name, "main", "postgateway.get_config", &error);
            panic!("{}",error);
        }
    };

    let grpc_url = config.grpc.url;

    let post_client = match PostClient::connect(grpc_url.clone()).await{
        Ok(client)=>client,
        Err(error)=>{
            ServiceLogger::err_logger(&handler_name, "main", "postgateway.get_postclient", &error);
            panic!("{}",error);
        }
    };
    let protected_post_client = match ProtectedPostClient::connect(grpc_url.clone()).await {
        Ok(protected_client)=>protected_client,
        Err(error)=>{
            ServiceLogger::err_logger(&handler_name, "main", "postgateaway.get_protected_lclien", &error);
            panic!("{}",error)
        }
    };

    let kafka_url = config.kafka.host;
    let kafka_config = match configure_kafka(kafka_url).await{
        Ok(kafka)=> kafka,
        Err(error)=>{
            ServiceLogger::err_logger(&handler_name, "main", "postgateaway.get_kafka_config", &error);
            panic!("{}",error)
        }
    };

    let state = Data::new(AppState {
        post_client: Arc::new(Mutex::new(post_client)),
        protected_post_client: Arc::new(Mutex::new(protected_post_client)),
        kafka_producer:Arc::new(Mutex::new(kafka_config))
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .configure(protected_post_config)
                    .configure(post_config)
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}