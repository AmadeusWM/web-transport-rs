use bytes::Buf;
use js_sys::{Reflect, Uint8Array};
use wasm_bindgen_futures::JsFuture;
use web_sys::{WebTransportSendStream, WritableStreamDefaultWriter};

use crate::WebError;

pub struct SendStream {
    writer: WritableStreamDefaultWriter,
}

impl SendStream {
    pub fn new(stream: WebTransportSendStream) -> Result<Self, WebError> {
        let writer = stream.get_writer()?;
        Ok(Self { writer })
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<(), WebError> {
        let data = Uint8Array::from(buf);
        JsFuture::from(self.writer.write_with_chunk(&data)).await?;
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::SendStream for SendStream {
    type Error = WebError;

    async fn write<B: Buf>(&mut self, buf: &mut B) -> Result<(), Self::Error> {
        let chunk = buf.chunk();
        self.write(chunk).await?;
        buf.advance(chunk.len());
        Ok(())
    }

    async fn close(&mut self, code: u32) -> Result<(), Self::Error> {
        let reason = code.into();
        JsFuture::from(self.writer.abort_with_reason(&reason)).await?;
        Ok(())
    }

    async fn priority(&mut self, order: i32) -> Result<(), Self::Error> {
        Reflect::set(&self.writer, &"sendOrder".into(), &order.into())?;
        Ok(())
    }
}

impl Drop for SendStream {
    fn drop(&mut self) {
        let _ = self.writer.close();
    }
}
