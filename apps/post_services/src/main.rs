use std::{env::var, error::Error, sync::Arc};

use modules::{post_middleware::AuthMiddleware, post_services::{AuthPostService, PostService}};
use pgsql_libs::{create_db_pool, DbPool};
use proto::{post_server::PostServer, protected_post_server::ProtectedPostServer};
use redis_libs::{redis_connect, RedisPool};
use tonic::{transport::Server, Request};

pub mod modules;

pub mod proto {
    tonic::include_proto!("post");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("post_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = "[::1]:50501".parse()?;

    // **1. Buat koneksi database**
    let db_url = var("DATABASE_URL").expect("invalid db url");
    let db_pool: DbPool = match create_db_pool(db_url, 5, 50).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("error db_pool: {}", error);
            std::process::exit(1);
        }
    };

    // **2. Buat koneksi Redis**
    let redis_host = var("REDIS_HOST").expect("invalid redis host");
    let redis_connect: RedisPool = redis_connect(redis_host, None);

    // **3. Bungkus redis pool dalam Arc**
    let redis_arc = Arc::new(redis_connect);
    let auth_middleware = AuthMiddleware::new(redis_arc.clone());

    // **4. Buat instance service**
    let post = PostService::new(db_pool.clone());
    let protected_post = AuthPostService::new(db_pool.clone());

    // **5. Middleware harus digunakan sebagai Interceptor**
    let interceptor = move |req: Request<()>| auth_middleware.auth_check(req);

    // **6. Buat reflection service**
    let services = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    // **7. Jalankan gRPC Server**
    Server::builder()
        .add_service(services)
        .add_service(ProtectedPostServer::with_interceptor(protected_post, interceptor))
        .add_service(PostServer::new(post))
        .serve(address)
        .await?;

    Ok(())
}
