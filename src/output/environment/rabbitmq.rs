#![allow(clippy::expect_fun_call)]

use once_cell::sync::OnceCell;

/// Environment key to access the RABBITMQ address
pub const RABBITMQ_ADDR_ENVKEY: &str = "RABBITMQ_ADDRESS";
/// Environment key to access the RABBITMQ port, should be a u16
pub const RABBITMQ_PORT_ENVKEY: &str = "RABBITMQ_PORT";
/// Environment key to access the RabbitMQ username
pub const RABBITMQ_USER_ENVKEY: &str = "RABBITMQ_USER";
/// Environment key to access the RABBITMQ password
pub const RABBITMQ_PASS_ENVKEY: &str = "RABBITMQ_PASSWORD";

/// RabbitMQ Address
pub static RABBITMQ_ADDR: OnceCell<String> = OnceCell::new();
/// RabbitMQ Port
pub static RABBITMQ_PORT: OnceCell<u16> = OnceCell::new();
/// RabbitMQ User
pub static RABBITMQ_USER: OnceCell<String> = OnceCell::new();
/// RabbitMQ Password
pub static RABBITMQ_PASS: OnceCell<String> = OnceCell::new();

/// Returns the RabbitMQ Address
pub fn get_rabbitmq_addr() -> &'static String {
    RABBITMQ_ADDR.get_or_init(|| {
        dotenvy::var(RABBITMQ_ADDR_ENVKEY)
            .expect(&format!(
                "{} should exist in .env file",
                RABBITMQ_ADDR_ENVKEY
            ))
            .parse::<String>()
            .unwrap()
    })
}

/// Returns the RabbitMQ port
pub fn get_rabbitmq_port() -> &'static u16 {
    RABBITMQ_PORT.get_or_init(|| {
        dotenvy::var(RABBITMQ_PORT_ENVKEY)
            .expect(&format!(
                "{} should exist in .env file",
                RABBITMQ_PORT_ENVKEY
            ))
            .parse::<u16>()
            .unwrap()
    })
}

/// Returns the RabbitMQ username
pub fn get_rabbitmq_username() -> &'static String {
    RABBITMQ_USER.get_or_init(|| {
        dotenvy::var(RABBITMQ_USER_ENVKEY)
            .expect(&format!(
                "{} should exist in .env file",
                RABBITMQ_USER_ENVKEY
            ))
            .parse::<String>()
            .unwrap()
    })
}

/// Returns the RabbitMQ password (secrets beaware)
pub fn get_rabbitmq_password() -> &'static String {
    RABBITMQ_PASS.get_or_init(|| {
        dotenvy::var(RABBITMQ_PASS_ENVKEY)
            .expect(&format!(
                "{} should exist in .env file",
                RABBITMQ_PASS_ENVKEY
            ))
            .parse::<String>()
            .unwrap()
    })
}
