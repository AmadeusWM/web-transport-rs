use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    WebTransportBidirectionalStream, WebTransportCloseInfo, WebTransportReceiveStream,
    WebTransportSendStream,
};

use crate::{reader::Reader, RecvStream, SendStream, WebError};

#[derive(Clone)]
pub struct Session {
    inner: web_sys::WebTransport,
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::Session for Session {
    type SendStream = SendStream;
    type RecvStream = RecvStream;
    type Error = WebError;

    async fn accept_uni(&mut self) -> Result<Self::RecvStream, Self::Error> {
        let mut reader = Reader::new(&self.inner.incoming_unidirectional_streams())?;
        let stream: WebTransportReceiveStream = reader.read().await?.expect("closed without error");

        Ok(stream.into())
    }

    async fn accept_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error> {
        let mut reader = Reader::new(&self.inner.incoming_bidirectional_streams())?;
        let stream: WebTransportBidirectionalStream =
            reader.read().await?.expect("closed without error");

        Ok((stream.writable().into(), stream.readable().into()))
    }

    async fn open_bi(&mut self) -> Result<(Self::SendStream, Self::RecvStream), Self::Error> {
        let stream: WebTransportBidirectionalStream =
            JsFuture::from(self.inner.create_bidirectional_stream())
                .await?
                .dyn_into()?;

        Ok((stream.writable().into(), stream.readable().into()))
    }

    async fn open_uni(&mut self) -> Result<Self::SendStream, Self::Error> {
        let stream: WebTransportSendStream =
            JsFuture::from(self.inner.create_unidirectional_stream())
                .await?
                .dyn_into()?;

        Ok(stream.into())
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
