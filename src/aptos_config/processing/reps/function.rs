use crate::blockchain_config::processing::traits::TryEncode;

use super::super::super::proto_codegen::aptos::modules::module::{
    exposed_function::GenericFunctionTypeParams, ExposedFunction,
};
use super::super::traits::{FromVec, FromVecRef};
use super::generictypeparam::{GenericTypeParam, GenericTypeParamError};
use super::movetype::{MoveType, MoveTypeError};
use super::visibility::{IntermediateVisibility, VisibilityError};
use aptos_protos::transaction::v1::MoveFunction;
use log::error;

#[derive(Debug, Clone)]
pub enum FunctionError {
    MoveType(MoveTypeError),
    GenericTypeParam(GenericTypeParamError),
    Visibility(VisibilityError),
}
impl From<MoveTypeError> for FunctionError {
    fn from(value: MoveTypeError) -> Self {
        Self::MoveType(value)
    }
}
impl From<GenericTypeParamError> for FunctionError {
    fn from(value: GenericTypeParamError) -> Self {
        Self::GenericTypeParam(value)
    }
}
impl From<VisibilityError> for FunctionError {
    fn from(value: VisibilityError) -> Self {
        Self::Visibility(value)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionName {
    pub value: String,
}

impl From<String> for FunctionName {
    fn from(value: String) -> Self {
        Self { value }
    }
}
#[allow(clippy::from_over_into)]
impl Into<String> for FunctionName {
    fn into(self) -> String {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: FunctionName,
    pub visibility: IntermediateVisibility,
    pub is_entry: bool,
    pub generic_type_params: Vec<GenericTypeParam>,
    pub params: Vec<MoveType>,
    pub r#return: Vec<MoveType>,
}

impl<T> FromVec<T> for Function {}
impl<T> FromVecRef<T> for Function {}

impl TryFrom<MoveFunction> for Function {
    type Error = FunctionError;
    fn try_from(value: MoveFunction) -> Result<Self, Self::Error> {
        Ok(Function {
            name: FunctionName::from(value.name.clone()),
            visibility: IntermediateVisibility::from(value.visibility()),
            is_entry: value.is_entry,
            generic_type_params: GenericTypeParam::from_vec(value.generic_type_params),
            params: match MoveType::try_from_vec(value.params) {
                Ok(vec) => vec,
                Err(err) => {
                    error!("Failed processing params from aptos MoveFunction into our Function");
                    return Err(err.into());
                }
            },
            r#return: match MoveType::try_from_vec(value.r#return) {
                Ok(vec) => vec,
                Err(err) => {
                    error!("Failed processing returns from aptos MoveFunction into our Function");
                    return Err(err.into());
                }
            },
        })
    }
}

impl TryFrom<&MoveFunction> for Function {
    type Error = FunctionError;
    fn try_from(value: &MoveFunction) -> Result<Self, Self::Error> {
        Ok(Function {
            name: FunctionName::from(value.name.clone()),
            visibility: IntermediateVisibility::from(value.visibility()),
            is_entry: value.is_entry,
            generic_type_params: Vec::from_iter(
                value.generic_type_params.iter().map(GenericTypeParam::from),
            ),
            params: {
                let mut params = Vec::with_capacity(value.params.len());
                for param in value.params.iter() {
                    params.push(match MoveType::try_from(param.clone()) {
                        Ok(mt) => mt,
                        Err(err) => {
                            error!("Failed to convert param from MoveFunction to our Function!");
                            return Err(err.into());
                        }
                    })
                }
                params
            },
            r#return: {
                let mut r#returns = Vec::with_capacity(value.r#return.len());
                for ret in value.r#return.iter() {
                    r#returns.push(match MoveType::try_from(ret.clone()) {
                        Ok(mt) => mt,
                        Err(err) => {
                            error!("Failed to convert return from MoveFunction to our Function!");
                            return Err(err.into());
                        }
                    })
                }
                r#returns
            },
        })
    }
}

impl TryInto<ExposedFunction> for Function {
    type Error = FunctionError;

    fn try_into(self) -> Result<ExposedFunction, Self::Error> {
        let mut gentypeparams: Vec<GenericFunctionTypeParams> = Vec::new();
        for gentypeparam in self.generic_type_params.iter() {
            gentypeparams.push(match gentypeparam.try_encode() {
                Ok(gtfp) => gtfp,
                Err(err) => {
                    error!("Failed to convert our function into ExposedFunction export proto");
                    return Err(err.into());
                }
            });
        }

        let mut params: Vec<String> = Vec::new();
        for param in self.params.iter() {
            params.push(match param.try_encode() {
                Ok(param) => param,
                Err(err) => {
                    error!("Failed to convert a param movetype value into movetype string export proto for converting to ExposedFunction: {:?}", param);
                    return Err(err.into())
                }
            });
        }

        let mut returns: Vec<String> = Vec::new();
        for r#return in self.r#return.iter() {
            returns.push(match r#return.try_encode() {
                Ok(ret) => ret,
                Err(err) => {
                    error!("Failed to convert a return movetype value into movetype string export proto for converting to ExposedFunction");
                    return Err(err.into());
                }
            });
        }

        Ok(ExposedFunction {
            name: self.name.into(),
            visibility: self.visibility.try_encode()?.into(),
            is_entry: self.is_entry,
            generic_type_params: gentypeparams,
            params,
            r#return: returns,
        })
    }
}

impl TryEncode<ExposedFunction> for Function {
    type Error = FunctionError;

    fn try_encode(&self) -> Result<ExposedFunction, Self::Error> {
        self.clone().try_into()
    }
}
