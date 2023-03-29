use timeblok::ir::ExactDateTime;
use wasm_bindgen::prelude::*;
mod utils;

#[wasm_bindgen]
pub fn compile(source: &str, base_time: i64) -> Option<String> {
    match timeblok::compile(source, ExactDateTime::from_timestamp(base_time)?) {
        Ok(ics) => Some(ics),
        Err(_) => None,
    }
}