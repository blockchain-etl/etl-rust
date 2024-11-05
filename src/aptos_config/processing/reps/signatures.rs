use super::super::super::proto_codegen::aptos::signatures as sig_protos;
use super::super::deferred::{Deferred, DeferredError};
use super::super::traits::Encode;
use super::address::{Address, AddressError};
use super::hashval::HashValue;
use super::publickey::{PublicKeyError, PublicKeyExtract};
use super::sigvalue::{SigValueError, SigValueExtraction};
use aptos_protos::transaction::v1 as input_protos;
use aptos_protos::transaction::v1::signature::Type as SigType;

#[derive(Debug, Clone)]
pub enum SignatureError {
    MissingSignatureData,
    MissingAccountSignatureData,
    MultiEd25519LengthMismath(usize, usize, usize),
    SignatureValueIssue(SigValueError),
    MultiEd25519MultiplePubkeyIndexMatch(usize),
    MultiKeyMultipleSignatureIndexMatch(usize),
    MissingSingleSender,
    UnspecifiedAccountSignature,
    SingleKeySignatureMissingPubKey,
    SingleKeySignatureMissingSignature,
    UnspecifiedSignatureType,
    FeePayerMissingSender,
    FeePayerMissingFeePayer,
    DeferredError,
    AddressError(AddressError),
    MultiAgentMissingSender,
    PubKeyError(PublicKeyError),
}

impl From<SigValueError> for SignatureError {
    fn from(value: SigValueError) -> Self {
        Self::SignatureValueIssue(value)
    }
}

impl From<PublicKeyError> for SignatureError {
    fn from(value: PublicKeyError) -> Self {
        Self::PubKeyError(value)
    }
}

impl From<AddressError> for SignatureError {
    fn from(value: AddressError) -> Self {
        Self::AddressError(value)
    }
}

impl<T> From<DeferredError<T>> for SignatureError {
    fn from(_: DeferredError<T>) -> SignatureError {
        SignatureError::DeferredError
    }
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSignatureData => write!(f, "Missing Signature Data"),
            Self::MissingAccountSignatureData => write!(f, "Missing Account Signature Data"),
            Self::MultiEd25519LengthMismath(pubkeys, sigs, pubkeyindices) => {
                write!(f, "Not enough public key indicies ({}) for number of sigs ({}), cannot map the {} pubkeys.", 
                       pubkeyindices, sigs, pubkeys)
            }
            Self::SignatureValueIssue(err) => {
                write!(f, "Encountered issue with Signature Value: {}", err)
            }
            Self::MultiEd25519MultiplePubkeyIndexMatch(index) => {
                write!(
                    f,
                    "Multiple signatures pointed to same public key index: {}",
                    index
                )
            }
            Self::MissingSingleSender => write!(f, "SingleSender missing its inner-value"),
            Self::UnspecifiedAccountSignature => write!(f, "Unspecified Account Signature type"),
            Self::SingleKeySignatureMissingPubKey => {
                write!(f, "SingleKeySignature missing Public Key")
            }
            Self::SingleKeySignatureMissingSignature => {
                write!(f, "SingleKeySignature missing Signature value")
            }
            Self::MultiKeyMultipleSignatureIndexMatch(index) => write!(
                f,
                "MultiKeySignature has multiple signatures pointing to the same pubkey index: {}",
                index
            ),
            Self::DeferredError => write!(f, "Error occured with deferred values"),
            Self::FeePayerMissingSender => write!(f, "FeePayer is missing the Sender"),
            Self::FeePayerMissingFeePayer => write!(f, "FeePayer is missing the FeePayer"),
            Self::AddressError(err) => write!(f, "Had an issue with the Address: {}", err),
            Self::UnspecifiedSignatureType => write!(f, "Received an Unspecified signature type"),
            Self::MultiAgentMissingSender => write!(f, "MultiAgent missing Sender"),
            Self::PubKeyError(err) => write!(f, "Issue with the public key: {}", err),
        }
    }
}

// EXTRACTION

