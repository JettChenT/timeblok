use std::fs;
use chrono::NaiveDateTime;
use anyhow::{Result, anyhow};

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate core;

mod converter;
mod environment;
pub mod importer;
pub mod ir;
mod output;
mod parser;
mod preset;
mod resolver;
mod utils;
#[cfg(test)]
mod tests;

use ir::ExactDateTime;
use parser::{BlokParser, Rule};
use pest::Parser;

pub fn tb_to_records(tb: &String) -> Result<Vec<ir::Record>> {
    if let Some(parsed) = BlokParser::parse(Rule::FILE, tb)?.next(){
        let records = parser::parse_file(parsed)?;
        Ok(records)
    }else{
        Err(anyhow!("Could not parse file"))
    }
}

pub fn records_to_resolved(records: Vec<ir::Record>, base_time: ExactDateTime) -> Result<Vec<ir::ExactRecord>> {
    let resolved = resolver::resolve(records, base_time);
    Ok(resolved)
}

pub fn resolved_to_ical(resolved: Vec<ir::ExactRecord>) -> Result<String> {
    let ical = converter::to_ical(resolved, None);
    Ok(ical)
}

pub fn resolved_to_ical_deterministic(resolved: Vec<ir::ExactRecord>, base_time: ExactDateTime) -> Result<String> {
    let ical = converter::to_ical(resolved, Some(base_time.to_chrono()?));
    Ok(ical)
}

pub fn resolved_to_csv(resolved: Vec<ir::ExactRecord>) -> Result<String> {
    let csv = converter::to_csv(resolved)?;
    Ok(csv)
}

pub fn compile(source: &str, base_time: ExactDateTime) -> Result<String> {
    let records = tb_to_records(&source.to_string())?;
    let resolved = records_to_resolved(records, base_time)?;
    let ical = resolved_to_ical(resolved)?;
    Ok(ical)
}

pub fn compile_deterministic(source: &str, base_time: ExactDateTime) -> Result<String> {
    let records = tb_to_records(&source.to_string())?;
    let resolved = records_to_resolved(records, base_time.clone())?;
    let ical = resolved_to_ical_deterministic(resolved, base_time)?;
    Ok(ical)
}