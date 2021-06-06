use std::iter::Peekable;

use anyhow::{anyhow, Context, Result};

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    // TODO: info
}

fn strtolu<CharIterator>(stream: &mut Peekable<CharIterator>) -> Result<u64>
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
    let num: u64 = buf.join("").parse()?;
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

        if peek == '*' {
            tokens.push(Token {
                kind: TokenKind::Mul,
            });
            stream.next();
            continue;
        }

        if peek == '/' {
            tokens.push(Token {
                kind: TokenKind::Div,
            });
            stream.next();
            continue;
        }

        if peek == '(' {
            tokens.push(Token {
                kind: TokenKind::LParen,
            });
            stream.next();
            continue;
        }

        if peek == ')' {
            tokens.push(Token {
                kind: TokenKind::RParen,
            });
            stream.next();
            continue;
        }

        if peek.is_digit(10) {
            let num: u64 = strtolu(&mut stream)?;
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
    fn test_strtol() -> Result<()> {
        let mut stream = "123abc".chars().peekable();
        let num = strtolu(&mut stream)?;
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
