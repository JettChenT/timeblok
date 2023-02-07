use std::any::Any;
use std::fmt::Debug;
use crate::environment::Environment;
use crate::ir::Date;
use crate::ir::filter::{BDF, Filter};
use anyhow::Result;

pub struct DynFilter<T> {
    pub filter: Box<dyn Fn(&T, Option<&Environment>) -> bool>,
    pub name: String,
}

impl<T> Debug for DynFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("DynFilter<{}>", self.name).as_str())
    }
}

impl<T:Debug> Filter<T> for DynFilter<T>{
    fn check(&self, value: &T, env: Option<&Environment>) -> bool {
        (self.filter)(value, env)
    }
}

#[derive(Debug)]
pub struct IdentFilter {
    pub ident:Ident
}

impl Filter<Date> for IdentFilter{
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        match env.unwrap().get(&self.ident.name) {
            Some(IdentData::DateFilter(filt)) => filt.check(value, env),
            _ => {
                eprintln!("Warning: {} is not a date filter, returning false", self.ident.name);
                false
            }
        }
    }
}


#[derive(Debug)]
pub enum IdentData{
    DateFilter (BDF<Date>)
}

#[derive(Debug)]
pub struct Ident{
    pub name: String,
}