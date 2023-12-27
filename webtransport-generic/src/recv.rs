use crate::ErrorCode;

/// A trait describing the "receive" actions of a QUIC stream.
#[async_trait::async_trait(?Send)]
pub trait RecvStream {
    type Error: ErrorCode;

    async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>, Self::Error>;

    /// Send a `STOP_SENDING` QUIC code.
    async fn stop(&mut self, code: u32) -> Result<(), Self::Error>;
}
