use dotenv::var;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref DB_URL: String = var("DATABASE_URL").expect("cant get database from env");
}
