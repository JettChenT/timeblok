use std::fmt::Debug;
use crate::environment::Environment;
use crate::ir::Date;
use crate::ir::filter::{BDF, Filter};

pub struct DynFilter<T> {
    pub filter: Box<dyn Fn(&T, &Environment) -> bool>,
    pub name: String,
}

impl<T> Debug for DynFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("DynFilter<{}>", self.name).as_str())
    }
}

impl<T:Debug> Filter<T> for DynFilter<T>{
    fn check(&self, value: &T, env: Option<&Environment>) -> bool {
        (self.filter)(value, env.unwrap())
    }
}


#[derive(Debug)]
pub enum Ident{
    Filter(BDF<Date>)
}