use crate::{Node, Node32, NodeMeta, MAX_PREFIX};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Display, Error, Formatter};
use std::mem::replace;

impl Node32 {
    pub(crate) fn new() -> Self {
        Node32 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            keys: Vec::with_capacity(32),
            children: Vec::with_capacity(32),
            term_leaf: None,
        }
    }

    pub(crate) fn copy(&mut self, node_to_copy: Node) {
        match node_to_copy {
            Node::Node16(node16) => {
                replace(&mut self.meta, node16.meta);
                replace(&mut self.children, node16.children);
                replace(&mut self.term_leaf, node16.term_leaf);
            }
            _ => panic!("only copying from node16 is allowed"),
        }
    }

    pub(crate) fn should_grow(&self) -> bool {
        self.children.len() == 32
    }

    // TODO ===================== Refactor and share between Node4 and Node32 =====

    pub(crate) fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        match key_char {
            Some(current_char) => {
                self.children.push((current_char, node));
                self.children.sort_unstable_by(|a, b| a.0.cmp(&b.0));

                // TODO fix this once the performance benefit has been established
                self.keys = vec![0u8; 32];
                for x in self.keys().iter().enumerate() {
                    self.keys[x.0] = *x.1;
                }
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

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn find_index_avx(&self, key: u8) -> Option<usize> {
        use std::arch::x86_64::*;
        let key = _mm256_set1_epi8(key as i8);
        let keys = _mm256_loadu_si256(self.keys.as_slice().as_ptr() as *const _);
        let cmp = _mm256_cmpeq_epi8(key, keys);
        let mask = _mm256_movemask_epi8(cmp);
        let tz = mask.trailing_zeros();

        if tz < 31 {
            Some(tz as usize)
        } else {
            None
        }
    }

    fn find_index(&self, key: u8) -> Option<usize> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { self.find_index_avx(key) };
            } else if is_x86_feature_detected!("sse4.2") {
                unimplemented!()
            }
        }
        match self.children.binary_search_by(|x| x.0.cmp(&key)) {
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
