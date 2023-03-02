use crate::environment::Environment;
use crate::ir::filter::Filter;
use crate::ir::Date;

use std::fmt::Debug;
use std::rc::Rc;

use super::command::Command;
use super::Value;

pub struct DynFilter<T> {
    pub filter: Rc<dyn Fn(&T, Option<&Environment>) -> bool>,
    pub name: String,
}

impl<T> Clone for DynFilter<T> {
    fn clone(&self) -> Self {
        DynFilter {
            filter: self.filter.clone(),
            name: self.name.clone(),
        }
    }
}

impl<T> Debug for DynFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("DynFilter<{}>", self.name).as_str())
    }
}

impl<T: Debug + Clone> Filter<T> for DynFilter<T> {
    fn check(&self, value: &T, env: Option<&Environment>) -> bool {
        (self.filter)(value, env)
    }
}

#[derive(Debug, Clone)]
pub struct IdentFilter {
    pub ident: Ident,
}

impl Filter<Date> for IdentFilter {
    fn check(&self, value: &Date, env: Option<&Environment>) -> bool {
        match env.unwrap().get(&self.ident.name) {
            Some(IdentData::Value(Value::DateFilter(filt))) => filt.check(value, env),
            _ => {
                eprintln!(
                    "Warning: {} is not a date filter, returning false",
                    self.ident.name
                );
                false
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum IdentData {
    Value(Value),
    Command(Command),
}

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
}
