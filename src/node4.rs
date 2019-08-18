use crate::{Leaf, Node, Node16, Node4, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;

impl Node4 {
    pub(crate) fn should_grow(&self) -> bool {
        self.keys().len() == 4
    }

    pub(crate) fn child_at(&self, key: u8) -> Option<&Node> {
        match self.children.iter().find(|n| n.0 == key) {
            Some(item) => Some(item.1.borrow()),
            None => None,
        }
    }

    pub(crate) fn child_at_mut(&mut self, key: u8) -> Option<&mut Node> {
        match self.children.iter_mut().find(|n| n.0 == key) {
            Some(item) => Some(item.1.borrow_mut()),
            None => None,
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
        self.children.first().unwrap().1.borrow()
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

    pub(crate) fn outgoing_children(&self) -> Vec<&Node> {
        self.children.iter().map(|i| i.1.borrow()).collect()
    }

    pub(crate) fn term_leaf_mut(&mut self) -> Option<&mut Box<Node>> {
        self.term_leaf.as_mut()
    }
    pub(crate) fn term_leaf(&self) -> Option<&Box<Node>> {
        self.term_leaf.as_ref()
    }

    pub(crate) fn new() -> Self {
        Node4 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            children: Vec::with_capacity(4),
            term_leaf: None,
        }
    }

    pub(crate) fn partial(&self) -> &[u8] {
        &self.meta.partial
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len
    }

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
}

impl Display for Node4 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Node4({clen}) {keys:?} {keys_ch:?} ({leaf}) - ({plen}) [{partial:?}]",
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
    use crate::Leaf;

    #[test]
    fn test_add_child4() {
        let mut node4 = Node4::new();
        // add first child
        node4.add_child(Node::None, Some(1));
        node4.add_child(Node::None, Some(4));
        node4.add_child(Node::None, Some(2));
        node4.add_child(Node::None, Some(3));

        let keys: Vec<u8> = (1..5).collect();
        let nodes = vec![Node::None; 4];
        let res: Vec<(Option<u8>, &Node)> = keys
            .iter()
            .zip(nodes.iter())
            .map(|x| (Some(*x.0), x.1))
            .collect();
        assert_eq!(node4.children(), res);
    }

    #[test]
    fn test_add_leaf() {
        let mut node4 = Node4::new();
        println!("&node4 = {:#?}", &node4);
        // add first child
        node4.add_child(Node::None, Some(1));
        println!("&node4 = {:#?}", &node4);
        // leaf
        let k = "1".as_bytes().to_vec();
        let leaf = Node::Leaf(Leaf::new(k.clone(), k));
        node4.add_child(leaf.clone(), None);
        // another child
        node4.add_child(Node::None, Some(4));

        let keys: Vec<u8> = vec![1, 4];
        let nodes = vec![&Node::None; 2];
        assert_eq!(node4.keys(), keys);
        assert_eq!(node4.outgoing_children(), nodes);
    }

    #[test]
    fn test_vec_sorting_by_node() {
        let mut items = Vec::new();
        items.push((1, Node::None));
        items.push((3, Node::None));
        items.push((2, Node::None));
        items.push((4, Node::None));

        items.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        println!("&items = {:#?}", &items);
    }

    #[test]
    fn test_display_string() {
        let mut node4 = Node4::new();
        node4.add_child(Node::None, Some(1));
        node4.add_child(Node::None, Some(4));
        node4.add_child(Node::None, Some(2));
        node4.add_child(Node::None, Some(3));
        let chars = vec![1, 2, 3, 4]
            .iter()
            .map(|i| *i as u8 as char)
            .collect::<Vec<char>>();
        let match_str = format!(
            "Node4(4) [1, 2, 3, 4] {chars:?} (false) - (0) [[]]",
            chars = chars
        );
        let node_str = format!("{}", &node4);
        assert_eq!(match_str, node_str);

        let k = "1".as_bytes().to_vec();
        let leaf = Node::Leaf(Leaf::new(k.clone(), k));
        node4.add_child(leaf.clone(), None);
        let match_str = format!(
            "Node4(5) [1, 2, 3, 4] {chars:?} (true) - (0) [[]]",
            chars = chars
        );
        let node_str = format!("{}", &node4);
        assert_eq!(match_str, node_str);
    }
}
