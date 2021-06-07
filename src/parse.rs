use std::iter::Peekable;

use crate::token::*;

use anyhow::{anyhow, Result};

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

/// expr    = mul ("+" mul | "-" mul)*
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

/// mul     = primary ("*" primary | "/" primary)*
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
        } else {
            break;
        }
    }
    return Ok(node);
}

/// primary = num | "(" expr ")"
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

/// Parses tokens into AST.
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
