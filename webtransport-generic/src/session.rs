use crate::ErrorCode;

use super::{RecvStream, SendStream};

/// Trait representing a WebTransport session.
///
/// The Session can be cloned to produce multiple handles and each method is &self, mirroing the Quinn API.
/// This is overly permissive, but otherwise Quinn would need an extra Arc<Mutex<Session>> wrapper which would hurt performance.
#[async_trait::async_trait(?Send)]
pub trait Session: Clone {
    type SendStream: SendStream;
    type RecvStream: RecvStream;
    type Error: ErrorCode;

    async fn accept_uni(&mut self) -> Result<Self::RecvStream, Self::Error>;
    async fn accept_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error>;
    async fn open_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error>;
    async fn open_uni(&mut self) -> Result<Self::SendStream, Self::Error>;

    async fn close(&mut self, code: u32, reason: &str) -> Result<(), Self::Error>;
    async fn closed(&self) -> Self::Error;
}
