mod ast_printer;
mod expr;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod stmt;
mod token;
mod token_type;

use ast_printer::AstPrinter;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use snafu::prelude::*;
use std::{
    env,
    io::{self, Write},
    path::Path,
};
use token::Token;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    let code = match args.len() {
        len if len > 2 => {
            println!("Usage: rlox [script]");
            64
        }
        2 => lox.run_file(&args[1]),
        _ => lox.run_prompt(),
    };
    std::process::exit(code);
}

struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Self {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file<T: AsRef<Path> + Into<String>>(&mut self, script_path: T) -> i32 {
        let file = std::fs::read_to_string(&script_path)
            .context(FileSnafu {
                path: script_path.into(),
            })
            .expect("Cannot read file");

        match self.run(file) {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("Failed to run file: {e}");
                if self.had_error {
                    65
                } else if self.had_runtime_error {
                    70
                } else {
                    panic!("Error but no error...")
                }
            }
        }
    }

    pub fn run_prompt(&mut self) -> i32 {
        let mut rl = rustyline::DefaultEditor::new().expect("Could not build REPL");
        loop {
            match rl.readline("> ") {
                Err(_) => return 0,
                Ok(line) => {
                    let _ = rl.add_history_entry(&line);
                    let _ = self.run(line).inspect_err(|e| {
                        eprintln!("{}", e);
                    });
                }
            }
        }
    }

    fn run(&mut self, script: String) -> Result<()> {
        let scanner = Scanner::new(script);
        let tokens = scanner.scan_tokens().inspect_err(|_| {
            self.had_error = true;
        })?;
        let mut parser = Parser::new(tokens);
        let printer = AstPrinter {};
        loop {
            let expr = parser.parse();
            match expr {
                Ok(e) => match e {
                    Some(expr) => {
                        //println!("{}", printer.print(expr));
                        self.interpreter.interpret(expr).inspect_err(|_| {
                            self.had_runtime_error = true;
                        })?;
                    }
                    None => {
                        break;
                    }
                },
                Err(err) => {
                    self.had_error = true;
                    eprintln!("{}", err);
                    break;
                }
            }
        }

        Ok(())
    }
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
    #[snafu(display("Runtime error - found {found}, expected {expected}\n[line {}]", token.line))]
    Runtime {
        found: String,
        expected: String,
        token: Token,
    },
}

type Result<T> = std::result::Result<T, LoxError>;
