use pyo3::prelude::*;
use ::timeblok::compile;
use ::timeblok::ir::{ExactDateTime, Event, ExactRecord, ExactRange, TimeZoneChoice};

#[pyfunction]
fn compile_with_basedate(source: &str, year: i32, month: u32, day: u32) -> Option<String> {
    match compile(source, ExactDateTime::from_ymd_hms(year, month, day,0,0,0)) {
        Ok(ics) => Some(ics),
        Err(_) => None,
    }
}

#[pymodule]
fn timeblok_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_with_basedate, m)?)?;
    Ok(())
}