/// Data extracted from Signatures
#[derive(Debug, Clone)]
pub struct SignatureExtract {
    pub buildtype: SigType,
    pub signature_data: input_protos::signature::Signature,
}

impl TryFrom<input_protos::Signature> for SignatureExtract {
    type Error = SignatureError;
    fn try_from(value: input_protos::Signature) -> Result<Self, Self::Error> {
        Ok(SignatureExtract {
            buildtype: value.r#type(),
            signature_data: match value.signature {
                Some(sigdata) => sigdata,
                None => return Err(SignatureError::MissingSignatureData),
            },
        })
    }
}

/// Data extracted from AccountSignature
#[derive(Debug, Clone)]
pub struct AcctSigExtract {
    pub sigtype: input_protos::account_signature::Type,
    pub signature_data: input_protos::account_signature::Signature,
}

impl TryFrom<input_protos::AccountSignature> for AcctSigExtract {
    type Error = SignatureError;
    fn try_from(value: input_protos::AccountSignature) -> Result<Self, Self::Error> {
        Ok(AcctSigExtract {
            sigtype: value.r#type(),
            signature_data: match value.signature {
                Some(sigdata) => sigdata,
                None => return Err(SignatureError::MissingAccountSignatureData),
            },
        })
    }
}

// SUBRECORD

/// Simple enum to represent a Pending value.  This allows us
/// to start whether an item has been set or is
pub enum Pending<T> {
    Set(T),
    NotSet,
}

/// A subsection of the final Signature Record we wish to create.
#[derive(Debug, Clone)]
pub struct SignatureSubRecord {
    pub buildtype: Deferred<sig_protos::signature::SignatureBuildType>,
    pub public_key: sig_protos::signature::PublicKey,
    pub signature: Option<sig_protos::signature::Signature>,
    pub threshold: Option<u32>,
    pub signer: Deferred<Address>,
    pub is_secondary: Deferred<Option<bool>>,
    pub is_feepayer: Deferred<Option<bool>>,
    pub is_sender: Deferred<Option<bool>>,
}

// Here we will implement a lot of the functions required
// to convert various protobufs from Aptos into our
// SignatureSubRecord to be finalized later.
impl SignatureSubRecord {
    /// Creates subrecords from the main aptos protos Signature struct
    pub fn from_signature(
        value: input_protos::Signature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        match (value.r#type(), value.signature) {
            (
                input_protos::signature::Type::Ed25519,
                Some(input_protos::signature::Signature::Ed25519(data)),
            ) => Ok(vec![Self::from_ed25519(data)?]),
            (
                input_protos::signature::Type::MultiEd25519,
                Some(input_protos::signature::Signature::MultiEd25519(data)),
            ) => Self::from_multied25519(data),
            (
                input_protos::signature::Type::MultiAgent,
                Some(input_protos::signature::Signature::MultiAgent(data)),
            ) => Self::from_multiagentsignature(data),
            (
                input_protos::signature::Type::FeePayer,
                Some(input_protos::signature::Signature::FeePayer(data)),
            ) => Self::from_feepayersignature(data),
            (
                input_protos::signature::Type::SingleSender,
                Some(input_protos::signature::Signature::SingleSender(data)),
            ) => Self::from_singlesender(data),
            (input_protos::signature::Type::Unspecified, _) => {
                Err(SignatureError::UnspecifiedSignatureType)
            }
            (_, None) => Err(SignatureError::MissingSignatureData),
            (_, _) => Err(SignatureError::UnspecifiedSignatureType),
        }
    }

