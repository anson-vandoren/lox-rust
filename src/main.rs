mod ast_printer;
mod expr;
mod interpreter;
mod lox_callable;
mod lox_function;
mod native;
mod object;
mod parser;
mod scanner;
mod stmt;
mod token;
mod token_type;

use std::{env, path::Path};

use ast_printer::AstPrinter;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use snafu::prelude::*;
use token::Token;
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    init_tracing();
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

fn init_tracing() {
    let format = format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| format.into()))
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .init();
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
            .context(FileSnafu { path: script_path.into() })
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

    #[instrument(skip(self, script), err, ret, level = "trace")]
    fn run(&mut self, script: String) -> Result<()> {
        let scanner = Scanner::new(script);
        let tokens = scanner.scan_tokens().inspect_err(|_| {
            self.had_error = true;
        })?;
        let mut parser = Parser::new(tokens);
        let _printer = AstPrinter {};
        let stmts = parser.parse();
        match stmts {
            Ok(s) => {
                self.interpreter.interpret(s).inspect_err(|_| {
                    self.had_runtime_error = true;
                })?;
            }
            Err(err) => {
                self.had_error = true;
                eprintln!("{}", err);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum LoxError {
    #[snafu(display("[line {line}] Error {whence}: {message}"))]
    Parsing { line: usize, whence: String, message: String },
    #[snafu(display("IO error"))]
    Io { source: std::io::Error },
    #[snafu(display("Could not read source file at '{path}'"))]
    File { source: std::io::Error, path: String },
    #[snafu(display("Fatal error, exiting"))]
    Fatal,
    #[snafu(display("Runtime error - found {found}, expected {expected}\n[line {}]", token.line))]
    Runtime { found: String, expected: String, token: Token },
    #[snafu(display("Internal error: {message}"))]
    Internal { message: String },
}

type Result<T> = std::result::Result<T, LoxError>;
