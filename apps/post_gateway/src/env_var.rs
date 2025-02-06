use dotenv::var;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref PORT: String = var("PORT").expect("invalid port");
}
