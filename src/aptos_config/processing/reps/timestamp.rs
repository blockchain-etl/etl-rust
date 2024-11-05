//! Timestamp
use std::env;

use super::super::super::proto_codegen::aptos::common::UnixTimestamp;
use crate::blockchain_config::processing::traits::{Encode, TryEncode};
use aptos_protos::util::timestamp::Timestamp as InputTimestamp;
use chrono::{DateTime, Utc};
use log::{error, info};
use once_cell::sync::OnceCell;

/// The timestamp formatting
static TIMESTAMP_FORMAT: OnceCell<String> = OnceCell::new();
pub const TIMESTAMP_FORMAT_ENVKEY: &str = "APTOS_TIMESTAMP_OUTPUT";
pub const TIMESTAMP_FORMAT_DEFAULT: &str = "%Y-%m-%d %T";
pub type TimestampEncodedType = String;
pub const BIGQUERRY_MAX_TIMESTAMP_STRING: &str = "9999-12-31 23:59:59";
pub const BIGQUERRY_MAX_TIMESTAMP_SECONDS: i64 = 253_402_300_799;
pub const BIGQUERRY_MIN_TIMESTAMP_STRING: &str = "0001-01-01 00:00:01";
pub const BIGQUERRY_MIN_TIMESTAMP_SECONDS: i64 = -2_208_988_800;

/// Performs the initial timestamp format loading.  Will either pull the value
/// from the .env using dotenvy, or will use the default if
/// `TIMESTAMP_FORMAT_ENVKEY` value is not present in the .env file.
/// In the event that we fail to pull the timestamp format, we will return
/// a [TimestampError::FailedToLoadTimestampFormat]
fn load_timestamp_format() -> Result<&'static str, TimestampError> {
    match dotenvy::var(TIMESTAMP_FORMAT_ENVKEY) {
        Ok(ts_fmt) => {
            let _ = TIMESTAMP_FORMAT.set(ts_fmt);
            Ok(TIMESTAMP_FORMAT.get().expect("Known set"))
        }
        // Load the default value if it is not present in .env
        Err(dotenvy::Error::EnvVar(env::VarError::NotPresent)) => {
            info!(
                "[ENV] No envkey `{}`, using default `\"{}\"`",
                TIMESTAMP_FORMAT_ENVKEY, TIMESTAMP_FORMAT_DEFAULT
            );
            let _ = TIMESTAMP_FORMAT.set(String::from(TIMESTAMP_FORMAT_DEFAULT));
            Ok(TIMESTAMP_FORMAT.get().expect("Known set").as_str())
        }
        // Since EnvVar is clonable, we can return that err with it
        Err(dotenvy::Error::EnvVar(varerr @ env::VarError::NotUnicode(_))) => {
            error!(
                "[ENV] Non-Unicode .env value found while loading `{}`: `{:?}`",
                TIMESTAMP_FORMAT_ENVKEY, varerr
            );
            Err(TimestampError::FailedToLoadTimestampFormat(Some(varerr)))
        }
        // otherwise, log the error and then return None, as dotenvy errors are
        //      not clonable, thus not returnable in our error structure.
        Err(err) => {
            error!(
                "[ENV] Issue parsing .env file (dotenvy Error) while loading `{}`: ({})",
                TIMESTAMP_FORMAT_ENVKEY, err
            );
            Err(TimestampError::FailedToLoadTimestampFormat(None))
        }
    }
}

/// Returns the timestamp format.  On first call, will load the
/// `TIMESTAMP_FORMAT_ENVKEY` environment variable, otherwise will
/// use the default.  Returns a `TimestampError::FailedToLoadTimestampFormat`
/// if an error occurs while loading from .env file.
#[inline]
pub fn get_timestamp_format() -> Result<&'static str, TimestampError> {
    match TIMESTAMP_FORMAT.get() {
        Some(ts_fmt) => Ok(ts_fmt.as_str()),
        None => Ok(load_timestamp_format()?),
    }
}

#[derive(Debug, Clone)]
pub enum TimestampError {
    NegativeNano(i64, i32),
    OutOfRangeUtc(i64, u32),
    FailedToLoadTimestampFormat(Option<env::VarError>),
    OutOfRangeBigQuery(i64, u32),
}
impl std::fmt::Display for TimestampError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeNano(_, nanos) => write!(f, "Negative Nano: {}", nanos),
            Self::FailedToLoadTimestampFormat(Some(err)) => {
                write!(f, "Failed to load timestamp format due to error: {}", err)
            }
            Self::FailedToLoadTimestampFormat(None) => {
                write!(f, "Failed to load timestamp format due to unknown error")
            }
            Self::OutOfRangeUtc(seconds, nanos) => write!(
                f,
                "Out of range for UTC: (seconds: {}, nanos: {})",
                seconds, nanos
            ),
            Self::OutOfRangeBigQuery(seconds, nanos) => write!(
                f,
                "Out of range for BigQuerry: (seconds: {}, nanos: {})",
                seconds, nanos
            ),
        }
    }
}
impl std::error::Error for TimestampError {}

