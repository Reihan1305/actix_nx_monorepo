use serde::Deserialize;

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Kafka{
    pub host: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Grpc{
    pub url: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct Logger{
    log: String
}

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
pub struct PostGatewayAppConfig{
 pub logger: Logger,
 pub grpc: Grpc,
 pub kafka: Kafka
}
