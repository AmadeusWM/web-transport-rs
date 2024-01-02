use bytes::{BufMut, Bytes};
use std::error::Error;

/// A trait describing the "receive" actions of a QUIC stream.
#[async_trait::async_trait(?Send)]
pub trait RecvStream {
    type Error: Error;

    async fn read<B: BufMut>(&mut self, buf: &mut B) -> Result<(), Self::Error>;
    async fn read_chunk(&mut self, max: usize) -> Result<Bytes, Self::Error>;

    // Helper function to keep calling Read until the buffer is full
    async fn read_exact<B: BufMut>(
        &mut self,
        buf: &mut B,
    ) -> Result<(), ReadFullError<Self::Error>> {
        while buf.has_remaining_mut() {
            self.read(buf).await?;
        }

        Ok(())
    }

    /// Send a `STOP_SENDING` QUIC code.
    ///
    /// If this is not called before Drop, the code will be 0.
    async fn close(&mut self, code: u32) -> Result<(), Self::Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum ReadFullError<T: Error> {
    #[error("unexpected end")]
    UnexpectedEnd,

    #[error("{0}")]
    ReadError(#[from] T),
}
