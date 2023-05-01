use std::{fmt::Debug, rc::Rc};

use anyhow::{anyhow, Result};

use crate::environment::Environment;
use crate::resolver::ResolverAction;

use super::{ident::IdentData, Value};

pub type CommandRes = Option<Vec<ResolverAction>>;

#[derive(Clone)]
pub struct Command {
    pub name: String,
    // An arity of 0 allows for an arbitrary amount of arguments
    pub arity: usize,
    pub func: Rc<dyn Fn(&Environment, &CommandCall) -> Result<CommandRes>>,
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Command<{}>", self.name).as_str())
    }
}

impl Command {
    pub fn run(&self, env: &Environment, call: &CommandCall) -> Result<CommandRes> {
        if self.arity>0 && call.args.len() != self.arity {
            return Err(anyhow!(
                "{} requires {} arguments, got {}",
                self.name,
                self.arity,
                call.args.len()
            ));
        }
        (self.func)(env, &call)
    }
}

#[derive(Debug)]
pub struct CommandCall {
    pub command: String,
    pub args: Vec<Value>,
    pub plain: String // Preserve the original argument string for custom parsing/error messages
}

impl CommandCall {
    pub fn run(&self, env: &Environment) -> Result<CommandRes> {
        let r = env.get(&self.command);
        let res = r.ok_or_else(|| anyhow!("{} is not defined", self.command))?;
        if let IdentData::Command(cmd) = res {
            cmd.run(env, self)
        } else {
            Err(anyhow!("{} is not a command", self.command))
        }
    }
}
