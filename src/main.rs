use std::env;

use rust9cc::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Wrong number of arguments.");
    }

    let input = args.get(1).unwrap();
    let tokens = tokenize(input).unwrap();
    // eprintln!("{:?}", tokens);
    // compile(&mut tokens.into_iter().peekable()).unwrap();
    let tokens = &mut tokens.into_iter().peekable();
    let root = parse_into_ast(tokens).unwrap();
    // eprintln!("{:?}", root);

    gen(&root).unwrap();
}
