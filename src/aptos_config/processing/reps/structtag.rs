use crate::blockchain_config::processing::traits::TryEncode;

use super::super::super::proto_codegen::aptos::resource_extras::StructTag as StructTagOut;
use super::super::traits::Encode;
use super::super::traits::FromVec;
use super::address::{Address, AddressError};
use super::generictypeparam::GenericTypeParamError;
use super::moduleid::ModuleName;
use super::movetype::{MoveType, MoveTypeError};
use aptos_protos::transaction::v1 as input_protos;

#[derive(Debug, Clone)]
pub struct StructName {
    value: String,
}

impl From<&str> for StructName {
    #[inline]
    fn from(value: &str) -> Self {
        Self {
            value: String::from(value),
        }
    }
}

impl From<&String> for StructName {
    #[inline]
    fn from(value: &String) -> Self {
        Self {
            value: value.clone(),
        }
    }
}

impl Encode<String> for StructName {
    #[inline]
    fn encode(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub enum StructTagError {
    GenericTypeParam(Box<GenericTypeParamError>),
    Address(Box<AddressError>),
    MoveType(Box<MoveTypeError>),
}

impl std::fmt::Display for StructTagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Address(err) => write!(f, "Issue encoding the address: {}", err),
            Self::GenericTypeParam(err) => {
                write!(f, "Issue encoding the generictypeparams: {}", err)
            }
            Self::MoveType(err) => write!(f, "Issue encoding the movetypes: {}", err),
        }
    }
}

impl From<GenericTypeParamError> for StructTagError {
    fn from(value: GenericTypeParamError) -> Self {
        StructTagError::GenericTypeParam(Box::new(value))
    }
}
impl From<AddressError> for StructTagError {
    fn from(value: AddressError) -> Self {
        StructTagError::Address(Box::new(value))
    }
}

impl From<MoveTypeError> for StructTagError {
    fn from(value: MoveTypeError) -> Self {
        StructTagError::MoveType(Box::new(value))
    }
}

#[derive(Debug, Clone)]
pub struct StructTag {
    pub address: Address,
    pub module: ModuleName,
    pub name: StructName,
    pub generic_type_params: Vec<MoveType>,
}

impl TryFrom<input_protos::MoveStructTag> for StructTag {
    type Error = StructTagError;
    #[inline]
    fn try_from(value: input_protos::MoveStructTag) -> Result<Self, Self::Error> {
        Ok(Self {
            address: (&value.address).into(),
            module: value.module.into(),
            name: (&value.name).into(),
            generic_type_params: MoveType::try_from_vec(value.generic_type_params)?,
        })
    }
}

impl TryEncode<String> for StructTag {
    type Error = StructTagError;
    fn try_encode(&self) -> Result<String, Self::Error> {
        Ok(format!(
            "{}::{}::{}{}",
            self.address.try_encode()?,
            self.module.encode(),
            self.name.encode(),
            {
                let mut strings = Vec::new();
                for movetype in self.generic_type_params.iter() {
                    strings.push(movetype.try_encode()?);
                }
                if !strings.is_empty() {
                    format!("{}{}{}", "<", strings.join(","), ">")
                } else {
                    String::from("")
                }
            }
        ))
    }
}

impl TryEncode<StructTagOut> for StructTag {
    type Error = StructTagError;
    fn try_encode(&self) -> Result<StructTagOut, Self::Error> {
        Ok(StructTagOut {
            address: self.address.try_encode()?,
            module: self.module.encode(),
            name: self.name.encode(),
            generic_type_params: {
                let mut strings = Vec::new();
                for movetype in self.generic_type_params.iter() {
                    strings.push(movetype.try_encode()?);
                }
                strings
            },
        })
    }
}
