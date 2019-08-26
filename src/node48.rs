use crate::{Node, Node48, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;

impl Node48 {
    pub(crate) fn new() -> Self {
        Node48 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            keys: vec![-1; 256],
            children: Vec::with_capacity(48),
            term_leaf: None,
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match node_to_copy {
            Node::Node16(node16) => {
                replace(&mut self.meta, node16.meta);
                replace(&mut self.term_leaf, node16.term_leaf);

                for child in node16.children {
                    self.children.push(child.1);
                    self.keys[child.0 as usize] = (self.children.len() - 1) as i8;
                }
            }
            _ => panic!("only copying from node16 is allowed"),
        }
    }

    pub(crate) fn should_grow(&self) -> bool {
        self.children.len() == 48
    }

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match key_char {
            Some(current_char) => {
                self.children.push(node);
                self.keys[current_char as usize] = (self.children.len() - 1) as i8;
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
        let mut key_index = 0;
        for key in self.keys.iter().enumerate() {
            if *key.1 >= 0 {
                key_index = key.0;
                break;
            }
        }
        self.children.get(key_index).unwrap().borrow()
    }

    pub(crate) fn children(&self) -> Vec<(Option<u8>, &Node)> {
        let mut result: Vec<(Option<u8>, &Node)> = Vec::new();
        for key in self.keys.iter().enumerate() {
            if *key.1 >= 0 {
                let key_index = *key.1;
                result.push((Some(key.0 as u8), self.children.get(key_index as usize).unwrap().borrow()))
            }
        }
        if self.term_leaf().is_some() {
            result.push((None, self.term_leaf.as_ref().unwrap()));
        }
        result
    }

    pub(crate) fn keys(&self) -> Vec<u8> {
        self.keys.iter().enumerate().filter(|x| *x.1 >= 0).map(|x| x.0 as u8).collect()
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
        let key_index = self.keys[key as usize];
        if key_index < 0 {
            return None
        }

        match self.children.get(key_index as usize) {
            Some(item) => Some(item.borrow()),
            None => None,
        }
    }

    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        let key_index = self.keys[key as usize];
        if key_index < 0 {
            return None
        }

        match self.children.get_mut(key_index as usize) {
            Some(item) => Some(item.borrow_mut()),
            None => None,
        }
    }
}

impl Display for Node48 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Node48({clen}) {keys:?} {keys_ch:?} ({leaf}) - ({plen}) [{partial:?}]",
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
        let mut node48 = Node48::new();
        node48.add_child(Node::None, Some(66));
        node48.add_child(Node::None, Some(67));
        node48.add_child(Node::None, Some(75));

        let res = node48.child_at(66);
        assert_eq!(res, Some(&Node::None));
        println!("&res = {:#?}", &res);
        println!("&node48 = {}", &node48);
    }
}
