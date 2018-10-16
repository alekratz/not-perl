//#![feature(nll)]
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate enum_methods;
//#[macro_use] extern crate galvanic_test;

mod common;
pub mod syntax;
pub mod ir;
pub mod vm;
pub mod compile;

use std::{
    io::{self, Write},
    env::{self, Args},
    process,
};
use common::read_file;

fn exec(args: Args) -> Result<(), String> {
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
