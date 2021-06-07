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
    LParen,
    RParen,
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
        self.advance(1).unwrap();
        self.peek()
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

    fn advance(&mut self, n: usize) -> Result<()> {
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

    fn head(&self, n: usize) -> &str {
        let (head, _) = self.reader.split_at(n);
        head
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

        // TokenKind from char.
        let peek: char = reader.peek().context("Expect a charctor.")?;
        if let Ok(kind) = TokenKind::try_from(peek) {
            tokens.push(Token { kind });
            reader.next().unwrap();
            continue;
        }

        if let Ok(num) = reader.consume_number() {
            tokens.push(Token {
                kind: TokenKind::Num(num),
            });
            continue;
        }

        return Err(anyhow!(format!("Unexpected char {}", peek)));
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

//
// AST
//

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num(u64),
}

pub type NodeRef = Box<Node>;

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<NodeRef>,
    pub rhs: Option<NodeRef>,
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<NodeRef>, rhs: Option<NodeRef>) -> Node {
        Self { kind, lhs, rhs }
    }

    pub fn make_ref(self) -> Option<NodeRef> {
        Some(Box::new(self))
    }
}

// expr    = mul ("+" mul | "-" mul)*
fn expr<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = mul(tokens)?;
    loop {
        if consume(TokenKind::Plus, tokens) {
            node = Node::new(NodeKind::Add, node.make_ref(), mul(tokens)?.make_ref());
        } else if consume(TokenKind::Minus, tokens) {
            node = Node::new(NodeKind::Sub, node.make_ref(), mul(tokens)?.make_ref());
        } else {
            break;
        }
    }
    return Ok(node);
}

// mul     = primary ("*" primary | "/" primary)*
fn mul<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = primary(tokens)?;
    loop {
        if consume(TokenKind::Mul, tokens) {
            node = Node::new(NodeKind::Mul, node.make_ref(), primary(tokens)?.make_ref());
        } else if consume(TokenKind::Div, tokens) {
            node = Node::new(NodeKind::Div, node.make_ref(), primary(tokens)?.make_ref());
        }
        break;
    }
    return Ok(node);
}

// primary = num | "(" expr ")"
fn primary<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let node = if consume(TokenKind::LParen, tokens) {
        let node = expr(tokens)?;
        expect(TokenKind::RParen, tokens)?;
        node
    } else {
        let num = expect_number(tokens)?;
        Node::new(NodeKind::Num(num), Option::None, Option::None)
    };
    Ok(node)
}

// Parse tokens and returns AST.
pub fn parse_into_ast<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let node = expr(tokens)?;
    let token = tokens.peek().unwrap();
    if token.kind != TokenKind::Eof {
        return Err(anyhow!(format!("Unexpected token {:?}", token)));
    }
    Ok(node)
}

pub fn gen(node: &Node) -> Result<()> {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    gen_main(node)?;

    println!("  pop rax");
    println!("  ret");

    Ok(())
}

fn gen_main(node: &Node) -> Result<()> {
    if let NodeKind::Num(num) = node.kind {
        println!("  push {}", num);
        return Ok(());
    }

    gen_main(
        node.lhs
            .as_ref()
            .context("Expect non null lhs, but is null.")?
            .as_ref(),
    )?;
    gen_main(
        node.rhs
            .as_ref()
            .context("Expect non null rhs, but is null.")?
            .as_ref(),
    )?;

    // Binary operation.
    println!("  pop rdi");
    println!("  pop rax");
    match node.kind {
        NodeKind::Add => println!("  add rax, rdi"),
        NodeKind::Sub => println!("  sub rax, rdi"),
        NodeKind::Mul => println!("  imul rax, rdi"),
        NodeKind::Div => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        _ => {
            return Err(anyhow!(format!(
                "Expected binary operator but got {:?}",
                node.kind
            )));
        }
    }
    println!("  push rax");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;
    use anyhow::Context;

    #[test]
    fn test_reader() -> Result<()> {
        let mut reader = InputReader::new("123abc");

        let head = reader.head(4);
        assert_eq!(head, "123a");

        let num = reader.consume_number()?;
        assert_eq!(num, 123);

        let peek =reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'a');
        
        reader.advance(1)?;
        let peek = reader.peek().context("Not peekable")?;
        assert_eq!(peek, 'b');

        Ok(())
    }

    #[test]
    fn test_tokenize() -> Result<()> {
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

        Ok(())
    }
}
