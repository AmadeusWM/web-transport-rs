use std::{error, fmt};

use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct WebError {
    value: JsValue,
}

impl From<JsValue> for WebError {
    fn from(value: JsValue) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl error::Error for WebError {}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print out the JsValue as a string
        match self.value.as_string() {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "{:?}", self.value),
        }
    }
}

impl webtransport_generic::ErrorCode for WebError {
    fn code(&self) -> Option<u32> {
        None
    }
}
