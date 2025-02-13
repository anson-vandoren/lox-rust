mod ast_printer;
mod expr;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod token;
mod token_type;

use ast_printer::AstPrinter;
use parser::Parser;
use scanner::Scanner;
use snafu::prelude::*;
use std::{
    env,
    io::{self, Write},
    path::Path,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let code = match args.len() {
        l if l > 2 => {
            println!("Usage: rlox [script]");
            64
        }
        2 => run_file(&args[1]).unwrap_or_else(|e| {
            eprintln!("Failed to run file: {e}");
            65
        }),
        _ => run_prompt().unwrap_or_else(|e| {
            eprintln!("Error in REPL: {e}");
            65
        }),
    };
    std::process::exit(code);
}

fn run_file<T: AsRef<Path> + Into<String>>(script_path: T) -> Result<i32, LoxError> {
    let file = std::fs::read_to_string(&script_path).context(FileSnafu {
        path: script_path.into(),
    })?;
    run(file)?;
    Ok(0)
}

fn run_prompt() -> Result<i32, LoxError> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        stdout.write_all(b"> ").context(IoSnafu {})?;
        stdout.flush().context(IoSnafu {})?;
        let bytes_in = stdin.read_line(&mut buffer).context(IoSnafu {})?;
        match bytes_in {
            0 => return Ok(0),
            _ => {
                run(buffer.clone())?;
            }
        }
    }
}

fn run(script: String) -> Result<i32, LoxError> {
    let scanner = Scanner::new(script);
    let tokens = scanner.scan_tokens()?;
    //println!("{:?}", tokens);
    //for token in tokens.iter() {
    //    println!("{}", token);
    //}
    let mut parser = Parser::new(tokens);
    let printer = AstPrinter {};
    loop {
        let expr = parser.parse();
        match expr {
            Ok(e) => match e {
                Some(e) => println!("{}", printer.print(e)),
                None => {
                    println!("EOF");
                    break;
                }
            },
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }

    Ok(0)
}

#[derive(Debug, Snafu)]
pub enum LoxError {
    #[snafu(display("[line {line}] Error {whence}: {message}"))]
    Parsing {
        line: usize,
        whence: String,
        message: String,
    },
    #[snafu(display("IO error"))]
    Io { source: std::io::Error },
    #[snafu(display("Could not read source file at '{path}'"))]
    File {
        source: std::io::Error,
        path: String,
    },
    #[snafu(display("Fatal error, exiting"))]
    Fatal,
}
