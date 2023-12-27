use js_sys::Reflect;
use wasm_bindgen_futures::JsFuture;
use web_sys::WebTransportSendStream;

use crate::{WebError, Writer};

pub struct SendStream {
    inner: WebTransportSendStream,
}

impl From<WebTransportSendStream> for SendStream {
    fn from(inner: WebTransportSendStream) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::SendStream for SendStream {
    type Error = WebError;

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut writer = Writer::new(&self.inner)?;
        writer.write(buf).await
    }

    async fn reset(&mut self, _code: u32) -> Result<(), Self::Error> {
        // TODO support error codes?
        JsFuture::from(self.inner.abort()).await?;
        Ok(())
    }

    fn priority(&mut self, order: i32) -> Result<(), Self::Error> {
        Reflect::set(&self.inner, &"sendOrder".into(), &order.into())?;
        Ok(())
    }
}

impl Drop for SendStream {
    fn drop(&mut self) {
        let _ = self.inner.close();
    }
}