#[derive(Debug, Clone)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: u32,
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TimeStamp(seconds={}, nanos={})",
            self.seconds, self.nanos
        )
    }
}

impl TryFrom<InputTimestamp> for Timestamp {
    type Error = TimestampError;
    #[inline]
    fn try_from(value: InputTimestamp) -> Result<Self, Self::Error> {
        Ok(Timestamp {
            seconds: value.seconds,
            nanos: match value.nanos {
                n if n < 0 => return Err(TimestampError::NegativeNano(value.seconds, value.nanos)),
                n => n as u32,
            },
        })
    }
}

impl TryInto<DateTime<Utc>> for Timestamp {
    type Error = TimestampError;
    fn try_into(self) -> Result<DateTime<Utc>, Self::Error> {
        match DateTime::from_timestamp(self.seconds, self.nanos) {
            Some(dt) => Ok(dt),
            None => Err(TimestampError::OutOfRangeUtc(self.seconds, self.nanos)),
        }
    }
}

impl TryEncode<String> for Timestamp {
    type Error = TimestampError;
    /// Returns into the format specified in .env (or the default format)
    fn try_encode(&self) -> Result<String, Self::Error> {
        // Raise error if it is out of range, otherwise go ahead and make it a string.
        match self.seconds {
            n if n < BIGQUERRY_MIN_TIMESTAMP_SECONDS => {
                Err(TimestampError::OutOfRangeBigQuery(n, self.nanos))
            }
            n if n > BIGQUERRY_MAX_TIMESTAMP_SECONDS => {
                Ok(String::from(BIGQUERRY_MAX_TIMESTAMP_STRING))
            }
            _ => {
                let dt: DateTime<Utc> = self.clone().try_into()?;
                let timestamp_string = dt.format(get_timestamp_format()?).to_string();
                Ok(timestamp_string)
            }
        }
    }
}

impl Encode<UnixTimestamp> for Timestamp {
    /// Returns the UnixTimestamp proto message struct
    fn encode(&self) -> UnixTimestamp {
        UnixTimestamp {
            seconds: self.seconds,
            nanos: self.nanos,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Maximum BigQuery Timestamp:
    pub const MAX_BQ: Timestamp = Timestamp {
        seconds: BIGQUERRY_MAX_TIMESTAMP_SECONDS,
        nanos: 0,
    };

    /// Minimum BigQuery Timestamp:
    pub const MIN_BQ: Timestamp = Timestamp {
        seconds: BIGQUERRY_MIN_TIMESTAMP_SECONDS,
        nanos: 0,
    };

    /// Over Maximum BigQuery Timestamp:
    pub const OVER_MAX_BQ: Timestamp = Timestamp {
        seconds: BIGQUERRY_MAX_TIMESTAMP_SECONDS + 1,
        nanos: 0,
    };

    /// Under Minimum BigQuery Timestamp:
    pub const UNDER_MIN_BQ: Timestamp = Timestamp {
        seconds: BIGQUERRY_MIN_TIMESTAMP_SECONDS - 1,
        nanos: 0,
    };

    /// Ensure that maximum big querry value will succeed
    #[test]
    fn encode_maxbq() {
        match MAX_BQ.try_encode() {
            Ok(value) => info!(
                "Successfully encoded the maximum BQ value \"{}\" from {:?}",
                value, MAX_BQ
            ),
            Err(error) => panic!(
                "Failed to encode MAX BQ value `{:?}` due to {}",
                MAX_BQ, error
            ),
        }
    }

    /// Ensure that maximum big querry value will succeed
    #[test]
    fn encode_minbq() {
        match MIN_BQ.try_encode() {
            Ok(value) => info!(
                "Successfully encoded the minimum BQ value \"{}\" from {:?}",
                value, MIN_BQ
            ),
            Err(error) => panic!(
                "Failed to encode MIN BQ value `{:?}` due to {}",
                MIN_BQ, error
            ),
        }
    }

    /// Ensure that over maximum bigquery value will fail
    #[test]
    fn encode_overmaxbq() {
        match OVER_MAX_BQ.try_encode() {
            Ok(value) => assert_eq!(
                value, BIGQUERRY_MAX_TIMESTAMP_STRING,
                "If over the max bq, should be the BIGQUERY_MAX_TIMESTAMP_STRING"
            ),
            Err(error) => panic!("Caught an error when shouldn't have: {}", error),
        }
    }

    /// Ensure that under minimum bigquery value will fail
    #[test]
    fn encode_underminbq() {
        match UNDER_MIN_BQ.try_encode() {
            Ok(value) => panic!("Shouldn't have been able to convert time: \"{:?}\"", value),
            Err(error) => info!("Caught an error when should have: {}", error),
        }
    }
}
