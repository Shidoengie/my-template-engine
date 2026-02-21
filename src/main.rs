use clap::{Parser, ValueEnum};

use std::fmt::{Debug, Display};
use std::fs;
use std::io::{self, Write};
use template_engine::Compiler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The code string or a filepath to execute.
    content: Option<String>,

    /// Treat `content` as a code string instead of a filepath.
    #[arg(short = 'i', long = "input", default_value_t = false)]
    is_code: bool,
    /// If specified, print the output of a compiler stage instead of executing.
    #[arg(short, long, value_enum)]
    stage: Option<Stage>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Stage {
    Lexer,
    Ast,
}

fn print_if_ok<T: Debug, E>(result: Result<T, E>) {
    if let Ok(value) = result {
        println!("{value:#?}");
    }
}

fn display_if_ok<T: Display, E>(result: Result<T, E>) {
    if let Ok(value) = result {
        println!("{value}");
    }
}
/// Runs a specific compiler stage on the given content.
/// `is_expr` should be true for REPL-like single expressions.
fn run_stage(compiler: &mut Compiler, stage: &Stage, content: &str) {
    match stage {
        Stage::Lexer => print_if_ok(compiler.lex(content)),
        Stage::Ast => print_if_ok(compiler.parse(content)),
    }
}

/// Processes a single input string or file.
fn run_once(args: &Args, compiler: &mut Compiler, content: String) {
    let code = if args.is_code {
        Ok(content)
    } else {
        fs::read_to_string(&content)
    };

    match code {
        Ok(code) => {
            if let Some(stage) = &args.stage {
                run_stage(compiler, stage, &code);
            } else {
                let _ = run_stage(compiler, &Stage::Ast, &code);
            }
        }
        Err(e) => {
            eprintln!("Error reading input: {e}");
        }
    }
}
/// Starts an interactive Read-Eval-Print-Loop (REPL).
fn run_repl(compiler: &mut Compiler, stage: Option<Stage>) {
    println!("Shlang REPL. Enter an empty line or press Ctrl+C to exit.");
    loop {
        print!(">: ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }

        if let Some(ref stage) = stage {
            run_stage(compiler, stage, line.trim());
        } else {
            run_stage(compiler, &Stage::Ast, line.trim());
        }
    }
}

fn main() {
    let args = Args::parse();
    let mut compiler = Compiler::new();

    if let Some(content) = args.content.clone() {
        run_once(&args, &mut compiler, content);
    } else {
        run_repl(&mut compiler, args.stage);
    }
}
