use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use web_sys::{WritableStream, WritableStreamDefaultWriter};

use crate::WebError;

// Wrapper around WritableStream which drops the lock on close.
pub struct Writer {
    inner: WritableStreamDefaultWriter,
}

impl Writer {
    pub fn new(stream: &WritableStream) -> Result<Self, WebError> {
        let inner = stream.get_writer()?;
        Ok(Self { inner })
    }

    pub async fn write(&mut self, buf: &[u8]) -> Result<usize, WebError> {
        let data = Uint8Array::from(buf);
        JsFuture::from(self.inner.write_with_chunk(&data)).await?;
        Ok(buf.len())
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        self.inner.release_lock();
    }
}
