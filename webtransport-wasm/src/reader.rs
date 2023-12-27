use js_sys::Reflect;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStream, ReadableStreamDefaultReader, ReadableStreamReadResult};

use crate::WebError;

// Wrapper around ReadableStream which drops the lock on close.
pub struct Reader {
    inner: ReadableStreamDefaultReader,
}

impl Reader {
    pub fn new(stream: &ReadableStream) -> Result<Self, WebError> {
        let inner = stream.get_reader().dyn_into().unwrap();
        Ok(Self { inner })
    }

    pub async fn read<T: JsCast>(&mut self) -> Result<Option<T>, WebError> {
        let result: ReadableStreamReadResult =
            JsFuture::from(self.inner.read()).await?.dyn_into()?;

        if Reflect::get(&result, &"done".into())?.is_truthy() {
            return Ok(None);
        }

        let result = Reflect::get(&result, &"value".into())?.dyn_into()?;
        Ok(Some(result))
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.inner.release_lock();
    }
}
