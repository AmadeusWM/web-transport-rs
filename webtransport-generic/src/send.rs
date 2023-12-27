use crate::ErrorCode;

/// A trait describing the "send" actions of a QUIC stream.
#[async_trait::async_trait(?Send)]
pub trait SendStream {
    type Error: ErrorCode;

    /// Write the byte slice to the stream.
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Send a QUIC reset code.
    async fn reset(&mut self, code: u32) -> Result<(), Self::Error>;

    /// Set the stream's priority relative to other streams on the same connection.
    /// The **highest** priority stream with pending data will be sent first.
    /// Zero is the default value.
    fn priority(&mut self, order: i32) -> Result<(), Self::Error>;
}
