#![doc = include_str!("README.md")]

pub mod metrics;
pub mod output;

/*#[cfg(feature = "SOLANA")]
//#[rustfmt::skip]
pub mod solana_config;
#[cfg(feature = "SOLANA")]
pub use solana_config as blockchain_config;
*/
#[cfg(feature = "APTOS")]
//#[rustfmt::skip]
pub mod aptos_config;
#[cfg(feature = "APTOS")]
pub use aptos_config as blockchain_config;

#[cfg(feature = "RPC")]
pub use source::json_rpc::*;
