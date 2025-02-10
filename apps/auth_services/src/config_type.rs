// use dotenv::var;
// use lazy_static::lazy_static;
// lazy_static! {
//     pub static ref URL: String = var("DATABASE_URL").expect("cant get database from env");
//     pub static ref PORT: String = var("PORT").expect("invalid port");
//     pub static ref REDIS_HOSTNAME: String = var("REDIS_HOST").expect("invalid hostname");
//     pub static ref RABBIT_URL: String = var("RABBIT_URL").expect("invalid url");
// }


use serde::Deserialize;

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Database{
    pub user: String,
    pub password: String,
    pub name: String,
    pub port: u64,
    pub host: String,
    pub url: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Apps{
    port:u64
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Redis{
    pub host: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct RabbitMq{
    pub url: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Logger{
    log: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct UserAppConfig{
 pub apps: Apps,
 pub database: Database,
 pub redis: Redis,
 pub rabbitmq: RabbitMq,
 pub logger: Logger
}
