use crate::{Node, Node16, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;

impl Node16 {
    pub(crate) fn new() -> Self {
        Node16 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            children: Vec::with_capacity(16),
            term_leaf: None,
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match node_to_copy {
            Node::Node4(node4) => {
                replace(&mut self.meta, node4.meta);
                replace(&mut self.children, node4.children);
                replace(&mut self.term_leaf, node4.term_leaf);
            }
            _ => panic!("only copying from node4 is allowed"),
        }
    }

    pub(crate) fn should_grow(&self) -> bool {
        self.children.len() == 16
    }

    // TODO ===================== Refactor and share between Node4 and Node16 =====

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match key_char {
            Some(current_char) => {
                self.children.push((current_char, node));
                self.children.sort_unstable_by(|a, b| a.0.cmp(&b.0));
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

    pub(crate) fn children(&self) -> Vec<(Option<u8>, &Node)> {
        let mut res: Vec<(Option<u8>, &Node)> =
            self.children.iter().map(|n| (Some(n.0), &n.1)).collect();
        if self.term_leaf().is_some() {
            res.push((None, self.term_leaf.as_ref().unwrap()));
        }
        res
    }

    pub(crate) fn keys(&self) -> Vec<u8> {
        self.children.iter().map(|i| i.0).collect()
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

    fn find_index(&self, key: u8) -> Option<usize> {
        match self.children.binary_search_by(|x| key.cmp(&x.0)) {
            Err(E) => None,
            Ok(index) => Some(index),
        }
    }
    pub(crate) fn child_at(&self, key: u8) -> Option<&Node> {
        let index = self.find_index(key);
        if index.is_none() {
            return None;
        }

        match self.children.get(index.unwrap()) {
            Some(item) => Some(item.1.borrow()),
            None => None,
        }
    }

    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        let index = self.find_index(key);
        // no match found
        if index.is_none() {
            return None;
        }

        match self.children.get_mut(index.unwrap()) {
            Some(item) => Some(item.1.borrow_mut()),
            None => None,
        }
    }
}

impl Display for Node16 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Node16({clen}) {keys:?} {keys_ch:?} ({leaf}) - ({plen}) [{partial:?}]",
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
