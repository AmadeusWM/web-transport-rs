use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    WebTransportBidirectionalStream, WebTransportCloseInfo, WebTransportReceiveStream,
    WebTransportSendStream,
};

use crate::{Reader, RecvStream, SendStream, WebError};

#[derive(Clone)]
pub struct Session {
    inner: web_sys::WebTransport,
}

impl Session {
    pub async fn new(url: &str) -> Result<Self, WebError> {
        let inner = web_sys::WebTransport::new(url)?;
        JsFuture::from(inner.ready()).await?;

        Ok(Self { inner })
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::Session for Session {
    type SendStream = SendStream;
    type RecvStream = RecvStream;
    type Error = WebError;

    async fn accept_uni(&mut self) -> Result<Self::RecvStream, Self::Error> {
        let mut reader = Reader::new(self.inner.incoming_unidirectional_streams())?;
        let stream: WebTransportReceiveStream = reader.read().await?.expect("closed without error");
        let recv = RecvStream::new(stream)?;
        Ok(recv)
    }

    async fn accept_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error> {
        let mut reader = Reader::new(self.inner.incoming_bidirectional_streams())?;
        let stream: WebTransportBidirectionalStream =
            reader.read().await?.expect("closed without error");

        let send = SendStream::new(stream.writable())?;
        let recv = RecvStream::new(stream.readable())?;

        Ok((send, recv))
    }

    async fn open_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error> {
        let stream: WebTransportBidirectionalStream =
            JsFuture::from(self.inner.create_bidirectional_stream())
                .await?
                .dyn_into()?;

        let send = SendStream::new(stream.writable())?;
        let recv = RecvStream::new(stream.readable())?;

        Ok((send, recv))
    }

    async fn open_uni(&mut self) -> Result<Self::SendStream, Self::Error> {
        let stream: WebTransportSendStream =
            JsFuture::from(self.inner.create_unidirectional_stream())
                .await?
                .dyn_into()?;

        let send = SendStream::new(stream)?;
        Ok(send)
    }

    async fn close(&mut self, code: u32, reason: &str) -> Result<(), Self::Error> {
        let mut info = WebTransportCloseInfo::new();
        info.close_code(code);
        info.reason(reason);
        self.inner.close_with_close_info(&info);
        Ok(())
    }

    async fn closed(&self) -> Self::Error {
        let err = JsFuture::from(self.inner.closed()).await.unwrap();
        WebError::from(err)
    }
}
