use std::iter::Peekable;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Num(i64),
    Plus,
    Minus,
    Eof,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    // TODO: info
}

fn strtol<CharIterator>(stream: &mut Peekable<CharIterator>) -> Result<i64>
where
    CharIterator: Iterator<Item = char>,
{
    let mut buf: Vec<String> = Vec::new();
    while let Some(c) = stream.peek() {
        if !c.is_digit(10) {
            break;
        }
        buf.push(c.to_string());
        stream.next();
    }
    let num: i64 = buf.join("").parse()?;
    Ok(num)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut stream = input.chars().into_iter().peekable();

    while let Some(&peek) = stream.peek() {
        if peek == ' ' {
            stream.next();
            continue;
        }

        if peek == '+' {
            tokens.push(Token {
                kind: TokenKind::Plus,
            });
            stream.next();
            continue;
        }

        if peek == '-' {
            tokens.push(Token {
                kind: TokenKind::Minus,
            });
            stream.next();
            continue;
        }

        if peek.is_digit(10) {
            let num: i64 = strtol(&mut stream)?;
            tokens.push(Token {
                kind: TokenKind::Num(num),
            });
            continue;
        }
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
    });

    Ok(tokens)
}

pub fn consume<Tokens>(kind: TokenKind, tokens: &mut Peekable<Tokens>) -> bool
where
    Tokens: Iterator<Item = Token>,
{
    if let Some(token) = tokens.peek() {
        if token.kind == kind {
            tokens.next();
            return true;
        }
    }
    false
}

pub fn expect<Tokens>(expected_kind: TokenKind, tokens: &mut Peekable<Tokens>) -> Result<()>
where
Tokens: Iterator<Item = Token>,
{
    let actual_kind = tokens
        .peek()
        .context("Not peekable.")?
        .kind;
    if actual_kind != expected_kind {
        return Err(anyhow!("Expect {:?}, but got {:?}", expected_kind, actual_kind));
    }
    Ok(())
}

pub fn expect_number<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<i64>
where
    Tokens: Iterator<Item = Token>,
{
    let kind = tokens
        .peek()
        .context("Not peekable.")?
        .kind;
    match kind {
        TokenKind::Num(num) => Ok(num),
        _ => Err(anyhow!("Expected num, but found {:?}", kind)),
    }
}

pub fn compile<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<()>
where
    Tokens: Iterator<Item = Token>,
{
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    // Must start with a number.
    println!("  mov rax, {}", expect_number(tokens).unwrap());

    while let Some(token) = tokens.peek() {
        if token.kind == TokenKind::Eof {
            break;
        }
        if consume(TokenKind::Plus, tokens) {
            println!("  add rax, {}", expect_number(tokens).unwrap());
        }
        else if consume(TokenKind::Minus, tokens) {
            println!("  sub rax, {}", expect_number(tokens).unwrap());
        }
        panic!("Unexpected token {:?}", token);
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::*;
    use anyhow::Context;

    #[test]
    fn test_strtol() -> Result<()> {
        let mut stream = "123abc".chars().peekable();
        let num = strtol(&mut stream)?;
        let peek = stream.peek().context("not peekable.")?;

        assert_eq!(num, 123);
        assert_eq!(*peek, 'a');

        Ok(())
    }

    #[test]
    fn test_tokenize() -> Result<()> {
        assert_eq!(
            tokenize("  1+23 - 456")?,
            vec![
                Token {
                    kind: TokenKind::Num(1)
                },
                Token {
                    kind: TokenKind::Plus
                },
                Token {
                    kind: TokenKind::Num(23)
                },
                Token {
                    kind: TokenKind::Minus
                },
                Token {
                    kind: TokenKind::Num(456)
                },
                Token {
                    kind: TokenKind::Eof
                },
            ]
        );

        Ok(())
    }
}
