use rdkafka::{error::KafkaError, producer::{BaseProducer, BaseRecord}, ClientConfig};
use tokio::sync::Mutex;
use std::sync::Arc;

pub type Producer = Arc<Mutex<BaseProducer>>;

pub async fn send_message(producer:&BaseProducer, topic: &str, key: &str, message: &str) -> Result<(), KafkaError> {
    let producer: &BaseProducer = producer;
    
    let record: BaseRecord<'_, [u8], [u8]> = rdkafka::producer::BaseRecord::to(topic)
        .key(key.as_bytes())
        .payload(message.as_bytes());

    producer.send(record).map_err(|(e, _)| e)?;
    Ok(())
}

pub async fn configure_kafka(kafka_host: String) -> Result<BaseProducer, KafkaError> {
    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", kafka_host)
        .set("acks", "all");

    let producer: BaseProducer = config.create()?;
    Ok(producer)
}
