use super::address::Address;
use super::function::{Function, FunctionError};
use super::moduleid::{ModuleId, ModuleName, MoveModuleIdError};
use super::mvstruct::{MvStruct, MvStructError};
use crate::blockchain_config::processing::traits::FromVecRef;
use aptos_protos::transaction::v1 as input_protos;

#[derive(Debug, Clone)]
pub enum ModuleError {
    Function(FunctionError),
    Struct(MvStructError),
    ModuleId(MoveModuleIdError),
}

impl From<FunctionError> for ModuleError {
    fn from(value: FunctionError) -> Self {
        Self::Function(value)
    }
}
impl From<MoveModuleIdError> for ModuleError {
    fn from(value: MoveModuleIdError) -> Self {
        Self::ModuleId(value)
    }
}

impl From<MvStructError> for ModuleError {
    fn from(value: MvStructError) -> Self {
        Self::Struct(value)
    }
}

impl From<std::convert::Infallible> for ModuleError {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!("Should never be using Infallible Error")
    }
}

#[derive(Debug, Clone)]
pub struct ModuleExtraction {
    pub address: Address,
    pub name: ModuleName,
    pub friends: Vec<ModuleId>,
    pub exposed_functions: Vec<Function>,
    pub structs: Vec<MvStruct>,
}

impl TryFrom<input_protos::MoveModule> for ModuleExtraction {
    type Error = ModuleError;
    fn try_from(value: input_protos::MoveModule) -> Result<Self, Self::Error> {
        Ok(ModuleExtraction {
            address: Address::from(&value.address),
            name: ModuleName::from(value.name.clone()),
            friends: ModuleId::try_from_vecref(&value.friends)?,
            exposed_functions: Function::try_from_vecref(&value.exposed_functions)?,
            structs: MvStruct::try_from_vecref(&value.structs)?,
        })
    }
}

impl TryFrom<&input_protos::MoveModule> for ModuleExtraction {
    type Error = ModuleError;
    fn try_from(value: &input_protos::MoveModule) -> Result<Self, Self::Error> {
        Ok(ModuleExtraction {
            address: Address::from(&value.address),
            name: ModuleName::from(value.name.clone()),
            friends: ModuleId::try_from_vecref(&value.friends)?,
            exposed_functions: Function::try_from_vecref(&value.exposed_functions)?,
            structs: MvStruct::try_from_vecref(&value.structs)?,
        })
    }
}
