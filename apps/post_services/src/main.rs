use std::{error::Error, sync::Arc};

use config_libs::libs_config;
use config_type::PostAppConfig;
use modules::{post::middleware::AuthMiddleware, post::handler::{AuthPostService, PostService}};
use pgsql_libs::{create_db_pool, DbPool};
use proto_libs::post_proto::{post_server::PostServer, protected_post_server::ProtectedPostServer};
use redis_libs::{redis_connect, RedisPool};
use tonic::{transport::Server, Request};

pub mod modules;
pub mod config_type;
// pub mod proto {
//     tonic::include_proto!("post");

//     pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
//         tonic::include_file_descriptor_set!("post_descriptor");
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = "[::1]:50501".parse()?;

    let config: PostAppConfig = libs_config("config/post_config", "POST");

    let db_url = config.database.url;
    let db_pool: DbPool = match create_db_pool(db_url, 5, 50).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("error db_pool: {}", error);
            std::process::exit(1);
        }
    };

    let redis_host = config.redis.host;
    let redis_connect: RedisPool = redis_connect(redis_host, None);

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
