use actix_web::{
    get, middleware::Logger,
    web::{scope, Data},
    App, HttpResponse, HttpServer, Responder
};
use env_var::{GRP_CURL, KAFKA_HOST};
use log::info;
use modules::post_handler::{post_config, protected_post_config};
use serde_json::json;
use dotenv::dotenv;
use env_logger;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use kafka_libs::{Producer,configure_kafka};
mod env_var;
mod modules;

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

    let grpc_url = GRP_CURL.clone();

    let post_client = PostClient::connect(grpc_url.clone()).await.expect("Failed to connect to PostClient");
    let protected_post_client = ProtectedPostClient::connect(grpc_url.clone()).await.expect("Failed to connect to ProtectedPostClient");

    let kafka_url = KAFKA_HOST.clone();
    let kafka_config = match configure_kafka(kafka_url).await{
        Ok(kafka)=> kafka,
        Err(error)=>{
            log::error!("kafka error: {}",error);
            std::process::exit(1)
        }
    };

    let state = Data::new(AppState {
        post_client: Arc::new(Mutex::new(post_client)),
        protected_post_client: Arc::new(Mutex::new(protected_post_client)),
        kafka_producer:Arc::new(Mutex::new(kafka_config))
    });

    info!("🚀 Starting server on http://0.0.0.0:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .service(api_health_check)
                    .configure(protected_post_config)
                    .configure(post_config)
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}

#[get("/healthcheck")]
async fn api_health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "success", "message": "🚀 API healthy and ready to go!" }))
}