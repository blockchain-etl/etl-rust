use super::super::super::proto_codegen::aptos::signatures::signature::{
    public_key::PublicKeyType, PublicKey,
};
use super::super::traits::Encode;
use super::hashval::HashValue;
use aptos_protos::transaction::v1 as input_protos;

#[derive(Debug, Clone)]
pub enum PublicKeyError {
    Unspecified,
    UnmappedType(input_protos::any_public_key::Type),
}

#[derive(Debug, Clone)]
pub struct PublicKeyExtract {
    value: HashValue,
    r#type: input_protos::any_public_key::Type,
    index: Option<u32>,
}

impl std::fmt::Display for PublicKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unspecified => write!(f, "Unspecified public key type"),
            Self::UnmappedType(bad_type) => write!(f, "No mapped version for: {:?}", bad_type),
        }
    }
}

impl PublicKeyExtract {
    /// Converts the type enum from the aptos-protos type to our enum.  Raises an error if
    /// unspecified or if we do not have a mapping from their current types.
    pub fn convert_type(
        intype: input_protos::any_public_key::Type,
    ) -> Result<PublicKeyType, PublicKeyError> {
        match intype {
            input_protos::any_public_key::Type::Ed25519 => Ok(PublicKeyType::Ed25519),
            input_protos::any_public_key::Type::Secp256k1Ecdsa => Ok(PublicKeyType::Secp256k1Ecdsa),
            input_protos::any_public_key::Type::Keyless => Ok(PublicKeyType::Keyless),
            input_protos::any_public_key::Type::Secp256r1Ecdsa => Ok(PublicKeyType::Secp256r1Ecdsa),
            input_protos::any_public_key::Type::Unspecified => Err(PublicKeyError::Unspecified),
            #[allow(unreachable_patterns)]
            other => Err(PublicKeyError::UnmappedType(other)),
        }
    }
}

impl From<&input_protos::AnyPublicKey> for PublicKeyExtract {
    /// Extracts from AnyPublicKey
    #[inline]
    fn from(value: &input_protos::AnyPublicKey) -> Self {
        Self {
            value: (&value.public_key).into(),
            r#type: value.r#type(),
            index: None,
        }
    }
}

impl TryInto<PublicKey> for PublicKeyExtract {
    type Error = PublicKeyError;

    /// Turns PublicKey directly into the output PublicKey subrecord
    #[inline]
    fn try_into(self) -> Result<PublicKey, Self::Error> {
        Ok(PublicKey {
            r#type: Self::convert_type(self.r#type)?.into(),
            value: self.value.encode(),
            index: self.index,
        })
    }
}
