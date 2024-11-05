//! This module contains the functionality for taking transaction
//! extractions and turning it into our output records.
//!
//! For each schemas, there should be a function that takes in a TransactionExtraction
//! and returns either Ok(None) if that transaction is not applicable for those
//! records, Ok(Vec<T>) where T is the record type if it is applicable, and
//! Err(error) in the event that there is an error.

pub mod all;
pub mod blocks;
pub mod changes;
pub mod events;
pub mod options;
pub mod signatures;
pub mod transactions;

pub mod byte_processing {
    use base64::prelude::{Engine, BASE64_STANDARD};
    pub fn bytes_to_base64(vec: &Vec<u8>) -> String {
        BASE64_STANDARD.encode(vec)
    }
    pub fn option_bytes_to_base64(vec: &Option<Vec<u8>>) -> Option<String> {
        vec.as_ref().map(bytes_to_base64)
    }
}
