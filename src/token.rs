use std::iter::Peekable;

use anyhow::{anyhow, Context, Result};

use crate::CompileError;

const BASE10: u32 = 10;

/// Represents location in a file (line, column).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
}

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: Loc,
}

struct InputReader<'a> {
    reader: &'a str,
    pub loc: Loc,
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
        InputReader {
            reader: input,
            loc: Loc { line: 0, col: 0 },
        }
    }

    fn len(&self) -> usize {
        self.reader.len()
    }

    fn starts_with(&self, pat: &str) -> bool {
        self.reader.starts_with(pat)
    }

    pub fn advance(&mut self, n: usize) -> Result<()> {
        let (head, tail) = self.reader.split_at(n);
        self.reader = tail;
        self.loc = if head.contains("\n") {
            Loc {
                col: 0,
                line: self.loc.line + 1,
            }
        } else {
            Loc {
                col: self.loc.col + 1,
                line: self.loc.line,
            }
        };
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
        let loc = reader.loc;

        if let Some(head) = reader.head(2) {
            if let Some(kind) = match head {
                "==" => Some(TokenKind::Eq),
                "!=" => Some(TokenKind::Neq),
                "<=" => Some(TokenKind::Leq),
                ">=" => Some(TokenKind::Geq),
                _ => None,
            } {
                tokens.push(Token { kind, loc });
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
                tokens.push(Token { kind, loc });
                reader.advance(1)?;
                continue;
            }
        }

        if let Ok(num) = reader.consume_number() {
            tokens.push(Token {
                kind: TokenKind::Num(num),
                loc,
            });
            continue;
        }

        return Err(CompileError::Tokenize(
            reader.peek().unwrap().to_string(),
            loc,
        ))?;
    }
    let token = Token {
        kind: TokenKind::Eof,
        loc: reader.loc,
    };
    tokens.push(token);

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
        assert_eq!(reader.loc, Loc { line: 0, col: 0 });

        let head = reader.head(10);
        assert_eq!(head.is_none(), true);

        let num = reader.consume_number()?;
        assert_eq!(num, 123);
        assert_eq!(reader.loc, Loc { line: 0, col: 3 });

        let peek = reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'a');

        reader.advance(1)?;
        let peek = reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'b');

        Ok(())
    }

    #[test]
    fn test_multiline_reader() -> Result<()> {
        let input = vec!["a", "bc"].join("\n");
        let mut reader = InputReader::new(input.as_str());

        reader.advance(1)?;
        assert_eq!(reader.loc, Loc { line: 0, col: 1 });

        reader.advance(1)?;
        assert_eq!(reader.peek().context("Not peekable")?, 'b');
        assert_eq!(reader.loc, Loc { line: 1, col: 0 });

        Ok(())
    }

    /// Remove loc from a given tokens.
    fn remove_loc(tokens: Vec<Token>) -> Vec<Token> {
        tokens
            .into_iter()
            .map(|x| Token {
                kind: x.kind,
                loc: Loc { col: 0, line: 0 },
            })
            .collect()
    }

    #[test]
    fn test_tokenize() -> Result<()> {
        let loc = Loc { line: 0, col: 0 };
        assert_eq!(
            tokenize("(2)")?,
            vec![
                Token {
                    kind: TokenKind::LParen,
                    loc: Loc { line: 0, col: 0 },
                },
                Token {
                    kind: TokenKind::Num(2),
                    loc: Loc { line: 0, col: 1 },
                },
                Token {
                    kind: TokenKind::RParen,
                    loc: Loc { line: 0, col: 2 },
                },
                Token {
                    kind: TokenKind::Eof,
                    loc: Loc { line: 0, col: 3 },
                },
            ]
        );
        assert_eq!(
            tokenize("  2 * (1+23) - 456 / 7")?,
            vec![
                Token {
                    kind: TokenKind::Num(2),
                    loc: Loc { line: 0, col: 2 },
                },
                Token {
                    kind: TokenKind::Mul,
                    loc: Loc { line: 0, col: 4 },
                },
                Token {
                    kind: TokenKind::LParen,
                    loc: Loc { line: 0, col: 6 },
                },
                Token {
                    kind: TokenKind::Num(1),
                    loc: Loc { line: 0, col: 7 },
                },
                Token {
                    kind: TokenKind::Plus,
                    loc: Loc { line: 0, col: 8 },
                },
                Token {
                    kind: TokenKind::Num(23),
                    loc: Loc { line: 0, col: 9 },
                },
                Token {
                    kind: TokenKind::RParen,
                    loc: Loc { line: 0, col: 11 },
                },
                Token {
                    kind: TokenKind::Minus,
                    loc: Loc { line: 0, col: 13 },
                },
                Token {
                    kind: TokenKind::Num(456),
                    loc: Loc { line: 0, col: 15 },
                },
                Token {
                    kind: TokenKind::Div,
                    loc: Loc { line: 0, col: 19 },
                },
                Token {
                    kind: TokenKind::Num(7),
                    loc: Loc { line: 0, col: 21 },
                },
                Token {
                    kind: TokenKind::Eof,
                    loc: Loc { line: 0, col: 22 },
                },
            ]
        );

        assert_eq!(
            remove_loc(tokenize("== != <= >= < >")?),
            vec![
                Token {
                    kind: TokenKind::Eq,
                    loc
                },
                Token {
                    kind: TokenKind::Neq,
                    loc
                },
                Token {
                    kind: TokenKind::Leq,
                    loc
                },
                Token {
                    kind: TokenKind::Geq,
                    loc
                },
                Token {
                    kind: TokenKind::Lt,
                    loc
                },
                Token {
                    kind: TokenKind::Gt,
                    loc
                },
                Token {
                    kind: TokenKind::Eof,
                    loc
                },
            ]
        );

        Ok(())
    }
}
