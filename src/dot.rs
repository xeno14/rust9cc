use crate::Node;

struct Counter {
    count: u64,
}

impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }

    fn get(&self) -> u64 {
        self.count
    }

    fn inc(&mut self) {
        self.count += 1;
    }
}

impl Iterator for Counter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let count: u64 = self.count;
        self.inc();
        Some(count)
    }
}

/// Recursively prints AST in dot language.
fn do_dot(node: &Node, counter: &mut Counter) {
    let node_id: u64 = counter.next().unwrap();

    // Print this node.
    println!("{}[label=\"{:?}\"];", node_id, node.kind);

    // Print children.
    if let Some(lhs) = node.lhs.as_ref() {
        println!("{} -> {};", node_id, counter.get());
        do_dot(lhs, counter);
    }
    if let Some(rhs) = node.rhs.as_ref() {
        println!("{} -> {};", node_id, counter.get());
        do_dot(rhs, counter);
    }
}

/// Prints AST in Graphviz dot language.
pub fn dotify_ast(root: &Node) {
    println!("digraph G {{");
    let mut counter = Counter::new();
    do_dot(root, &mut counter);
    println!("}}");
}