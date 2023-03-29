use anyhow::{anyhow, Result};

#[cfg(not(target_family = "wasm"))]
mod cli;
#[cfg(not(target_family = "wasm"))]
mod args;

#[cfg(not(target_family = "wasm"))]
fn main(){
    cli::main();
}

#[cfg(target_family = "wasm")]
fn main(){
    panic!("CLI not supported in WASM!")
}