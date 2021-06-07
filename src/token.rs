use std::{convert::TryFrom, iter::Peekable};

use anyhow::{anyhow, Context, Result};

const BASE10: u32 = 10;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
    Num(u64),
    Plus,
    Minus,
    Mul,
    Div,
    LParen, // (
    RParen, // )
    Eq,     // ==
    Neq,    // !=
    Lt,     // <
    Leq,    // <=
    Gt,     // >
    Geq,    // >=
    Eof,
}

impl TryFrom<char> for TokenKind {
    type Error = anyhow::Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            _ => {
                return Err(anyhow!(format!("")));
            }
        };
        Ok(kind)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    // TODO: info
}

struct InputReader<'a> {
    reader: &'a str,
}

impl<'a> Iterator for InputReader<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.len() == 0 {
            return None;
        }
        let res = self.peek();
        self.advance(1).unwrap();
        res
    }
}

impl<'a> InputReader<'a> {
    fn new(input: &'a str) -> Self {
        InputReader { reader: input }
    }

    fn len(&self) -> usize {
        self.reader.len()
    }

    fn starts_with(&self, pat: &str) -> bool {
        self.reader.starts_with(pat)
    }

    pub fn advance(&mut self, n: usize) -> Result<()> {
        let (_, reader) = self.reader.split_at(n);
        self.reader = reader;
        Ok(())
    }

    fn consume_number(&mut self) -> Result<u64> {
        let mut buf: Vec<String> = Vec::new();
        while let Some(c) = self.peek() {
            if !c.is_digit(BASE10) {
                break;
            }
            buf.push(c.to_string());
            self.advance(1)?;
        }
        let num: u64 = buf.join("").parse()?;
        Ok(num)
    }

    fn peek(&self) -> Option<char> {
        self.reader.chars().nth(0)
    }

    fn head(&self, n: usize) -> Option<&str> {
        if self.reader.len() < n {
            return None;
        }
        let (head, _) = self.reader.split_at(n);
        Some(head)
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    // let mut stream = input.chars().into_iter().peekable()
    let mut reader = InputReader::new(input);

    while reader.len() > 0 {
        if reader.starts_with(" ") {
            reader.advance(1)?;
            continue;
        }

        if let Some(head) = reader.head(2) {
            if let Some(kind) = match head {
                "==" => Some(TokenKind::Eq),
                "!=" => Some(TokenKind::Neq),
                "<=" => Some(TokenKind::Leq),
                ">=" => Some(TokenKind::Geq),
                _ => None,
            } {
                tokens.push(Token { kind });
                reader.advance(2)?;
                continue;
            }
        }

        if let Some(head) = reader.head(1) {
            if let Some(kind) = match head {
                "+" => Some(TokenKind::Plus),
                "-" => Some(TokenKind::Minus),
                "*" => Some(TokenKind::Mul),
                "/" => Some(TokenKind::Div),
                "(" => Some(TokenKind::LParen),
                ")" => Some(TokenKind::RParen),
                "<" => Some(TokenKind::Lt),
                ">" => Some(TokenKind::Gt),
                _ => None,
            } {
                tokens.push(Token { kind });
                reader.advance(1)?;
                continue;
            }
        }

        if let Ok(num) = reader.consume_number() {
            tokens.push(Token {
                kind: TokenKind::Num(num),
            });
            continue;
        }

        return Err(anyhow!(format!("Unable to tokenize {:?}", reader.peek())));
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
    });

    Ok(tokens)
}

// Consumes if the current token is expected one.
pub fn consume<Tokens>(expected_kind: TokenKind, tokens: &mut Peekable<Tokens>) -> bool
where
    Tokens: Iterator<Item = Token>,
{
    if let Some(token) = tokens.peek() {
        if token.kind == expected_kind {
            tokens.next();
            return true;
        }
    }
    false
}

// Expects a given kind of token and read next.
pub fn expect<Tokens>(expected_kind: TokenKind, tokens: &mut Peekable<Tokens>) -> Result<()>
where
    Tokens: Iterator<Item = Token>,
{
    let actual_kind = tokens.peek().context("Not peekable.")?.kind;
    if actual_kind != expected_kind {
        return Err(anyhow!(
            "Expect {:?}, but got {:?}",
            expected_kind,
            actual_kind
        ));
    }
    tokens.next();
    Ok(())
}

// Expects a number and read next.
pub fn expect_number<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<u64>
where
    Tokens: Iterator<Item = Token>,
{
    let kind = tokens.peek().context("Not peekable.")?.kind;
    match kind {
        TokenKind::Num(num) => {
            tokens.next();
            Ok(num)
        }
        _ => Err(anyhow!("Expected num, but found {:?}", kind)),
    }
}

#[cfg(test)]
mod tests {
    use crate::token::*;
    use anyhow::Context;

    #[test]
    fn test_reader() -> Result<()> {
        let mut reader = InputReader::new("123abc");

        let head = reader.head(4);
        assert_eq!(head.unwrap(), "123a");

        let head = reader.head(10);
        assert_eq!(head.is_none(), true);

        let num = reader.consume_number()?;
        assert_eq!(num, 123);

        let peek = reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'a');

        reader.advance(1)?;
        let peek = reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'b');

        Ok(())
    }

    #[test]
    fn test_tokenize() -> Result<()> {
        tokenize("(-1+2)*3")?;
        assert_eq!(
            tokenize("(2)")?,
            vec![
                Token {
                    kind: TokenKind::LParen
                },
                Token {
                    kind: TokenKind::Num(2)
                },
                Token {
                    kind: TokenKind::RParen
                },
                Token {
                    kind: TokenKind::Eof
                },
            ]
        );
        assert_eq!(
            tokenize("  2 * (1+23) - 456 / 7")?,
            vec![
                Token {
                    kind: TokenKind::Num(2)
                },
                Token {
                    kind: TokenKind::Mul
                },
                Token {
                    kind: TokenKind::LParen
                },
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
                    kind: TokenKind::RParen
                },
                Token {
                    kind: TokenKind::Minus
                },
                Token {
                    kind: TokenKind::Num(456)
                },
                Token {
                    kind: TokenKind::Div
                },
                Token {
                    kind: TokenKind::Num(7)
                },
                Token {
                    kind: TokenKind::Eof
                },
            ]
        );

        assert_eq!(
            tokenize("== != <= >= < >")?,
            vec![
                Token {
                    kind: TokenKind::Eq
                },
                Token {
                    kind: TokenKind::Neq
                },
                Token {
                    kind: TokenKind::Leq
                },
                Token {
                    kind: TokenKind::Geq
                },
                Token {
                    kind: TokenKind::Lt
                },
                Token {
                    kind: TokenKind::Gt
                },
                Token {
                    kind: TokenKind::Eof
                },
            ]
        );

        Ok(())
    }
}
