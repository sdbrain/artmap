use std::collections::BTreeMap;

const MAX_PREFIX: usize =  8;

#[derive(Debug)]
pub struct Art {
    root: Box<Node>,
    size: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    None,
    Leaf(Leaf),
    Node4(Node4),
    //    Node16(Node16),
    //    Node48(Node48),
    //    Node256(Node256),
}

#[derive(Debug, Clone, PartialEq)]
struct NodeMeta {
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
    children: BTreeMap<Option<u8>, Box<Node>>
}

#[derive(Debug, Clone)]
struct Node16 {
    meta: NodeMeta,
    keys: Vec<u8>,
    children: Vec<Box<Node>>,
}

#[derive(Debug, Clone)]
struct Node48 {}

#[derive(Debug, Clone)]
struct Node256 {}

mod art;
