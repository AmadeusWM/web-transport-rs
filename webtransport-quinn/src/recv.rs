use std::{io, pin::Pin, slice, task};

use bytes::Bytes;

use crate::SessionError;

/// A stream that can be used to recieve bytes. See [`quinn::RecvStream`].
#[derive(Debug)]
pub struct RecvStream {
    inner: quinn::RecvStream,
}

impl RecvStream {
    pub(crate) fn new(stream: quinn::RecvStream) -> Self {
        Self { inner: stream }
    }

    /// Tell the other end to stop sending data with the given error code. See [`quinn::RecvStream::stop`].
    /// This is a u32 with WebTransport since it shares the error space with HTTP/3.
    pub fn stop(&mut self, code: u32) -> Result<(), quinn::UnknownStream> {
        let code = webtransport_proto::error_to_http3(code);
        let code = quinn::VarInt::try_from(code).unwrap();
        self.inner.stop(code)
    }

    // Unfortunately, we have to wrap ReadError for a bunch of functions.

    /// Read some data into the buffer and return the amount read. See [`quinn::RecvStream::read`].
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>, ReadError> {
        self.inner.read(buf).await.map_err(Into::into)
    }

    /// Fill the entire buffer with data. See [`quinn::RecvStream::read_exact`].
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ReadExactError> {
        self.inner.read_exact(buf).await.map_err(Into::into)
    }

    /// Read a chunk of data from the stream. See [`quinn::RecvStream::read_chunk`].
    pub async fn read_chunk(
        &mut self,
        max_length: usize,
        ordered: bool,
    ) -> Result<Option<quinn::Chunk>, ReadError> {
        self.inner
            .read_chunk(max_length, ordered)
            .await
            .map_err(Into::into)
    }

    /// Read chunks of data from the stream. See [`quinn::RecvStream::read_chunks`].
    pub async fn read_chunks(&mut self, bufs: &mut [Bytes]) -> Result<Option<usize>, ReadError> {
        self.inner.read_chunks(bufs).await.map_err(Into::into)
    }

    /// Read until the end of the stream or the limit is hit. See [`quinn::RecvStream::read_to_end`].
    pub async fn read_to_end(&mut self, size_limit: usize) -> Result<Vec<u8>, ReadToEndError> {
        self.inner.read_to_end(size_limit).await.map_err(Into::into)
    }

    // We purposely don't expose the stream ID or 0RTT because it's not valid with WebTransport
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::RecvStream for RecvStream {
    type Error = ReadError;

    async fn read<B: bytes::BufMut>(&mut self, buf: &mut B) -> Result<(), Self::Error> {
        while buf.has_remaining_mut() {
            let dst = buf.chunk_mut();
            let dst = unsafe { slice::from_raw_parts_mut(dst.as_mut_ptr(), dst.len()) };

            match self.read(dst).await? {
                Some(0) => panic!("read returned 0"),
                Some(n) => unsafe {
                    buf.advance_mut(n);
                },
                None => break,
            };
        }

        Ok(())
    }

    async fn read_chunk(&mut self, max: usize) -> Result<Bytes, Self::Error> {
        Ok(match self.read_chunk(max, true).await? {
            Some(chunk) => chunk.bytes,
            None => Bytes::new(),
        })
    }

    async fn close(&mut self, code: u32) -> Result<(), Self::Error> {
        self.stop(code).map_err(|_| ReadError::Closed)
    }
}

impl tokio::io::AsyncRead for RecvStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf,
    ) -> task::Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

/// An error when reading from [`crate::RecvStream`]. Similar to [`quinn::ReadError`].
#[derive(Clone, thiserror::Error, Debug)]
pub enum ReadError {
    #[error("session error: {0}")]
    SessionError(#[from] SessionError),

    #[error("RESET_STREAM: {0}")]
    Reset(u32),

    #[error("invalid RESET_STREAM: {0}")]
    InvalidReset(quinn::VarInt),

    #[error("stream already closed")]
    Closed,

    #[error("ordered read on unordered stream")]
    IllegalOrderedRead,
}

impl From<quinn::ReadError> for ReadError {
    fn from(value: quinn::ReadError) -> Self {
        match value {
            quinn::ReadError::Reset(code) => {
                match webtransport_proto::error_from_http3(code.into_inner()) {
                    Some(code) => ReadError::Reset(code),
                    None => ReadError::InvalidReset(code),
                }
            }
            quinn::ReadError::ConnectionLost(e) => ReadError::SessionError(e.into()),
            quinn::ReadError::IllegalOrderedRead => ReadError::IllegalOrderedRead,
            quinn::ReadError::UnknownStream => ReadError::Closed,
            quinn::ReadError::ZeroRttRejected => unreachable!("0-RTT not supported"),
        }
    }
}

/// An error returned by [`crate::RecvStream::read_exact`]. Similar to [`quinn::ReadExactError`].
#[derive(Clone, thiserror::Error, Debug)]
pub enum ReadExactError {
    #[error("finished early")]
    FinishedEarly,

    #[error("read error: {0}")]
    ReadError(#[from] ReadError),
}

impl From<quinn::ReadExactError> for ReadExactError {
    fn from(e: quinn::ReadExactError) -> Self {
        match e {
            quinn::ReadExactError::FinishedEarly => ReadExactError::FinishedEarly,
            quinn::ReadExactError::ReadError(e) => ReadExactError::ReadError(e.into()),
        }
    }
}

/// An error returned by [`crate::RecvStream::read_to_end`]. Similar to [`quinn::ReadToEndError`].
#[derive(Clone, thiserror::Error, Debug)]
pub enum ReadToEndError {
    #[error("too long")]
    TooLong,

    #[error("read error: {0}")]
    ReadError(#[from] ReadError),
}

impl From<quinn::ReadToEndError> for ReadToEndError {
    fn from(e: quinn::ReadToEndError) -> Self {
        match e {
            quinn::ReadToEndError::TooLong => ReadToEndError::TooLong,
            quinn::ReadToEndError::Read(e) => ReadToEndError::ReadError(e.into()),
        }
    }
}
