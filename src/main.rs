use std::{env, iter::Peekable};

use rust9cc::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Wrong number of arguments.");
    }

    let input = args.get(1).unwrap();
    let tokens = tokenize(input).unwrap();
    let mut tokens = tokens.into_iter().peekable();

    println!("  ret");
}
