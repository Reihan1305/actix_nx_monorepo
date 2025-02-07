use dotenv::var;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref PORT: String = var("PORT").expect("invalid port");
    pub static ref GRP_CURL: String = var("GRPC_URL").expect("invalid url");
    pub static ref KAFKA_HOST: String = var("KAFKA_HOST").expect("invalid kafka url");
}

