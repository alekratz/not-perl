//#![feature(nll)]
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate enum_methods;

mod common;
pub mod syntax;
pub mod ir;
pub mod vm;
pub mod repl;

use std::{
    io::{self, Write},
    env::{self, Args},
    process,
};
use common::read_file;
use ir::CompileState;
use repl::Repl;

fn exec(args: Args) -> Result<(), String> {
    let mut compiler = CompileState::new();
    compiler.begin();

    for filename in args.skip(1) {
        let contents = match read_file(&filename) {
            Ok(lexer) => lexer,
            Err(e) => {
                return Err(format!("could not read {}: {}", filename, e));
            }
        };
        if let Err(e) = compiler.feed_str(&filename, &contents) {
            return Err(format!("could not compile {}: {}", filename, e));
        }
    }
    let compile_unit = compiler.into_compile_unit();
    let mut vm = vm::Vm::new();
    if let Err(e) = vm.launch(compile_unit) {
        return Err(format!("VM runtime error: {}", e));
    }
    Ok(())
}

fn repl() {
    let mut repl = Repl::new();
    loop {
        let mut line = String::new();
        {
            let s = io::stdout();
            let mut stdout = s.lock();
            write!(stdout, " > ");
            stdout.flush().unwrap();
        }
        io::stdin().read_line(&mut line).unwrap();
        if line.len() == 0 {
            break;
        }
        match repl.execute_line(&line) {
            Ok(None) => {},
            Ok(Some(val)) => println!("{}", val.display_string()),
            Err(e) => {
                eprintln!("Error: {}", e);
                continue;
            },
        }
    }
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
