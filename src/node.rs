use crate::{Node, MAX_PREFIX, NodeMeta};
use std::cmp::min;

impl Node {
    fn match_key(&self, key: &[u8], max_match_len: usize, depth: usize) -> Option<usize> {
        // TODO fix this once compilation errors are fixed
        //        let one = &self.meta.partial[0..max_match_len];
        //        let two = &key[depth..];
        //        ne_idx(one, two)

        let meta = match self {
            Node::Node4(node4) => &node4.meta,
            Node::Node16(node16) => &node16.meta,
            Node::Node256(node256) => &node256.meta,
            // TODO return error
            _ => panic!("Should not be here")
        };

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

                Node::Node4(node4) => {
                    // if we have a node at LEAF_INDEX, assign tmp_node to that and continue
                    // else use the first element in the children list
                    match node4.children.get(node4.max_leaf_index()).unwrap() {
                        Node::None => {
                            tmp_node = node4.children.first().unwrap();
                        }
                        node => {
                            tmp_node = node;
                        }
                    }
                }
                Node::Node16(node16) => {
                    // if we have a node at LEAF_INDEX, assign tmp_node to that and continue
                    // else use the first element in the children list
                    match node16.children.get(node16.max_leaf_index()).unwrap() {
                        Node::None => {
                            tmp_node = node16.children.first().unwrap();
                        }
                        node => {
                            tmp_node = node;
                        }
                    }
                }
                Node::Node256(node256) => {
                    // if we have a node at LEAF_INDEX, assign tmp_node to that and continue
                    // else find the first non empty child and assign it to tmp_node and continue
                    match node256.children.get(node256.max_leaf_index()).unwrap() {
                        Node::None => {
                            for child in node256.children.iter() {
                                if let Node::None = child {
                                    // no op
                                } else {
                                    tmp_node = child;
                                    break;
                                }
                            }
                        }
                        node => {
                            tmp_node = node;
                        }
                    }
                }
                Node::None => {
                    panic!("Should not be here");
                }
            }
        }
    }

    fn get_meta(&self) -> &NodeMeta {
        match self {
            Node::Node4(node4) => {
                &node4.meta
            }
            Node::Node16(node16) => {
                &node16.meta
            }
            Node::Node256(node256) => {
                &node256.meta
            }
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    fn get_meta_mut(&mut self) -> &mut NodeMeta {
        match self {
            Node::Node4(node4) => {
                &mut node4.meta
            }
            Node::Node16(node16) => {
                &mut node16.meta
            }
            Node::Node256(node256) => {
                &mut node256.meta
            }
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    pub(crate) fn set_prefix_len(&mut self, new_prefix_len: usize) {
        self.get_meta_mut().prefix_len = new_prefix_len;
    }

    pub(crate) fn set_partial(&mut self, new_partial: Vec<u8>) {
        self.get_meta_mut().partial = new_partial;
    }

    pub(crate) fn add_child(&mut self, node: Node, key_char: usize) {
        match self {
            Node::Node4(node4) => {
                node4.add_child(node, key_char);
            }
            _ => {}
        }
    }

    pub(crate) fn child_exists(&self, key: &[u8], depth: usize) -> bool {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(key_char) = key.get(depth) {
                    let node = node4.children.get(*key_char as usize).unwrap();
                    match node {
                        Node::None => false,
                        _ => true,
                    }
                } else if key.len() == depth {
                    let node = node4.children.get(node4.max_leaf_index()).unwrap();
                    match node {
                        Node::None => false,
                        _ => true,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub(crate) fn find_child(&self, key: &[u8], depth: usize) -> Option<&Node> {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(ch) = key.get(depth) {
                    if let Some(child_node) = node4.children.get(*ch as usize) {
                        Some(child_node)
                    } else {
                        None
                    }
                } else if depth == key.len() {
                    if let Some(child_node) = node4.children.get(node4.max_leaf_index()) {
                        Some(child_node)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn find_child_mut(&mut self, key: &[u8], depth: usize) -> Option<&mut Node> {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(ch) = key.get(depth) {
                    if let Some(child_node) = node4.children.get_mut(*ch as usize) {
                        Some(child_node)
                    } else {
                        None
                    }
                } else if key.len() == depth {
                    let leaf_index = node4.max_leaf_index();
                    if let Some(child_node) = node4.children.get_mut(leaf_index) {
                        Some(child_node)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn prefix_len(&self) -> usize {
        match self {
            Node::Node4(node) => node.prefix_len(),
            _ => 0,
        }
    }

    pub(crate) fn partial(&self) -> &[u8] {
        match self {
            Node::Node4(node) => node.partial(),
            _ => unimplemented!(),
        }
    }

    fn children(&self) -> Vec<(usize, &Node)> {
        match self {
            Node::Node4(node) => node.children(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn max_leaf_index(&self) -> usize {
        match self {
            Node::Node4(node) => node.max_leaf_index(),
            Node::Node16(node16) => node16.max_leaf_index(),
            Node::Node256(node256) => node256.max_leaf_index(),
            _ => panic!("Should not be here")
        }
    }
}