    /// Create SignatureSubRecord Vec from a MultiAgent
    pub fn from_multiagentsignature(
        value: input_protos::MultiAgentSignature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        // extract secondary records
        let mut subrecords = {
            // Create secondary records
            let mut subrecords: Vec<SignatureSubRecord> =
                Vec::with_capacity(value.secondary_signer_addresses.len());
            // Go through and create records for the signatures
            for (acctsig, addrstr) in value
                .secondary_signers
                .iter()
                .zip(value.secondary_signer_addresses.iter())
            {
                let address = Address::from(addrstr);
                for subrecord in Self::from_accountsignature(acctsig.clone())?.into_iter() {
                    subrecords.push(SignatureSubRecord {
                        buildtype: Deferred::DeferredFallback(
                            sig_protos::signature::SignatureBuildType::MultiAgent,
                        ),
                        signer: Deferred::Present(address.clone()),
                        is_secondary: Deferred::Present(Some(true)),
                        is_sender: Deferred::Present(Some(false)),
                        ..subrecord
                    })
                }
            }
            subrecords
        };

        // Add sender records
        match value.sender {
            Some(sender) => {
                for subrecord in Self::from_accountsignature(sender)? {
                    subrecords.push(SignatureSubRecord {
                        buildtype: Deferred::DeferredFallback(
                            sig_protos::signature::SignatureBuildType::MultiAgent,
                        ),
                        is_secondary: Deferred::Present(Some(false)),
                        is_sender: Deferred::Present(Some(true)),
                        ..subrecord
                    })
                }
            }
            None => return Err(SignatureError::MultiAgentMissingSender),
        };

        Ok(subrecords)
    }

    /// Create SignatureSubRecord Vec from a FeePayerSignature
    pub fn from_feepayersignature(
        value: input_protos::FeePayerSignature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        // Create sender records
        let sender_subrecords = {
            // Extract the sender
            let sender = match value.sender {
                Some(sender) => sender,
                None => return Err(SignatureError::FeePayerMissingSender),
            };
            // Build the subrecords
            let mut subrecords: Vec<SignatureSubRecord> = Self::from_accountsignature(sender)?;
            // Update them with Context from the FeePayer
            for subrecord in subrecords.iter_mut() {
                subrecord.buildtype =
                    Deferred::DeferredFallback(sig_protos::signature::SignatureBuildType::FeePayer);
                subrecord.is_feepayer = Deferred::Present(Some(false));
                subrecord.is_secondary = Deferred::Present(Some(false));
                subrecord.is_sender = Deferred::Present(Some(true));
            }
            subrecords
        };

        let feepayer_subrecords = {
            // Extract feepayer
            let feepayer = match value.fee_payer_signer {
                Some(feepayer) => feepayer,
                None => return Err(SignatureError::FeePayerMissingFeePayer),
            };
            // Build the subrecords
            let mut subrecords: Vec<SignatureSubRecord> = Self::from_accountsignature(feepayer)?;
            // Update them with context from the feepayer
            for subrecord in subrecords.iter_mut() {
                subrecord.buildtype =
                    Deferred::DeferredFallback(sig_protos::signature::SignatureBuildType::FeePayer);
                subrecord.signer = Deferred::Present(Address::from(&value.fee_payer_address));
                subrecord.is_feepayer = Deferred::Present(Some(true));
                subrecord.is_secondary = Deferred::Present(Some(false));
                subrecord.is_sender = Deferred::Present(Some(false));
            }
            subrecords
        };

        // Extract secondary records
        let secondary_subrecords = {
            let mut subrecords: Vec<SignatureSubRecord> =
                Vec::with_capacity(value.secondary_signers.len());
            for (acctsig, addr_string) in value
                .secondary_signers
                .iter()
                .zip(value.secondary_signer_addresses.iter())
            {
                let address = Address::from(addr_string);
                for cur_subrecord in Self::from_accountsignature(acctsig.clone())?.into_iter() {
                    subrecords.push(SignatureSubRecord {
                        buildtype: Deferred::DeferredFallback(
                            sig_protos::signature::SignatureBuildType::FeePayer,
                        ),
                        signer: Deferred::Present(address.clone()),
                        is_feepayer: Deferred::Present(Some(false)),
                        is_secondary: Deferred::Present(Some(true)),
                        is_sender: Deferred::Present(Some(false)),
                        ..cur_subrecord
                    })
                }
            }
            subrecords
        };
        Ok([sender_subrecords, feepayer_subrecords, secondary_subrecords].concat())
    }

