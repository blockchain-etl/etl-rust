use super::address::{Address, AddressError};
use crate::blockchain_config::processing::traits::{Encode, FromVec, FromVecRef, TryEncode};
use aptos_protos::transaction::v1 as input_protos;

/// Any error occuring when dealing with `MoveModuleId`s
#[derive(Debug, Clone)]
pub enum MoveModuleIdError {
    Address(AddressError),
}

impl From<AddressError> for MoveModuleIdError {
    #[inline]
    fn from(value: AddressError) -> Self {
        MoveModuleIdError::Address(value)
    }
}

impl std::fmt::Display for MoveModuleIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(err) => write!(f, "Error while encoding address: {}", err),
        }
    }
}

impl std::error::Error for MoveModuleIdError {}

/// Simple wrapper for String, allows us to implement custom
/// Module name encoding later
#[derive(Debug, Clone)]
pub struct ModuleName {
    value: String,
}

impl From<&str> for ModuleName {
    #[inline]
    fn from(value: &str) -> Self {
        Self {
            value: String::from(value),
        }
    }
}

impl From<String> for ModuleName {
    #[inline]
    fn from(value: String) -> Self {
        Self { value }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ModuleName {
    #[inline]
    fn to_string(&self) -> String {
        self.value.clone()
    }
}

impl Encode<String> for ModuleName {
    #[inline]
    fn encode(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ModuleId {
    pub address: Address,
    pub name: ModuleName,
}

impl ModuleId {
    #[inline]
    pub fn new<A: Into<Address>, M: Into<ModuleName>>(address: A, name: M) -> Self {
        Self {
            address: address.into(),
            name: name.into(),
        }
    }
    #[inline]
    pub fn split(&self) -> (Address, ModuleName) {
        (self.address.clone(), self.name.clone())
    }
}

impl From<input_protos::MoveModuleId> for ModuleId {
    #[inline]
    fn from(value: input_protos::MoveModuleId) -> Self {
        Self {
            address: (&value.address).into(),
            name: value.name.clone().into(),
        }
    }
}

impl From<&input_protos::MoveModuleId> for ModuleId {
    #[inline]
    fn from(value: &input_protos::MoveModuleId) -> Self {
        Self {
            address: (&value.address).into(),
            name: value.name.clone().into(),
        }
    }
}

impl TryEncode<String> for ModuleId {
    type Error = MoveModuleIdError;
    #[inline]
    fn try_encode(&self) -> Result<String, Self::Error> {
        Ok(format!(
            "{}::{}",
            self.address.try_encode()?,
            self.name.encode()
        ))
    }
}

impl TryEncode<Friend> for ModuleId {
    type Error = MoveModuleIdError;
    fn try_encode(&self) -> Result<Friend, Self::Error> {
        Ok(Friend {
            address: self.address.try_encode()?,
            name: self.name.encode(),
        })
    }
}

use super::super::super::proto_codegen::aptos::modules::module::Friend;

impl<T> FromVec<T> for ModuleId {}
impl<T> FromVecRef<T> for ModuleId {}
