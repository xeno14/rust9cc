use std::process::exit;

use clap::{App, Arg};
use rust9cc::CompileError;
use rust9cc::display_compile_error;
use rust9cc::gen;
use rust9cc::parse::parse_into_ast;
use rust9cc::dot::dotify_ast;
use rust9cc::token::Token;
use rust9cc::token::tokenize;

const MODE_AST: &str = "ast";
const MODE_TOKEN: &str = "token";
const MODE_X86: &str = "x86";

fn main() {
    let matches = App::new("rust9cc")
        .version("0.0.1")
        .arg(
            Arg::with_name("mode")
                .long("mode")
                .possible_values(&[MODE_AST, MODE_TOKEN, MODE_X86])
                .default_value(MODE_X86),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input expression.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let tokens = match tokenize(input) {
        Ok(tokens) => tokens,
        Err(err) => match err.downcast_ref::<CompileError>() {
            Some(CompileError::Tokenize(_, loc)) => {
                display_compile_error(input, *loc, err.to_string().as_str());
                exit(1);
            },
            _ => {
                println!("{}", err);
                exit(1);
            }
        }
    };

    let mode = matches.value_of("mode").unwrap();
    if mode == MODE_TOKEN {
        for token in tokens.iter() {
            println!("{:?}", token);
        }
        return;
    }

    let tokens = &mut tokens.into_iter().peekable();
    let root = parse_into_ast(tokens).unwrap();

    if mode == MODE_AST {
        dotify_ast(&root);
        return;
    }

    gen(&root).unwrap();
}
