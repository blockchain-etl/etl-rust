#![allow(clippy::expect_fun_call)]
use once_cell::sync::OnceCell;

/// Environment key to access the Kafka address
pub const KAFKA_ADDR_ENVKEY: &str = "KAFKA_ADDRESS";
/// Environment key to access the Kafka port, should be a u16
pub const KAFKA_PORT_ENVKEY: &str = "KAFKA_PORT";

/// Kafka Address
pub static KAFKA_ADDR: OnceCell<String> = OnceCell::new();
/// Kafka Port
pub static KAFKA_PORT: OnceCell<u16> = OnceCell::new();

/// Returns the RabbitMQ Address
pub fn get_kafka_addr() -> &'static String {
    KAFKA_ADDR.get_or_init(|| {
        dotenvy::var(KAFKA_ADDR_ENVKEY)
            .expect(&format!("{} should exist in .env file", KAFKA_ADDR_ENVKEY))
            .parse::<String>()
            .unwrap()
    })
}

/// Returns the RabbitMQ port
pub fn get_kafka_port() -> &'static u16 {
    KAFKA_PORT.get_or_init(|| {
        dotenvy::var(KAFKA_PORT_ENVKEY)
            .expect(&format!("{} should exist in .env file", KAFKA_PORT_ENVKEY))
            .parse::<u16>()
            .unwrap()
    })
}
