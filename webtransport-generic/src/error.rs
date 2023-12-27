use std::error::Error;

/// Trait that represent an error from the transport layer
pub trait ErrorCode: Error {
    /// Get the QUIC error code from CONNECTION_CLOSE
    fn code(&self) -> Option<u32>;
}
