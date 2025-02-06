
use actix_web::{
    get, middleware::Logger, web::{self, scope}, App, HttpResponse, HttpServer, Responder
};
use lapin::{options::{BasicPublishOptions, QueueDeclareOptions}, types::FieldTable, BasicProperties};
use log::{error, info};
use modules::user_handlers::{auth_config, token_config, user_config};
use pgsql_libs::{create_db_pool, DbPool};
use r2d2_redis::redis::Commands;
use serde_json::json;
use dotenv::dotenv;
use env_logger;
use redis_libs::{RedisPool,redis_connect};
mod middlewares;
mod modules;
mod env_var;
use env_var::{DB_URL, RABBIT_URL, REDIS_HOSTNAME};
use rabbitmq_libs::{RabbitMqPool,rabbit_connect};
pub struct AppState {
    db: DbPool,
    redis: RedisPool,
    rabbit: RabbitMqPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); 
    env_logger::init();

    info!("Starting server...");

    let db_url: String = DB_URL.clone();
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

    let rabbit_url: String = RABBIT_URL.clone();

    let rabbit_pool: RabbitMqPool = rabbit_connect(rabbit_url);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db_pool.clone(), redis: redis_pool.clone(), rabbit:rabbit_pool.clone()}))
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
    let mut error_messages = vec![];

    // Check Database
    if let Err(err) = sqlx::query("SELECT 1;").fetch_one(&data.db).await {
        error!("failed to connect database");
        error_messages.push(json!({
            "database": format!("❌ Cannot connect to database: {}", err)
        }));
    }
    info!("checking database");

    match data.redis.get() {
        Ok(mut conn) => {
            let _: () = conn.set("testing_redis", "yoo").expect("Failed to set Redis key");
            let redis_value: String = conn.get("testing_redis").expect("Failed to get Redis key");
            info!("✅ Redis healthy: {}", redis_value);
            let _ : () = conn.del("testing_redis").expect("failed to delete redis");
        }
        Err(err) => {
            error!("failed to connect redis");
            error_messages.push(json!({
                "redis": format!("❌ Cannot connect to redis: {}", err)
            }));
        }
    }
    info!("checking redis");

    match data.rabbit.get().await {
        Ok(conn) => {
            let channel = conn.create_channel().await.expect("Failed to create channel");
                channel
                .queue_declare("test_queue", QueueDeclareOptions::default(), FieldTable::default())
                .await
                .expect("Failed to declare queue");
            let _ = channel
                .basic_publish(
                    "",
                    "test_queue",
                    BasicPublishOptions::default(),
                    b"Hello, RabbitMQ!",
                    BasicProperties::default(),
                )
                .await
                .expect("Failed to publish message");
                info!("✅ RabbitMQ is healthy");
        }
        Err(err) => {
            error!("failed to connect rabbit");
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
