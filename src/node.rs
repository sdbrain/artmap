use crate::{Node, Node16, Node256, Node32, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::cmp::min;
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;
use xi_rope::compare::{ne_idx, ne_idx_rev};

impl Node {
    pub(crate) fn key_char(key: &[u8], depth: usize) -> Option<u8> {
        if key.len() - 1 < depth {
            None
        } else {
            Some(key[depth])
        }
    }

    fn match_key(&self, key: &[u8], max_match_len: usize, depth: usize) -> Option<usize> {
        let meta = self.get_meta();
        let mut idx = 0;
        while idx < max_match_len {
            if meta.partial[idx] != key[depth + idx] {
                return Some(idx);
            }
            idx += 1;
        }
        Some(idx)
    }

    pub(crate) fn prefix_match(&self, key: &[u8], depth: usize) -> usize {
        // match from depth..max_match_len
        let max_match_len = min(min(MAX_PREFIX, self.partial().len()), key.len() - depth);
        self.match_key(key, max_match_len, depth).unwrap_or(0)
    }

    pub(crate) fn prefix_match_deep(&self, key: &[u8], depth: usize) -> usize {
        let mut mismatch_idx = self.prefix_match(key, depth);
        if mismatch_idx < MAX_PREFIX {
            mismatch_idx
        } else {
            // find leaf following the minimum node (None key)
            let leaf = self.minimum();
            if let Node::Leaf(leaf) = leaf {
                let limit = min(leaf.key.len(), key.len()) - depth;
                while mismatch_idx < limit {
                    if leaf.key[mismatch_idx + depth] != key[mismatch_idx + depth] {
                        break;
                    }
                    mismatch_idx += 1;
                }
                mismatch_idx
            } else {
                0
            }
        }
    }

    pub(crate) fn minimum(&self) -> &Node {
        let mut tmp_node = self;
        loop {
            match tmp_node {
                Node::Leaf(_) => {
                    return tmp_node;
                }
                Node::None => {
                    panic!("Should not be here");
                }
                node => {
                    // if we have a node at term_leaf, assign tmp_node to that and continue
                    // else use the first element in the children list
                    if node.term_leaf().is_some() {
                        tmp_node = node.term_leaf().as_ref().unwrap();
                    } else {
                        tmp_node = node.first();
                    }
                }
            }
        }
    }

    pub(crate) fn set_prefix_len(&mut self, new_prefix_len: usize) {
        self.get_meta_mut().prefix_len = new_prefix_len;
    }

    pub(crate) fn set_partial(&mut self, new_partial: Vec<u8>) {
        self.get_meta_mut().partial = new_partial;
    }

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match self {
            Node::Node4(node4) => {
                if node4.should_grow() {
                    let mut node16 = Node::Node16(Node16::new());
                    let old_node = replace(self, node16);
                    self.copy(old_node);
                    self.add_child(node, key_char);
                } else {
                    node4.add_child(node, key_char);
                }
            }
            Node::Node16(node16) => {
                if node16.should_grow() {
                    let mut node256 = Node::Node256(Node256::new());
                    let old_node = replace(self, node256);
                    self.copy(old_node);
                    self.add_child(node, key_char);
                } else {
                    node16.add_child(node, key_char);
                }
            }
            Node::Node32(node32) => {
                if node32.should_grow() {
                    let mut node256 = Node::Node256(Node256::new());
                    let old_node = replace(self, node256);
                    self.copy(old_node);
                    self.add_child(node, key_char);
                } else {
                    node32.add_child(node, key_char);
                }
            }
            Node::Node256(node256) => node256.add_child(node, key_char),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn child_exists(&self, key: &[u8], depth: usize) -> bool {
        if let Some(key_char) = key.get(depth) {
            self.child_at(*key_char).is_some()
        } else if key.len() == depth {
            self.term_leaf().is_some()
        } else {
            false
        }
    }

    pub(crate) fn find_child(&self, key: &[u8], depth: usize) -> Option<&Node> {
        if let Some(ch) = key.get(depth) {
            self.child_at(*ch)
        } else if depth == key.len() {
            if let Some(child_node) = &self.term_leaf() {
                Some(child_node)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn find_child_mut(&mut self, key: &[u8], depth: usize) -> Option<&mut Node> {
        if let Some(ch) = key.get(depth) {
            self.child_at_mut(*ch)
        } else if key.len() == depth {
            if self.term_leaf().is_some() {
                Some(self.term_leaf_mut().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_meta(&self) -> &NodeMeta {
        match self {
            Node::Node4(node4) => &node4.meta,
            Node::Node16(node16) => &node16.meta,
            Node::Node32(node32) => &node32.meta,
            Node::Node256(node256) => &node256.meta,
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    fn get_meta_mut(&mut self) -> &mut NodeMeta {
        match self {
            Node::Node4(node4) => &mut node4.meta,
            Node::Node16(node16) => &mut node16.meta,
            Node::Node32(node32) => &mut node32.meta,
            Node::Node256(node256) => &mut node256.meta,
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    pub(crate) fn term_leaf(&self) -> Option<&Box<Node>> {
        match self {
            Node::Node4(node4) => node4.term_leaf(),
            Node::Node16(node16) => node16.term_leaf(),
            Node::Node32(node32) => node32.term_leaf(),
            Node::Node256(node256) => node256.term_leaf(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn term_leaf_mut(&mut self) -> Option<&mut Box<Node>> {
        match self {
            Node::Node4(node4) => node4.term_leaf_mut(),
            Node::Node16(node16) => node16.term_leaf_mut(),
            Node::Node32(node32) => node32.term_leaf_mut(),
            Node::Node256(node256) => node256.term_leaf_mut(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn child_at(&self, key: u8) -> Option<&Node> {
        match self {
            Node::Node4(node4) => node4.child_at(key),
            Node::Node16(node16) => node16.child_at(key),
            Node::Node32(node32) => node32.child_at(key),
            Node::Node256(node256) => node256.child_at(key),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        match self {
            Node::Node4(node4) => node4.child_at_mut(key),
            Node::Node16(node16) => node16.child_at_mut(key),
            Node::Node32(node32) => node32.child_at_mut(key),
            Node::Node256(node256) => node256.child_at_mut(key),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn prefix_len(&self) -> usize {
        match self {
            Node::Node4(node4) => node4.prefix_len(),
            Node::Node16(node16) => node16.prefix_len(),
            Node::Node32(node32) => node32.prefix_len(),
            Node::Node256(node256) => node256.prefix_len(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn partial(&self) -> &[u8] {
        match self {
            Node::Node4(node4) => node4.partial(),
            Node::Node16(node16) => node16.partial(),
            Node::Node32(node32) => node32.partial(),
            Node::Node256(node256) => node256.partial(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn children(&self) -> Vec<(Option<u8>, &Node)> {
        match self {
            Node::Node4(node4) => node4.children(),
            Node::Node16(node16) => node16.children(),
            Node::Node32(node32) => node32.children(),
            Node::Node256(node256) => node256.children(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn first(&self) -> &Node {
        match self {
            Node::Node4(node4) => node4.first(),
            Node::Node16(node16) => node16.first(),
            Node::Node32(node32) => node32.first(),
            Node::Node256(node256) => node256.first(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match self {
            Node::Node4(node4) => panic!("should not be here"),
            Node::Node16(node16) => node16.copy(node_to_copy),
            Node::Node32(node32) => node32.copy(node_to_copy),
            Node::Node256(node256) => node256.copy(node_to_copy),
            _ => unimplemented!(),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Node::Node4(node4) => write!(f, "{}", node4),
            Node::Node16(node16) => write!(f, "{}", node16),
            Node::Node32(node32) => write!(f, "{}", node32),
            Node::Node256(node256) => write!(f, "{}", node256),
            Node::Leaf(leaf) => write!(f, "{}", leaf),
            Node::None => write!(f, ""),
        }
    }
}
