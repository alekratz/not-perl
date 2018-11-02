#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate enum_methods;
extern crate failure;
//#[macro_use] extern crate failure_derive;
//#[macro_use] extern crate galvanic_test;

#[macro_use] pub mod common;
pub mod syntax;
pub mod ir;
pub mod vm;
pub mod compile;
pub mod util;

use std::{
    env::{self, Args},
    process,
};

fn exec(mut args: Args) -> Result<(), common::ProcessError> {
    args.next().expect("exec() must be called with at least 2 args");
    let path = args.next()
        .expect("exec() must be called with at least 2 args");
    let args: Vec<_> = args.collect();
    let mut compile = compile::Compile::new();
    compile.update_from_path(&path)
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
