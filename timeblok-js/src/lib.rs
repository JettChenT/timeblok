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

#[wasm_bindgen]
pub fn compile_with_basedate(source: &str, year: i32, month: u32, day: u32) -> Option<String> {
    match timeblok::compile(source, ExactDateTime::from_ymd_hms(year, month, day,0,0,0)) {
        Ok(ics) => Some(ics),
        Err(_) => None,
    }
}