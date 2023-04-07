use timeblok::ir::ExactDateTime;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
mod utils;
extern crate web_sys;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

macro_rules! log_debug {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!("{:?}", $( $t )* ).into());
    }
}


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

#[wasm_bindgen]
pub fn compile_verbose(source: &str, year: i32, month: u32, day: u32) -> Option<String> {
    let records = timeblok::tb_to_records(&source.to_string()).ok()?;
    log_debug!(records);
    let resolved = timeblok::records_to_resolved(records, ExactDateTime::from_ymd_hms(year, month, day,0,0,0)).ok()?;
    log_debug!(resolved);
    let ics = timeblok::resolved_to_ical(resolved).ok()?;
    Some(ics)
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
pub fn test_compile() {
    use timeblok::importer::*;
    let res = compile_with_basedate("2023-4-\n{mon}\n10am do stuff", 2023, 4, 7);
    assert!(res.is_some());
    if let Some(ref icsdat) = res {
        let resolved = ics_to_records(&import_ics(icsdat).unwrap());
        assert!(resolved.len()==4);
    }
}