//#![feature(nll)]
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate enum_methods;
extern crate failure;
//#[macro_use] extern crate failure_derive;
//#[macro_use] extern crate galvanic_test;

mod common;
mod util;
pub mod syntax;
pub mod ir;
pub mod vm;
pub mod compile;

use std::{
    //io::{self, Write},
    env::{self, Args},
    process,
};
use util::read_file;

fn exec(_args: Args) -> Result<(), String> {
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
