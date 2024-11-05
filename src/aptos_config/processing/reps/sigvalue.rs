//! This module handles representing signatures values, a term
//! used just to differentiate between the concept of Signature and
//! its related data and the actual byte data that makes up the
//! signature value.
use super::super::super::proto_codegen::aptos::signatures::signature::{
    signature::SignatureType as SigType, Signature as SigValue,
};
use aptos_protos::transaction::v1 as input_protos;
use aptos_protos::transaction::v1::any_signature::SignatureVariant;

use super::super::traits::Encode;
use super::hashval::HashValue;

/// Error due to issues with the True Signature
#[derive(Debug, Clone)]
pub enum SigValueError {
    Unspecified,
    IndexedSigMissingSigData,
    UnmappedType(input_protos::any_signature::Type),
    UnsupportedSignatureVariant,
}

impl std::fmt::Display for SigValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unspecified => write!(f, "Unspecified Signature Value"),
            Self::IndexedSigMissingSigData => {
                write!(f, "Missing AnySignature value from IndexedSignature")
            }
            Self::UnmappedType(sigtype) => write!(f, "Unmapped Signature Type: {:?}", sigtype),
            Self::UnsupportedSignatureVariant => {
                write!(f, "Unsupported SignatureVariant, review supported protos")
            }
        }
    }
}

impl std::error::Error for SigValueError {}

/// Extracted Signature Value
#[derive(Debug, Clone)]
pub struct SigValueExtraction {
    pub value: HashValue,
    pub sigvaltype: input_protos::any_signature::Type,
    pub index: Option<u32>,
}

impl SigValueExtraction {
    pub fn convert_type(
        value: input_protos::any_signature::Type,
    ) -> Result<SigType, SigValueError> {
        match value {
            input_protos::any_signature::Type::Ed25519 => Ok(SigType::Ed25519),
            input_protos::any_signature::Type::Secp256k1Ecdsa => Ok(SigType::Secp256k1Ecdsa),
            input_protos::any_signature::Type::Keyless => Ok(SigType::Keyless),
            input_protos::any_signature::Type::Webauthn => Ok(SigType::Webauthn),
            input_protos::any_signature::Type::Unspecified => Err(SigValueError::Unspecified),
            #[allow(unreachable_patterns)] // Future proofing for protobuf updates
            other => Err(SigValueError::UnmappedType(other)),
        }
    }
}

impl TryFrom<&input_protos::AnySignature> for SigValueExtraction {
    type Error = SigValueError;
    fn try_from(value: &input_protos::AnySignature) -> Result<Self, Self::Error> {
        Ok(Self {
            value: match &value.signature_variant {
                Some(SignatureVariant::Ed25519(ed25519)) => ed25519.signature.clone().into(),
                Some(SignatureVariant::Keyless(keyless)) => keyless.signature.clone().into(),
                Some(SignatureVariant::Secp256k1Ecdsa(secp256k1ecdsa)) => {
                    secp256k1ecdsa.signature.clone().into()
                }
                Some(SignatureVariant::Webauthn(webauth)) => webauth.signature.clone().into(),
                None => return Err(SigValueError::UnsupportedSignatureVariant),
            },
            sigvaltype: value.r#type(),
            index: None,
        })
    }
}

impl TryFrom<&input_protos::IndexedSignature> for SigValueExtraction {
    type Error = SigValueError;
    fn try_from(value: &input_protos::IndexedSignature) -> Result<Self, Self::Error> {
        Ok(Self {
            index: Some(value.index),
            ..match &value.signature {
                Some(anysig) => Self::try_from(anysig)?,
                None => return Err(SigValueError::IndexedSigMissingSigData),
            }
        })
    }
}

impl TryInto<SigValue> for SigValueExtraction {
    type Error = SigValueError;
    fn try_into(self) -> Result<SigValue, Self::Error> {
        Ok(SigValue {
            r#type: match self.sigvaltype {
                input_protos::any_signature::Type::Unspecified => {
                    return Err(SigValueError::Unspecified)
                }
                sigvaltype => SigValueExtraction::convert_type(sigvaltype)?.into(),
            },
            value: self.value.encode(),
            index: self.index,
        })
    }
}
