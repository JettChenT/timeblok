use std::fmt::Debug;

use anyhow::{Result, anyhow};

use crate::environment::Environment;

pub struct Command{
    pub name: String,
    pub arity: usize,
    pub func: Box<dyn Fn(&mut Environment, &[String]) -> Result<()>>,
}

impl Debug for Command{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Command<{}>", self.name).as_str())
    }
}

impl Command{
    pub fn run(&self, env: &mut Environment, args: Vec<String>) -> Result<()>{
        if args.len() != self.arity {
            return Err(anyhow!("{} requires {} arguments, got {}", self.name, self.arity, args.len()));
        }
        (self.func)(env, &args)
    }
}