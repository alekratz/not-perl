#![feature(nll)]

pub mod syntax;
mod common;

use std::{
    env,
    process,
};

use syntax::{Lexer, Parser};
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
}
