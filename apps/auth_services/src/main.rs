use actix_web::{
    get, middleware::Logger, web::{self, scope}, App, HttpResponse, HttpServer, Responder
};
use pgsql_libs::{create_db_pool, DbPool};
use serde_json::json;
use dotenv::dotenv;
use env_logger;

mod env_var;
use env_var::DB_URL;

pub struct AppState {
    db: DbPool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables
    env_logger::init(); // enable logger

    let db_url: String = DB_URL.clone();

    // try to create db pool 
    let db_pool: DbPool = match create_db_pool(db_url, 5, 50).await {
        Ok(pool) => {
            println!("✅ Database connection success");
            pool
        }
        Err(err) => {
            eprintln!("❌ Database connection failed: {}", err);
            std::process::exit(1);
        }
    };

    // Start server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { db: db_pool.clone() }))
            .wrap(Logger::default()) // add logging middleware
            .service(
                scope("/api")
                    .service(api_health_check)
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
        error_messages.push(json!({
            "database": format!("❌ Cannot connect to database: {}", err)
        }));
    }

    if !error_messages.is_empty() {
        return HttpResponse::BadRequest().json(json!({ "error": error_messages }));
    }

    HttpResponse::Ok().json(json!({ "status": "success", "message": "🚀 API healthy and ready to go!" }))
}
