use deadpool_lapin::{BuildError, Config, Manager, Pool, Timeouts};
use lapin::ConnectionProperties;
use std::time::Duration;

pub type RabbitMqPool = Pool;

pub fn rabbit_connect(rabbitmq_url: String, max_conn:usize) -> Result<RabbitMqPool,BuildError> {
    let config = Config {
        url: Some(rabbitmq_url),
        ..Default::default()
    };

    let manager = Manager::new(config.url.unwrap(),ConnectionProperties::default());

    match Pool::builder(manager)
    .max_size(max_conn)
    .timeouts(Timeouts{
        wait:Some(Duration::from_secs(60)),
        create:Some(Duration::from_secs(60)),
        recycle:Some(Duration::from_secs(60))
    })
    .runtime(deadpool_lapin::Runtime::Tokio1)
    .build() {
        Ok(rabbit_pool)=>Ok(rabbit_pool),
        Err(err)=>Err(err)
    }
}