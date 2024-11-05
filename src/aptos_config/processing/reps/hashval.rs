//! Useful for representing State Change Hash, Event Root Hash,
//! Accumulator Hash, etc.  No error handling necessary, as all actions
//! are Infallible.
use super::super::traits::{Encode, FromVec, FromVecRef};

#[derive(Debug, Clone)]
pub struct HashValue {
    bytes: Vec<u8>,
}

impl Encode<String> for HashValue {
    fn encode(&self) -> String {
        format!("0x{}", hex::encode(&self.bytes))
    }
}

impl std::fmt::Display for HashValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashValue({:?})", self.bytes)
    }
}

impl From<&Vec<u8>> for HashValue {
    fn from(value: &Vec<u8>) -> Self {
        Self {
            bytes: value.clone(),
        }
    }
}

impl From<Vec<u8>> for HashValue {
    fn from(value: Vec<u8>) -> Self {
        Self {
            bytes: value.clone(),
        }
    }
}

impl<T> FromVec<T> for HashValue {}
impl<T> FromVecRef<T> for HashValue {}
