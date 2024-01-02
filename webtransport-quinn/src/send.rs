use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use crate::SessionError;
use bytes::{Buf, Bytes};

/// A stream that can be used to send bytes. See [`quinn::SendStream`].
///
/// This wrapper is mainly needed for error codes, which is unfortunate.
/// WebTransport uses u32 error codes and they're mapped in a reserved HTTP/3 error space.
#[derive(Debug)]
pub struct SendStream {
    stream: quinn::SendStream,
}

impl SendStream {
    pub(crate) fn new(stream: quinn::SendStream) -> Self {
        Self { stream }
    }

    /// Abruptly reset the stream with the provided error code. See [`quinn::SendStream::reset`].
    /// This is a u32 with WebTransport because we share the error space with HTTP/3.
    pub fn reset(&mut self, code: u32) -> Result<(), StreamClosed> {
        let code = webtransport_proto::error_to_http3(code);
        let code = quinn::VarInt::try_from(code).unwrap();
        self.stream.reset(code).map_err(Into::into)
    }

    /// Wait until the stream has been stopped and return the error code. See [`quinn::SendStream::stopped`].
    /// Unlike Quinn, this returns None if the code is not a valid WebTransport error code.
    pub async fn stopped(&mut self) -> Result<Option<u32>, StoppedError> {
        let code = self.stream.stopped().await?;
        Ok(webtransport_proto::error_from_http3(code.into_inner()))
    }

    // Unfortunately, we have to wrap WriteError for a bunch of functions.

    /// Write some data to the stream, returning the size written. See [`quinn::SendStream::write`].
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, WriteError> {
        self.stream.write(buf).await.map_err(Into::into)
    }

    /// Write all of the data to the stream. See [`quinn::SendStream::write_all`].
    pub async fn write_all(&mut self, buf: &[u8]) -> Result<(), WriteError> {
        self.stream.write_all(buf).await.map_err(Into::into)
    }

    /// Write chunks of data to the stream. See [`quinn::SendStream::write_chunks`].
    pub async fn write_chunks(
        &mut self,
        bufs: &mut [Bytes],
    ) -> Result<quinn_proto::Written, WriteError> {
        self.stream.write_chunks(bufs).await.map_err(Into::into)
    }

    /// Write a chunk of data to the stream. See [`quinn::SendStream::write_chunk`].
    pub async fn write_chunk(&mut self, buf: Bytes) -> Result<(), WriteError> {
        self.stream.write_chunk(buf).await.map_err(Into::into)
    }

    /// Write all of the chunks of data to the stream. See [`quinn::SendStream::write_all_chunks`].
    pub async fn write_all_chunks(&mut self, bufs: &mut [Bytes]) -> Result<(), WriteError> {
        self.stream.write_all_chunks(bufs).await.map_err(Into::into)
    }

    /// Wait until all of the data has been written to the stream. See [`quinn::SendStream::finish`].
    pub async fn finish(&mut self) -> Result<(), WriteError> {
        self.stream.finish().await.map_err(Into::into)
    }

    pub fn set_priority(&self, order: i32) -> Result<(), StreamClosed> {
        self.stream.set_priority(order).map_err(Into::into)
    }

    pub fn priority(&self) -> Result<i32, StreamClosed> {
        self.stream.priority().map_err(Into::into)
    }
}

impl tokio::io::AsyncWrite for SendStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::SendStream for SendStream {
    type Error = WriteError;

    async fn write<B: Buf>(&mut self, buf: &mut B) -> Result<(), Self::Error> {
        let n = SendStream::write(self, buf.chunk()).await?;
        buf.advance(n);
        Ok(())
    }

    async fn close(&mut self, code: u32) -> Result<(), Self::Error> {
        SendStream::reset(self, code).map_err(|_| WriteError::Closed)
    }

    async fn priority(&mut self, order: i32) -> Result<(), Self::Error> {
        SendStream::set_priority(self, order).map_err(|_| WriteError::Closed)
    }
}

/// An error when writing to [`crate::SendStream`]. Similar to [`quinn::WriteError`].
#[derive(Clone, thiserror::Error, Debug)]
pub enum WriteError {
    #[error("STOP_SENDING: {0}")]
    Stopped(u32),

    #[error("invalid STOP_SENDING: {0}")]
    InvalidStopped(quinn::VarInt),

    #[error("session error: {0}")]
    SessionError(#[from] SessionError),

    #[error("stream closed")]
    Closed,
}

impl From<quinn::WriteError> for WriteError {
    fn from(e: quinn::WriteError) -> Self {
        match e {
            quinn::WriteError::Stopped(code) => {
                match webtransport_proto::error_from_http3(code.into_inner()) {
                    Some(code) => WriteError::Stopped(code),
                    None => WriteError::InvalidStopped(code),
                }
            }
            quinn::WriteError::UnknownStream => WriteError::Closed,
            quinn::WriteError::ConnectionLost(e) => WriteError::SessionError(e.into()),
            quinn::WriteError::ZeroRttRejected => unreachable!("0-RTT not supported"),
        }
    }
}

/// An error indicating the stream was already closed. Same as [`quinn::UnknownStream`] but a less confusing name.
#[derive(Clone, thiserror::Error, Debug)]
#[error("stream closed")]
pub struct StreamClosed;

impl From<quinn::UnknownStream> for StreamClosed {
    fn from(_: quinn::UnknownStream) -> Self {
        StreamClosed
    }
}

/// An error returned by [`crate::SendStream::stopped`]. Similar to [`quinn::StoppedError`].
#[derive(Clone, thiserror::Error, Debug)]
pub enum StoppedError {
    #[error("session error: {0}")]
    SessionError(#[from] SessionError),

    #[error("stream already closed")]
    Closed,
}

impl From<quinn::StoppedError> for StoppedError {
    fn from(e: quinn::StoppedError) -> Self {
        match e {
            quinn::StoppedError::ConnectionLost(e) => StoppedError::SessionError(e.into()),
            quinn::StoppedError::UnknownStream => StoppedError::Closed,
            quinn::StoppedError::ZeroRttRejected => unreachable!("0-RTT not supported"),
        }
    }
}
