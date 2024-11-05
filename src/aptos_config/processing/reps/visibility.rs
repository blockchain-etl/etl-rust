use aptos_protos::transaction::v1 as input_protos;

use super::super::super::proto_codegen::aptos::common::Visibility as output_visibility;
use super::super::traits::TryEncode;

#[derive(Debug, Clone)]
pub enum VisibilityError {
    DecodeError(prost::DecodeError, i32),
    Unspecified,
}

#[derive(Debug, Clone)]
pub struct IntermediateVisibility {
    inner: input_protos::move_function::Visibility,
}

impl From<input_protos::move_function::Visibility> for IntermediateVisibility {
    fn from(value: input_protos::move_function::Visibility) -> Self {
        Self { inner: value }
    }
}

impl TryFrom<i32> for IntermediateVisibility {
    type Error = VisibilityError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match input_protos::move_function::Visibility::try_from(value) {
            Ok(visibility) => Ok(Self::from(visibility)),
            Err(decode_err) => Err(VisibilityError::DecodeError(decode_err, value)),
        }
    }
}

impl TryEncode<output_visibility> for IntermediateVisibility {
    type Error = VisibilityError;
    fn try_encode(&self) -> Result<output_visibility, Self::Error> {
        match self.inner {
            input_protos::move_function::Visibility::Friend => Ok(output_visibility::Friend),
            input_protos::move_function::Visibility::Private => Ok(output_visibility::Private),
            input_protos::move_function::Visibility::Public => Ok(output_visibility::Public),
            input_protos::move_function::Visibility::Unspecified => {
                Err(VisibilityError::Unspecified)
            }
        }
    }
}
