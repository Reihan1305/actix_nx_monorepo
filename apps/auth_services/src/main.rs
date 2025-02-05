use actix_web::{
    get, middleware::Logger, web::{self, scope}, App, HttpResponse, HttpServer, Responder
};
use log::{error, info};
use modules::user_handlers::{auth_config, token_config};
use pgsql_libs::{create_db_pool, DbPool};
use serde_json::json;
use dotenv::dotenv;
use env_logger;
use redis_libs::{RedisPool,redis_connect};
mod middlewares;
mod modules;
mod utils;
mod env_var;
use env_var::{DB_URL, REDIS_HOSTNAME};

pub struct AppState {
    db: DbPool,
    redis: RedisPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables
    env_logger::init(); // enable logger

    info!("Starting server...");

    let db_url: String = DB_URL.clone();
    // try to create db pool 
    let db_pool: DbPool = match create_db_pool(db_url, 5, 50).await {
        Ok(pool) => {
            info!("✅ Database connection success");
            pool
        }
        Err(err) => {
            error!("❌ Database connection failed: {}", err);
            std::process::exit(1);
        }
    };
    let redis_host = REDIS_HOSTNAME.clone();

    let redis_pool: RedisPool = redis_connect(redis_host,None);

    // Start server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db_pool.clone(), redis: redis_pool.clone()}))
            .wrap(Logger::default())
            .service(
                scope("/api")
                    .service(api_health_check)
                    .configure(auth_config)
                    .configure(token_config)
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/healthcheck")]
async fn api_health_check(data: web::Data<AppState>) -> impl Responder {
    let mut error_messages = vec![];

    // Check Database
    if let Err(err) = sqlx::query("SELECT 1;").fetch_one(&data.db).await {
        error!("failed to connect database");
        error_messages.push(json!({
            "database": format!("❌ Cannot connect to database: {}", err)
        }));
    }
    info!("checking database");

    if !error_messages.is_empty() {
        return HttpResponse::BadRequest().json(json!({ "error": error_messages }));
    }

    HttpResponse::Ok().json(json!({ "status": "success", "message": "🚀 API healthy and ready to go!" }))
}
