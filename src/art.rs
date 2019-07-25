use crate::{Art, Leaf, Node, Node4, NodeMeta, MAX_PREFIX};
use std::borrow::{BorrowMut, Borrow};
use std::mem::replace;
use std::cmp::min;
use std::collections::BTreeMap;

impl Art {
    pub fn new() -> Self {
        Art {
            root: Box::new(Node::None),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let current = self.root.borrow_mut();
        let mut depth = 0;
        let mut count = 0;

        loop {
            match current {
                Node::None => {
                    let leaf = Box::new(Node::Leaf(Leaf::new(key, value)));
                    self.root = leaf;
                    break;
                }
                Node::Leaf(leaf) => {
                    // replace value if the key is same
                    if leaf.key.eq(&key) {
                        leaf.value = value;
                        count += 1;
                        break;
                    }

                    // upgrade the leaf to Node4
                    let mut node4 = Node4::new();

                    // compute prefix
                    let prefix_len = leaf
                        .key
                        .iter()
                        .zip(key.iter())
                        .skip(depth)
                        .take_while(|item| item.0 == item.1)
                        .count();
                    node4.meta.partial = key[depth..min(prefix_len, MAX_PREFIX)].to_vec();
                    // push the depth by prefix len
                    depth = depth + prefix_len;

                    // add the leaves to the new node 4

                    node4.add_leaf(leaf.clone(), depth);
                    count += 1;

                    let mut leaf2 = Leaf::new(key, value);
                    node4.add_leaf(leaf2, depth);
                    count += 1;

                    *current = Node::Node4(node4);
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        self.size += count;
    }
}

impl Leaf {
    fn new(new_key: Vec<u8>, new_value: Vec<u8>) -> Self {
        Leaf {
            key: new_key,
            value: new_value,
        }
    }

    fn key_char(&self, depth: usize) -> Option<u8> {
        if self.key.len() - 1 < depth {
            None
        } else {
            Some(self.key[depth])
        }
    }
}

impl Node4 {
    fn new() -> Self {
        Node4 {
            meta: NodeMeta {
                partial: Vec::with_capacity(10),
            },
            children: BTreeMap::new(),
        }
    }

    fn add_leaf(&mut self, node: Leaf, depth: usize) {
        let key = node.key_char(depth);
        self.children.insert(key, Box::from(Node::Leaf(node)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node4;
    use std::any::Any;

    fn _insert(art: &mut Art, items: &Vec<&str>) {
        items.iter().for_each(|item| {
            art.insert(Vec::from(item.as_bytes()), Vec::from(item.as_bytes()));
        });
    }

    #[test]
    fn test_insert_root() {
        let mut art = Art::new();
        let items = vec!["A"];
        _insert(&mut art, &items);

        let key = items.first().unwrap().clone();
        let value = items.first().unwrap().clone();
        assert_eq!(
            *art.root,
            Node::Leaf(Leaf::new(
                key.as_bytes().to_vec(),
                value.as_bytes().to_vec(),
            ))
        )
    }

    #[test]
    fn test_insert_leaf_replace_value() {
        let mut art = Art::new();
        let items = vec!["A"];
        _insert(&mut art, &items);

        // update key A with value B
        let key = items.first().unwrap().as_bytes().to_vec();
        let new_value = "B".as_bytes().to_vec();
        art.insert(key.clone(), new_value.clone());

        assert_eq!(*art.root, Node::Leaf(Leaf::new(key, new_value)));
    }

    #[test]
    fn test_insert_second_leaf() {
        let mut art = Art::new();
        let items = vec!["A", "AMD"];
        _insert(&mut art, &items);

        assert_eq!(art.len(), 2);

        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.meta.partial.len(), 1);
            assert_eq!(node.meta.partial, "A".as_bytes().to_vec());
            assert_eq!(node.children.len(), 2);

            // all nodes should be of type leaf
            for (key, child) in node.children.iter() {
                match child.borrow() {
                    Node::Leaf(c) => {}
                    _ => { panic!(" Node should be of type leaf") }
                }
            }

            let A = node.children.get(&None);
            let M = node.children.get(&Some(*"M".as_bytes().first().unwrap()));

            assert!(A.is_some());
            assert!(M.is_some());
        } else {
            // node is not of type node4 so fail
            panic!("Node should be of type node4 {:#?}", &art.root);
        }
    }
}

//        let mut b = &Box::new(Leaf::new(Vec::from("test".as_bytes()), Vec::from("test".as_bytes())));
//        let a = Box::new(Leaf::new(Vec::from("test".as_bytes()), Vec::from("test".as_bytes())));
//        b = &a;
//        dbg!(&b);

//        let mut keys = vec![1, 2, 4, 3];
//        keys.sort_unstable();
//        dbg!(&keys);
//        if let Ok(index) = keys.binary_search(&1) {
//            let mut arr  = vec![Box::new(Node::None); 4];
//            arr[index] = Box::new(Node::Node4(Node4::new()));
//            dbg!(&arr);
//        }

//        let a = "AMD".as_bytes().to_vec();
//        let b = "A".as_bytes().to_vec();
//
//        let prefix = a.iter().zip(b.iter()).take_while(|item| {
//            item.0 == item.1
//        }).count();
