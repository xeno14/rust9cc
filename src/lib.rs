pub mod token;
pub mod dot;
pub mod parse;

use self::parse::*;

use anyhow::{anyhow, Context, Result};

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
