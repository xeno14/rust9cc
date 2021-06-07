use clap::{App, Arg};
use rust9cc::*;

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
                .default_value(MODE_X86)
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input expression.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let tokens = tokenize(input).unwrap();
    
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
        println!("{:?}", root);
        return;
    }

    gen(&root).unwrap();
}
