const MAX_PREFIX: usize = 8;

#[derive(Debug)]
pub struct Art {
    root: Node,
    size: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    None,
    Leaf(Leaf),
    Node4(Node4),
    Node16(Node16),
    //    Node48(Node48),
    Node256(Node256),
}

#[derive(Debug, Clone, PartialEq)]
struct NodeMeta {
    // this holds the total size of the prefix and it
    // could be bigger than the partial vector
    // in the partial vector, we store only items
    // that are < MAX_PREFIX len
    prefix_len: usize,
    partial: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
struct Leaf {
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
struct Node4 {
    meta: NodeMeta,
    children: Vec<(u8, Node)>,
    term_leaf: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
struct Node16 {
    meta: NodeMeta,
    children: Vec<(u8, Node)>,
    term_leaf: Option<Box<Node>>,
}

//#[derive(Debug, Clone)]
//struct Node48 {}

#[derive(Debug, Clone, PartialEq)]
struct Node256 {
    meta: NodeMeta,
    children: Vec<Node>,
}

mod art;
mod leaf;
mod node;
mod node16;
mod node256;
mod node4;
