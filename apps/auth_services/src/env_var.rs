use dotenv::var;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref DB_URL: String = var("DATABASE_URL").expect("cant get database from env");
    pub static ref PORT: String = var("PORT").expect("invalid port");
    pub static ref REDIS_HOSTNAME: String = var("REDIS_HOST").expect("invalid hostname");
    pub static ref RABBIT_URL: String = var("RABBIT_URL").expect("invalid url");
}
