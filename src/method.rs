use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

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