use std::{fmt::Debug, rc::Rc};

use anyhow::{anyhow, Result};

use crate::environment::Environment;

use super::{ident::IdentData, Value};

#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub arity: usize,
    pub func: Rc<dyn Fn(&Environment, &[Value]) -> Result<()>>,
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Command<{}>", self.name).as_str())
    }
}

impl Command {
    pub fn run(&self, env: &Environment, args: &Vec<Value>) -> Result<()> {
        if args.len() != self.arity {
            return Err(anyhow!(
                "{} requires {} arguments, got {}",
                self.name,
                self.arity,
                args.len()
            ));
        }
        (self.func)(env, args)
    }
}

#[derive(Debug)]
pub struct CommandCall {
    pub command: String,
    pub args: Vec<Value>,
}

impl CommandCall {
    pub fn run(&self, env: &Environment) -> Result<()> {
        let r = env.get(&self.command);
        let res = r.ok_or_else(|| anyhow!("{} is not defined", self.command))?;
        if let IdentData::Command(cmd) = res {
            cmd.run(env, &self.args)
        } else {
            Err(anyhow!("{} is not a command", self.command))
        }
    }
}
