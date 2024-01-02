use std::cmp;

use bytes::{BufMut, Bytes};
use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStreamDefaultReader, ReadableStreamReadResult, WebTransportReceiveStream};

use crate::WebError;

pub struct RecvStream {
    reader: ReadableStreamDefaultReader,
    buffer: Bytes,
}

impl RecvStream {
    pub fn new(stream: WebTransportReceiveStream) -> Result<Self, WebError> {
        if stream.locked() {
            return Err("locked".into());
        }

        let reader = stream.get_reader().unchecked_into();

        Ok(Self {
            reader,
            buffer: Bytes::new(),
        })
    }

    async fn read_chunk(&mut self, max: usize) -> Result<Bytes, WebError> {
        // Check if we need to read more data into the buffer.
        if self.buffer.len() == 0 {
            let result: ReadableStreamReadResult =
                JsFuture::from(self.reader.read()).await?.dyn_into()?;

            if Reflect::get(&result, &"done".into())?.is_truthy() {
                return Ok(Bytes::new()); // EOF
            }

            let result: Uint8Array = Reflect::get(&result, &"value".into())?.dyn_into()?;

            // TODO check if we're making a memory copy here
            self.buffer = result.to_vec().into();
        }

        let res = self.buffer.split_to(cmp::min(max, self.buffer.len()));
        Ok(res)
    }
}

#[async_trait::async_trait(?Send)]
impl webtransport_generic::RecvStream for RecvStream {
    type Error = WebError;

    async fn read<B: BufMut>(&mut self, buf: &mut B) -> Result<(), Self::Error> {
        let chunk = self.read_chunk(buf.remaining_mut()).await?;
        buf.put(chunk);
        Ok(())
    }

    async fn read_chunk(&mut self, max: usize) -> Result<Bytes, Self::Error> {
        self.read_chunk(max).await
    }

    async fn close(&mut self, code: u32) -> Result<(), Self::Error> {
        let code = code.into();
        JsFuture::from(self.reader.cancel_with_reason(&code)).await?;
        Ok(())
    }
}

impl Drop for RecvStream {
    fn drop(&mut self) {
        let _ = self.reader.cancel();
    }
}
