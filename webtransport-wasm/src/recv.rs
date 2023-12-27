use std::cmp;

use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use web_sys::WebTransportReceiveStream;

use crate::{Reader, WebError};

pub struct RecvStream {
    inner: WebTransportReceiveStream,
    reader: Reader,
    buf: Vec<u8>,
}

impl From<WebTransportReceiveStream> for RecvStream {
    fn from(inner: WebTransportReceiveStream) -> Self {
        let reader = Reader::new(&inner).unwrap();
        let buf = Vec::new();

        RecvStream { inner, reader, buf }
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::RecvStream for RecvStream {
    type Error = WebError;

    async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>, Self::Error> {
        if buf.is_empty() {
            return Ok(Some(0));
        }

        // Read more data into the internal buffer.
        // TODO use BYOB reader to avoid this when WebWorker support lands
        if self.buf.is_empty() {
            let data: Option<Uint8Array> = self.reader.read().await?;
            self.buf = match data {
                Some(data) => data.to_vec(),
                None => return Ok(None),
            };
        }

        // Return as much data as we can from the internal buffer.
        let size = cmp::min(buf.len(), self.buf.len());
        buf[..size].copy_from_slice(&self.buf[..size]);
        self.buf.drain(..size);

        Ok(Some(size))
    }

    async fn stop(&mut self, _code: u32) -> Result<(), Self::Error> {
        // TODO WebTransport doesn't support error codes?
        JsFuture::from(self.inner.cancel()).await?;
        Ok(())
    }
}

impl Drop for RecvStream {
    fn drop(&mut self) {
        let _ = self.inner.cancel();
    }
}
