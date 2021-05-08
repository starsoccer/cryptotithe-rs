use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Method {
    FIFO = "FIFO",
    LIFO = "LIFO",
    HCFO = "HCFO",
    LCFO = "LCFO",
    LTFO = "LTFO",
    HTFO = "HTFO",
}