    /// Creates a singular SignatureSubRecord from an Ed25519Signature
    pub fn from_ed25519(
        value: input_protos::Ed25519Signature,
    ) -> Result<SignatureSubRecord, SignatureError> {
        Ok(SignatureSubRecord {
            buildtype: Deferred::DeferredFallback(
                sig_protos::signature::SignatureBuildType::Ed25519,
            ),
            public_key: sig_protos::signature::PublicKey {
                r#type: sig_protos::signature::signature::SignatureType::Ed25519.into(),
                value: HashValue::from(value.public_key.clone()).encode(),
                index: None,
            },
            signature: Some(sig_protos::signature::Signature {
                r#type: sig_protos::signature::signature::SignatureType::Ed25519.into(),
                value: HashValue::from(value.signature.clone()).encode(),
                index: None,
            }),
            threshold: None,
            signer: Deferred::Deferred,
            is_feepayer: Deferred::Present(None),
            is_secondary: Deferred::Present(None),
            is_sender: Deferred::Present(None),
        })
    }

    // Creates a

    /// Creates a vector of signaturesubrecords from
    /// a multied25519 signature
    pub fn from_multied25519(
        value: input_protos::MultiEd25519Signature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        // Verify lengths match
        if value.signatures.len() != value.public_key_indices.len() {
            return Err(SignatureError::MultiEd25519LengthMismath(
                value.public_keys.len(),
                value.signatures.len(),
                value.public_key_indices.len(),
            ));
        }

        // Creates a mapping for index -> Signature value
        let mut sigs: std::collections::HashMap<usize, sig_protos::signature::Signature> = {
            let mut sigs = std::collections::HashMap::with_capacity(value.signatures.len());
            for (sig, index) in value.signatures.iter().zip(value.public_key_indices.iter()) {
                // Create a Signature Value
                let sigval = sig_protos::signature::Signature {
                    r#type: sig_protos::signature::signature::SignatureType::Ed25519.into(),
                    value: HashValue::from(sig).encode(),
                    index: Some(*index),
                };
                // Add to the hashmap
                match sigs.insert(*index as usize, sigval) {
                    None => {}
                    // If Some, the value was already added
                    Some(_) => {
                        return Err(SignatureError::MultiEd25519MultiplePubkeyIndexMatch(
                            *index as usize,
                        ))
                    }
                }
            }
            sigs
        };

        // Create subrecords by combining the pubkeys mapped in sigs or by
        // using an empty signature if a pubkey had no mapped signature.
        let records = {
            let mut records: Vec<SignatureSubRecord> = Vec::with_capacity(value.public_keys.len());
            for (index, pubkey) in value.public_keys.iter().enumerate() {
                records.push(SignatureSubRecord {
                    buildtype: Deferred::DeferredFallback(
                        sig_protos::signature::SignatureBuildType::MultiEd25519,
                    ),
                    public_key: sig_protos::signature::PublicKey {
                        r#type: sig_protos::signature::public_key::PublicKeyType::Ed25519.into(),
                        value: HashValue::from(pubkey).encode(),
                        index: Some(index as u32),
                    },
                    signature: sigs.remove(&index),
                    threshold: Some(value.threshold),
                    signer: Deferred::Deferred,
                    is_secondary: Deferred::Present(None),
                    is_feepayer: Deferred::Present(None),
                    is_sender: Deferred::Present(None),
                })
            }
            records
        };
        Ok(records)
    }

    pub fn from_singlekeysignature(
        value: input_protos::SingleKeySignature,
    ) -> Result<SignatureSubRecord, SignatureError> {
        // Extract the potentially missing values
        let (pubkey, sig) = match (value.public_key, value.signature) {
            (Some(pubkey), Some(sig)) => (
                PublicKeyExtract::from(&pubkey),
                SigValueExtraction::try_from(&sig)?,
            ),
            (None, _) => return Err(SignatureError::SingleKeySignatureMissingPubKey),
            (_, None) => return Err(SignatureError::SingleKeySignatureMissingSignature),
        };

        Ok(SignatureSubRecord {
            buildtype: Deferred::Deferred,
            public_key: pubkey.try_into()?,
            signature: Some(sig.try_into()?),
            threshold: None,
            signer: Deferred::Deferred,
            is_secondary: Deferred::Present(None),
            is_feepayer: Deferred::Present(None),
            is_sender: Deferred::Present(None),
        })
    }

