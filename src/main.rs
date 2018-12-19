#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
extern crate failure;

#[macro_use]
pub mod common;
pub mod syntax;
pub mod util;
pub mod ir;

use std::{
    env::{self, Args},
    process,
};

fn exec(mut args: Args) -> Result<(), common::ProcessError> {
    unimplemented!()
}

fn repl() {
    unimplemented!()
}

fn main() {
    let argv = env::args();
    if argv.len() < 2 {
        repl();
    } else {
        if let Err(e) = exec(argv) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
