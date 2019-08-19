use crate::{Node, Node32, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;
use itertools::Itertools;
use hashbrown::HashMap;

impl Node32 {
    pub(crate) fn new() -> Self {
        Node32 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            children: HashMap::new(),
            term_leaf: None,
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match node_to_copy {
            Node::Node16(node16) => {
                replace(&mut self.meta, node16.meta);
                replace(&mut self.term_leaf, node16.term_leaf);

                for child in node16.children {
                    self.children.insert(child.0, child.1);
                }
            }
            _ => panic!("only copying from node16 is allowed"),
        }
    }

    pub(crate) fn should_grow(&self) -> bool {
        self.children.len() == 64
    }

    // TODO ===================== Refactor and share between Node4 and Node32 =====

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match key_char {
            Some(current_char) => {
                self.children.insert(current_char, node);
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
        let mut x = self.children.keys().collect_vec();
        x.sort_unstable();

        self.children.get(x.first().unwrap()).unwrap()
    }

    pub(crate) fn children(&self) -> Vec<(Option<u8>, &Node)> {
        let mut res: Vec<(Option<u8>, &Node)> =
            self.children.iter().map(|n| (Some(*n.0), n.1)).collect();
        if self.term_leaf().is_some() {
            res.push((None, self.term_leaf.as_ref().unwrap()));
        }
        res
    }

    pub(crate) fn keys(&self) -> Vec<u8> {
        self.children.iter().map(|i| *i.0).collect()
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
        match self.children.get(&key) {
            Some(item) => Some(item.borrow()),
            None => None,
        }
    }

    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        match self.children.get_mut(&key) {
            Some(item) => Some(item.borrow_mut()),
            None => None,
        }
    }
}

impl Display for Node32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Node32({clen}) {keys:?} {keys_ch:?} ({leaf}) - ({plen}) [{partial:?}]",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_child_at() {
        let mut node32 = Node32::new();
        node32.add_child(Node::None, Some(1));
        node32.add_child(Node::None, Some(2));
        node32.add_child(Node::None, Some(4));

        let res = node32.child_at(1);
        assert_eq!(res, Some(&Node::None));
        println!("&res = {:#?}", &res);
    }
}
