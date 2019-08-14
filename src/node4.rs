use crate::{Node, Node4, NodeMeta, MAX_PREFIX};

impl Node4 {
    pub(crate) fn len(&self) -> usize {
        let mut count: usize = 0;
        for (c, node) in self.children.iter().enumerate() {
            match node {
                Node::None => {}
                _ => {
                    count += 1;
                }
            }
        }
        count
    }

    pub(crate) fn children(&self) -> Vec<(usize, &Node)> {
        self.children
            .iter()
            .enumerate()
            .filter(|n| match *n.1 {
                Node::None => false,
                _ => true,
            })
            .map(|n| (n.0, n.1))
            .collect()
    }

    pub(crate) fn max_leaf_index(&self) -> usize {
        3 + 1
    }

    pub(crate) fn new() -> Self {
        Node4 {
            meta: NodeMeta {
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            keys: vec![0; 4],
            children: vec![Node::None; 4 + 1],
        }
    }

    pub(crate) fn partial(&self) -> &[u8] {
        &self.meta.partial
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.meta.prefix_len
    }

    pub(crate) fn add_child(&mut self, node: Node, key_char: usize) {
        self.children[key_char] = node;
    }
}
