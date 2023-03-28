use timeblok::{*, ir::ExactDateTime};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str, base_time: i64) -> Option<String> {
    let records = tb_to_records(&source.to_string()).unwrap();
    let resolved = records_to_resolved(records, ExactDateTime::from_timestamp(base_time).unwrap()).unwrap();
    let ical = resolved_to_ical(resolved).unwrap();
    Some(ical)
}