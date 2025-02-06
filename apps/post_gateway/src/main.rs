use actix_web::{
    get, middleware::Logger, web::scope, App, HttpResponse, HttpServer, Responder
};
use log::info;
use serde_json::json;
use dotenv::dotenv;
use env_logger;
mod env_var;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); 
    env_logger::init();

    info!("Starting server...");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .service(api_health_check)
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