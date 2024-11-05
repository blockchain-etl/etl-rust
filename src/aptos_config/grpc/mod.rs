//! Provides the constants and functions code for
//! starting GRPC streams.
pub mod environment;
pub mod stream;

pub use stream::{get_stream, get_stream_response, StreamCreationError};
