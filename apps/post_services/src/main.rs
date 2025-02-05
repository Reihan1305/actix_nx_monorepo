use std::{env::var, error::Error};

use modules::post_services::PostService;
use pgsql_libs::{create_db_pool, DbPool};
use proto::post_server::PostServer;
use tonic::transport::Server;

pub mod modules;

pub mod proto {
    tonic::include_proto!("post");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("post_descriptor");
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>>{
    let address = "[::1]:50501".parse()?;

    let db_url = var("DATABASE_URL").expect("invalid db url");

    let db_pool: DbPool = match create_db_pool(db_url, 5, 50).await{
        Ok(pool)=>pool,
        Err(error)=>{
            println!("error db_pool: {}",error);
            std::process::exit(1);
        }
    };

    let post = PostService::new(db_pool);

    let services = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
    .build_v1()?;

    Server::builder()
    .add_service(services)
    .add_service(PostServer::new(post))
    .serve(address)
    .await?;

    Ok(())
}