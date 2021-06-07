use std::iter::Peekable;

use crate::token::*;

use anyhow::{anyhow, Result};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
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

    pub fn new_num(num: u64) -> Node {
        Self {
            kind: NodeKind::Num(num),
            lhs: None,
            rhs: None,
        }
    }

    pub fn make_ref(self) -> Option<NodeRef> {
        Some(Box::new(self))
    }
}

/// expr    = equality
fn expr<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = equality(tokens)?;
    loop {
        if consume(TokenKind::Plus, tokens) {
            node = Node::new(NodeKind::Add, node.make_ref(), equality(tokens)?.make_ref());
        } else if consume(TokenKind::Minus, tokens) {
            node = Node::new(NodeKind::Sub, node.make_ref(), equality(tokens)?.make_ref());
        } else {
            break;
        }
    }
    Ok(node)
}

/// equality   = relational ("==" relational | "!=" relational)*
fn equality<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = relational(tokens)?;
    loop {
        if consume(TokenKind::Eq, tokens) {
            node = Node::new(
                NodeKind::Eq,
                node.make_ref(),
                relational(tokens)?.make_ref(),
            );
        } else if consume(TokenKind::Neq, tokens) {
            node = Node::new(
                NodeKind::Neq,
                node.make_ref(),
                relational(tokens)?.make_ref(),
            );
        } else {
            break;
        }
    }
    Ok(node)
}

/// relational = add ("<" add | "<=" add | ">" add | ">=" add)*
fn relational<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = add(tokens)?;
    loop {
        if consume(TokenKind::Lt, tokens) {
            node = Node::new(NodeKind::Lt, node.make_ref(), add(tokens)?.make_ref());
        } else if consume(TokenKind::Leq, tokens) {
            node = Node::new(NodeKind::Leq, node.make_ref(), add(tokens)?.make_ref());
        } else if consume(TokenKind::Gt, tokens) {
            node = Node::new(NodeKind::Gt, node.make_ref(), add(tokens)?.make_ref());
        } else if consume(TokenKind::Geq, tokens) {
            node = Node::new(NodeKind::Geq, node.make_ref(), add(tokens)?.make_ref());
        } else {
            break;
        }
    }
    Ok(node)
}

/// add        = mul ("+" mul | "-" mul)*
fn add<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
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

/// mul     = unary ("*" unary | "/" unary)*
fn mul<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    let mut node = unary(tokens)?;
    loop {
        if consume(TokenKind::Mul, tokens) {
            node = Node::new(NodeKind::Mul, node.make_ref(), unary(tokens)?.make_ref());
        } else if consume(TokenKind::Div, tokens) {
            node = Node::new(NodeKind::Div, node.make_ref(), unary(tokens)?.make_ref());
        } else {
            break;
        }
    }
    return Ok(node);
}

/// unary = ("+" | "-")? primary
fn unary<Tokens>(tokens: &mut Peekable<Tokens>) -> Result<Node>
where
    Tokens: Iterator<Item = Token>,
{
    if consume(TokenKind::Plus, tokens) {
        primary(tokens)
    } else if consume(TokenKind::Minus, tokens) {
        let node = Node::new(
            NodeKind::Sub,
            Node::new_num(0).make_ref(),
            primary(tokens)?.make_ref(),
        );
        Ok(node)
    } else {
        primary(tokens)
    }
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
        Node::new_num(num)
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
