#[macro_use] extern crate matches;
#[macro_use] extern crate static_assertions;
#[macro_use] extern crate log;

#[macro_use]
pub mod common;
pub mod syntax;
pub mod util;
pub mod ir;
pub mod vm;

use std::env::{self, Args};
use env_logger;
use crate::common::{
    FromPath,
};

fn exec(mut args: Args) -> Result<(), common::error::ProcessError> {
    let path = args.skip(1)
        .next()
        .unwrap();
    // TODO other args
    let ir_block = ir::Block::from_path(path)?;
    Ok(())
}

fn repl() {
    unimplemented!()
}

fn main() -> Result<(), common::error::ProcessError> {
    env_logger::init();
    let argv = env::args();
    if argv.len() < 2 {
        repl();
        Ok(())
    } else {
        Ok(exec(argv)?)
    }
}
