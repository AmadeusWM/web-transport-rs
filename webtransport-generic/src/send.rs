use bytes::Buf;

/// A trait describing the "send" actions of a QUIC stream.
#[async_trait::async_trait(?Send)]
pub trait SendStream {
    type Error: std::error::Error;

    async fn write<B: Buf>(&mut self, buf: &mut B) -> Result<(), Self::Error>;

    // Helper function that keeps calling Write until the buffer is empty
    async fn write_all<B: Buf>(&mut self, buf: &mut B) -> Result<(), Self::Error> {
        while buf.has_remaining() {
            self.write(buf).await?;
        }

        Ok(())
    }

    /// Send a QUIC reset code.
    ///
    /// If this is not called before Drop, the stream will terminate cleanly.
    async fn close(&mut self, code: u32) -> Result<(), Self::Error>;

    /// Set the stream's priority relative to other streams on the same connection.
    /// The **highest** priority stream with pending data will be sent first.
    /// Zero is the default value.
    async fn priority(&mut self, order: i32) -> Result<(), Self::Error>;
}