    /// Returns a vector of SignatureSubRecords from multikeysignature
    pub fn from_multikeysignature(
        value: input_protos::MultiKeySignature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        let subrecords = {
            // Stores records we are creating
            let mut subrecords: Vec<SignatureSubRecord> =
                Vec::with_capacity(value.public_keys.len());

            // Store mappings of signatures to pubkeys
            let mut sigs: std::collections::HashMap<usize, sig_protos::signature::Signature> =
                std::collections::HashMap::with_capacity(value.signatures.len());
            for sig in value.signatures.iter() {
                let index = sig.index as usize;
                match sigs.insert(index, (SigValueExtraction::try_from(sig)?).try_into()?) {
                    None => {}
                    Some(_) => {
                        return Err(SignatureError::MultiKeyMultipleSignatureIndexMatch(index));
                    }
                }
            }

            for (index, pubkey) in value.public_keys.iter().enumerate() {
                subrecords.push(SignatureSubRecord {
                    buildtype: Deferred::Deferred,
                    public_key: PublicKeyExtract::from(pubkey).try_into()?,
                    signature: sigs.remove(&index),
                    threshold: Some(value.signatures_required),
                    signer: Deferred::Deferred,
                    is_secondary: Deferred::Present(None),
                    is_feepayer: Deferred::Present(None),
                    is_sender: Deferred::Present(None),
                })
            }
            subrecords
        };

        Ok(subrecords)
    }

    /// Given an AccountSignature, returns SignatureSubRecords
    pub fn from_accountsignature(
        value: input_protos::AccountSignature,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        match (value.r#type(), value.signature) {
            (
                input_protos::account_signature::Type::Ed25519,
                Some(input_protos::account_signature::Signature::Ed25519(sigdata)),
            ) => Ok(vec![Self::from_ed25519(sigdata)?]),
            (
                input_protos::account_signature::Type::MultiEd25519,
                Some(input_protos::account_signature::Signature::MultiEd25519(sigdata)),
            ) => Self::from_multied25519(sigdata),
            (
                input_protos::account_signature::Type::MultiKey,
                Some(input_protos::account_signature::Signature::MultiKeySignature(sigdata)),
            ) => Self::from_multikeysignature(sigdata),
            (
                input_protos::account_signature::Type::SingleKey,
                Some(input_protos::account_signature::Signature::SingleKeySignature(sigdata)),
            ) => Ok(vec![Self::from_singlekeysignature(sigdata)?]),
            (input_protos::account_signature::Type::Unspecified, _) => {
                Err(SignatureError::UnspecifiedAccountSignature)
            }
            (_, None) => Err(SignatureError::MissingAccountSignatureData),
            (_, _) => Err(SignatureError::UnspecifiedAccountSignature),
        }
    }

    /// Given a SingleSender, returns SignatureSubRecords
    pub fn from_singlesender(
        value: input_protos::SingleSender,
    ) -> Result<Vec<SignatureSubRecord>, SignatureError> {
        match value.sender {
            Some(single_sender) => {
                let subrecords = Self::from_accountsignature(single_sender)?;
                let mut outsubrecords = Vec::with_capacity(subrecords.len());
                for subrecord in subrecords.into_iter() {
                    outsubrecords.push(SignatureSubRecord {
                        buildtype: Deferred::DeferredFallback(
                            sig_protos::signature::SignatureBuildType::SingleSender,
                        ),
                        ..subrecord
                    })
                }
                Ok(outsubrecords)
            }
            None => Err(SignatureError::MissingSingleSender),
        }
    }
}
