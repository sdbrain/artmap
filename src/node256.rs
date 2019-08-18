use crate::{Node, Node256, NodeMeta, MAX_PREFIX};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;

impl Node256 {
    pub(crate) fn new() -> Self {
        Node256 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            children: vec![Node::None; 256],
            term_leaf: None,
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match node_to_copy {
            Node::Node32(node32) => {
                replace(&mut self.meta, node32.meta);
                replace(&mut self.term_leaf, node32.term_leaf);

                // copy the children
                for (key, val) in node32.children {
                    let key = key as usize;
                    self.children[key] = val;
                }
            }
            _ => panic!("only copying from node16 is allowed"),
        };
    }

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match key_char {
            Some(current_char) => {
                let current_char = current_char as usize;
                self.children[current_char] = node;
            }
            None => {
                // key char would be None in the case of leaf nodes.
                self.term_leaf = Some(Box::new(node))
            }
        }
    }

    pub(crate) fn len(&self) -> usize {
        let mut leaf_count = 0;
        if self.term_leaf.is_some() {
            leaf_count += 1;
        }
        self.children.len() + leaf_count
    }

    pub(crate) fn first(&self) -> &Node {
        self.children
            .iter()
            .find(|x| match x {
                Node::None => false,
                node => true,
            })
            .unwrap()
    }

    pub(crate) fn children(&self) -> Vec<(Option<u8>, &Node)> {
        let mut res: Vec<(Option<u8>, &Node)> = self
            .children
            .iter()
            .enumerate()
            .map(|n| (Some(n.0 as u8), n.1))
            .filter(|n| match n.1 {
                Node::None => false,
                _ => true,
            })
            .collect();
        if self.term_leaf().is_some() {
            res.push((None, self.term_leaf.as_ref().unwrap()));
        }
        res
    }

    pub(crate) fn keys(&self) -> Vec<u8> {
        self.children
            .iter()
            .enumerate()
            .map(|n| (n.0 as u8, n.1))
            .filter(|n| match n.1 {
                Node::None => false,
                _ => true,
            })
            .map(|n| n.0)
            .collect()
    }

    pub(crate) fn term_leaf_mut(&mut self) -> Option<&mut Box<Node>> {
        self.term_leaf.as_mut()
    }

    pub(crate) fn term_leaf(&self) -> Option<&Box<Node>> {
        self.term_leaf.as_ref()
    }

    pub(crate) fn partial(&self) -> &[u8] {
        &self.meta.partial
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len
    }

    pub(crate) fn child_at(&self, key: u8) -> Option<&Node> {
        let res = self.children.get(key as usize).unwrap();
        let res = match res {
            Node::None => None,
            _ => Some(res),
        };
        res
    }

    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        let res = self.children.get_mut(key as usize).unwrap();
        let res = match res {
            Node::None => None,
            _ => Some(res),
        };
        res
    }
}

impl Display for Node256 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Node256({clen}) {keys:?} {keys_ch:?} ({leaf}) - ({plen}) [{partial:?}]",
            clen = self.children().len(),
            keys = self.keys(),
            keys_ch = self
                .keys()
                .iter()
                .map(|i| *i as char)
                .collect::<Vec<char>>(),
            plen = self.prefix_len(),
            partial = self
                .partial()
                .iter()
                .map(|c| *c as char)
                .collect::<Vec<char>>(),
            leaf = self.term_leaf().is_some()
        )
    }
}
