use dotenvy;
use log::warn;
use once_cell::sync::OnceCell;
use std::{env::VarError, time::Duration};
use url::Url;

// Environment keys
/// The key to look for the Aptos GRPC address
const GRPC_ADDRESS_URL_ENVKEY: &str = "APTOS_GRPC_ADDR";
/// The key to look for the fallback Aptos GRPC address
const GRPC_ADDRESS_URL_FB_ENVKEY: &str = "APTOS_GRPC_ADDR_FALLBACK";
/// The key to look for the ping timeout
const GRPC_PING_TIMEOUT_ENVKEY: &str = "APTOS_GRPC_PING_TIMEOUT";
/// The key to look for the ping inteval
const GRPC_PING_INTERVAL_ENVKEY: &str = "APTOS_GRPC_PING_INTERVAL";
/// The key to look for the grpc auth key
const GRPC_APTOS_AUTH_ENVKEY: &str = "APTOS_GRPC_AUTH";
/// The key to look for the grpc auth key for fallback
const GRPC_APTOS_AUTH_FB_ENVKEY: &str = "APTOS_GRPC_AUTH_FALLBACK";
/// The key to look for the project name
const GRPC_PROJECT_NAME_ENVKEY: &str = "APTOS_GRPC_PROJECT_NAME";

// DEFAULT VALUES
/// Default GRPC ping timeout.  Used if not set by the .env
pub const GRPC_PING_TIMEOUT_DEFAULT: &str = "10";
/// Default GRPC ping timeout.  Used if not set by the .env
pub const GRPC_PING_INTERVAL_DEFAULT: &str = "10";
/// Default GRPC ping timeout.  Used if not set by the .env
pub const GRPC_PROJECT_NAME_DEFAULT: &str = "CUSTOM";

// ONCE CELLS
/// The GRPC Address
static GRPC_ADDR: OnceCell<String> = OnceCell::new();
/// The GRPC Address (FB) (optional)
static GRPC_ADDR_FB: OnceCell<Option<String>> = OnceCell::new();
/// The GRPC timeout waiting for ping
static GRPC_PING_TIMEOUT: OnceCell<u64> = OnceCell::new();
/// The GRPC interval between pings
static GRPC_PING_INTERVAL: OnceCell<u64> = OnceCell::new();
/// The grpc auth string to add as a Bearer header
static GRPC_AUTH: OnceCell<String> = OnceCell::new();
/// The grpc auth string to add as a Bearer header (FB) (Optional)
static GRPC_AUTH_FB: OnceCell<Option<String>> = OnceCell::new();
/// The name of the project (for aptos info)
static GRPC_PROJ_NAME: OnceCell<String> = OnceCell::new();

// ACCESS FUNCTIONS
/// Returns the grpc service address as a String
pub fn get_service_address_str() -> &'static String {
    GRPC_ADDR.get_or_init(|| {
        dotenvy::var(GRPC_ADDRESS_URL_ENVKEY)
            .unwrap_or_else(|_| panic!("{} should exist in .env file", GRPC_ADDRESS_URL_ENVKEY))
            .parse::<String>()
            .unwrap()
    })
}
/// Returns the grpc service address as a String
pub fn get_service_address_str_fallback() -> &'static Option<String> {
    GRPC_ADDR_FB.get_or_init(|| match dotenvy::var(GRPC_ADDRESS_URL_FB_ENVKEY) {
        Ok(value) => Some(value.parse::<String>().expect("Failed to parse")),
        Err(dotenvy::Error::EnvVar(VarError::NotPresent)) => None,
        Err(err) => panic!("Error while getting fallback: {}", err),
    })
}

/// Returns the GRPC service address as a parsed URL
pub fn get_service_address() -> Url {
    let url_string = get_service_address_str();
    Url::parse(url_string).unwrap_or_else(|_| panic!("Failed to parse url {}", url_string))
}

/// Returns the GRPC fallback service address as a parsed URL
pub fn get_service_address_fallback() -> Option<Url> {
    let url_string = get_service_address_str_fallback();
    url_string.as_ref().map(|string| {
        Url::parse(string).unwrap_or_else(|_| panic!("Failed to parse url {}", string))
    })
}

/// Returns the grpc auth token as a String
pub fn get_auth_token() -> &'static String {
    GRPC_AUTH.get_or_init(|| {
        dotenvy::var(GRPC_APTOS_AUTH_ENVKEY)
            .unwrap_or({
                warn!(
                    "No env var {} in .env file. Attempting to connect to Aptos gRPC without it...",
                    GRPC_APTOS_AUTH_ENVKEY
                );
                String::new()
            })
            .parse::<String>()
            .unwrap()
    })
}

/// Returns the grpc auth token as a String
pub fn get_auth_token_fallback() -> &'static Option<String> {
    GRPC_AUTH_FB.get_or_init(|| match dotenvy::var(GRPC_APTOS_AUTH_FB_ENVKEY) {
        Ok(auth) => Some(auth),
        Err(dotenvy::Error::EnvVar(VarError::NotPresent)) => None,
        Err(err) => unreachable!("Failed to get fallback auth key: {}", err),
    })
}

/// Returns the grpc auth token as a String
#[inline]
pub fn get_ping_timeout_u64() -> &'static u64 {
    GRPC_PING_TIMEOUT.get_or_init(|| {
        dotenvy::var(GRPC_PING_TIMEOUT_ENVKEY)
            .unwrap_or(String::from(GRPC_PING_TIMEOUT_DEFAULT))
            .parse::<u64>()
            .unwrap()
    })
}

/// Returns the grpc auth token as a String
#[inline]
pub fn get_ping_interval_u64() -> &'static u64 {
    GRPC_PING_INTERVAL.get_or_init(|| {
        dotenvy::var(GRPC_PING_INTERVAL_ENVKEY)
            .unwrap_or(String::from(GRPC_PING_INTERVAL_DEFAULT))
            .parse::<u64>()
            .unwrap()
    })
}

/// Returns the ping timeout duration
pub fn get_ping_timeout() -> Duration {
    Duration::from_secs(*get_ping_timeout_u64())
}

/// Returns the ping interval duration
pub fn get_ping_interval() -> Duration {
    Duration::from_secs(*get_ping_interval_u64())
}

pub fn get_project_name() -> &'static String {
    GRPC_PROJ_NAME.get_or_init(|| {
        dotenvy::var(GRPC_PROJECT_NAME_ENVKEY)
            .unwrap_or(String::from(GRPC_PROJECT_NAME_DEFAULT))
            .parse::<String>()
            .unwrap()
    })
}
