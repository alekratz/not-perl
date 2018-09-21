//#![feature(nll)]
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate enum_methods;

mod common;
pub mod syntax;
pub mod ir;
pub mod vm;

use std::{
    env,
    process,
};

use syntax::{Lexer, Parser};
use ir::{IrTree, Ir, Compile};
use common::read_file;

fn main() {
    let mut argv = env::args();
    if argv.len() != 2 {
        println!("usage: {} filename", argv.next().unwrap());
        process::exit(1);
    }

    let filename = argv.nth(1).unwrap();

    let contents = match read_file(&filename) {
        Ok(lexer) => lexer,
        Err(e) => {
            eprintln!("could not read {}: {}", filename, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(contents.chars(), &filename);
    let parser = Parser::from_lexer(lexer);
    let tree = match parser.into_parse_tree() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("could not parse {}: {}", filename, e);
            process::exit(1);
        },
    };

    let ir_tree = IrTree::from_syntax(&tree);
    //let bytecode = ir_tree.compile_to_bytecode();
    let compiler = Compile::new();
    //println!("{:#?}", ir_tree);
    //println!("{:#?}", code);
    let compile_unit = match compiler.compile_ir_tree(&ir_tree) {
        Ok(cu) => cu,
        Err(e) => {
            eprintln!("could not compile {}: {}", filename, e);
            process::exit(1);
        },
    };
    let mut vm = vm::Vm::from_compile_unit(compile_unit);
    if let Err(e) = vm.launch() {
        eprintln!("VM runtime error in {}: {}", filename, e);
        process::exit(1);
    }
}
