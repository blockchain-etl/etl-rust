use super::super::traits::{FromVec, FromVecRef, TryEncode};

/// The expected prefix to be removed
const ADDRESS_PREFIX: &str = "0x";
/// The maximum address length
const ADDRESS_FORMATTED_LENGTH: usize = 64;
/// Number of attempts until we give up on fixing the address
const NUMBER_ATTEMPTS: usize = 10;

/// Address is an enum containing ways to express an Address.  Getting it into
/// this form allows running try_encode, which should encode it to our desired
/// outcome.
#[derive(Debug, Clone)]
pub enum Address {
    /// Address in Byte representation
    Bytes(Vec<u8>),
    /// Addres in [String] representation
    String(String),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytes(bytes) => write!(f, "AddressBytes({:?})", bytes),
            Self::String(string) => write!(f, "AddressString({})", string),
        }
    }
}

impl From<&str> for Address {
    fn from(value: &str) -> Self {
        Self::String(String::from(value))
    }
}

impl From<&String> for Address {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<&Vec<u8>> for Address {
    fn from(value: &Vec<u8>) -> Self {
        Self::Bytes(value.clone())
    }
}

impl<T> FromVec<T> for Address {}
impl<T> FromVecRef<T> for Address {}

impl Address {
    /// Returns an AddressErrorType if there is an issue.  Multiple errors
    /// may exist, and it is returned on the first error found.
    pub fn validate_string(string: &str) -> Result<(), AddressErrorType> {
        // Validate contains prefix
        if !string.starts_with(ADDRESS_PREFIX) {
            return Err(AddressErrorType::MissingPrefix);
        }

        match string.len() {
            n if n < ADDRESS_FORMATTED_LENGTH + ADDRESS_PREFIX.len() => {
                return Err(AddressErrorType::TooShort(n))
            }
            n if n > ADDRESS_FORMATTED_LENGTH + ADDRESS_PREFIX.len() => {
                return Err(AddressErrorType::TooLong(n))
            }
            _ => (),
        }

        // Validate all characters are ascii_hexdigit, if not gather the non-hex and return them
        if !Self::strip_prefix(string)
            .chars()
            .all(|c| char::is_ascii_hexdigit(&c))
        {
            return Err(AddressErrorType::NonHexChars(
                string.chars().filter(|&c| !c.is_ascii_hexdigit()).collect(),
            ));
        }

        Ok(())
    }

    /// Strips the prefix off a string
    fn strip_prefix(string: &str) -> &str {
        string.strip_prefix(ADDRESS_PREFIX).unwrap_or(string)
    }

    pub fn try_fix_string(string: &str) -> Result<String, AddressErrorType> {
        let mut address = String::from(string);
        for _ in 0..NUMBER_ATTEMPTS {
            match Self::validate_string(&address) {
                Ok(_) => return Ok(address),
                // We can fix this by adding the prefix.  We can also standardize the length since
                // we are already reformatting.
                Err(AddressErrorType::MissingPrefix) => {
                    address = format!("0x{:0>64}", address);
                }
                // We can fix too short by prefixing
                Err(AddressErrorType::TooShort(_)) => {
                    let prefixless = Self::strip_prefix(string);
                    address = format!("0x{:0>64}", prefixless);
                }
                // Any other issue we cannot fix, return failure
                Err(bad_error) => return Err(bad_error),
            }
        }
        unreachable!("Should have returned from inside the loop.");
    }
}

impl TryEncode<String> for Address {
    type Error = AddressError;
    /// Encodes the address as the string following our format.  
    fn try_encode(&self) -> Result<String, Self::Error> {
        match Self::try_fix_string(&match self {
            Self::Bytes(bytes) => hex::encode(bytes),
            Self::String(string) => string.clone(),
        }) {
            Ok(address) => Ok(address),
            Err(address_error_type) => Err(AddressError {
                address: self.clone(),
                error: address_error_type,
            }),
        }
    }
}

/// AddressError is the main error structure.  We use this
#[derive(Debug, Clone)]
pub struct AddressError {
    address: Address,
    error: AddressErrorType,
}

impl AddressError {
    pub fn error_type(&self) -> &AddressErrorType {
        &self.error
    }
}

impl std::fmt::Display for AddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddressError on \"{}\": {}", self.address, self.error)
    }
}

/// Represents the specific AddressErrorType
#[derive(Debug, Clone)]
pub enum AddressErrorType {
    MissingPrefix,
    TooLong(usize),
    TooShort(usize),
    NonHexChars(String),
}

impl std::fmt::Display for AddressErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrefix => write!(f, "Address missing prefix: \"{}\"", ADDRESS_PREFIX),
            Self::TooLong(length) => write!(f, "Address too long: {} chars", length),
            Self::TooShort(length) => write!(f, "Address too short: {} chars", length),
            Self::NonHexChars(nonhex) => {
                write!(f, "Address contains non-hex chars: \"{}\"", nonhex)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use log::debug;

    use super::*;

    const TEST_IN_ADDR1: &str = "0x000001";
    const TEST_OUT_ADDR1: &str =
        "0x0000000000000000000000000000000000000000000000000000000000000001";

    /// Validates whether or not we are standardizing [TEST_IN_ADDR1] into the expected [TEST_OUT_ADDR1]
    #[test]
    pub fn standardize_addr() {
        let addr = Address::from(TEST_IN_ADDR1);
        match addr.try_encode() {
            Ok(addr) => {
                debug!("Formatted TEST_IN_ADDR1 without error: {}", addr);
                assert_eq!(
                    addr, TEST_OUT_ADDR1,
                    "Formatted address does not match expected out address"
                );
            }
            Err(err) => panic!("Failed to format Addr `{}` due to: {}", addr, err),
        }
    }
}
