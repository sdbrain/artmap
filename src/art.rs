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
                    node4.meta.prefix_len = prefix_len;
                    // push the depth by prefix len
                    depth = depth + prefix_len;

                    // add the leaves to the new node 4

                    let leaf = leaf.clone();
                    let key_char = leaf.key_char(depth);
                    node4.add_child(Node::Leaf(leaf), key_char);
                    count += 1;

                    let leaf2 = Leaf::new(key, value);
                    let key_char = leaf2.key_char(depth);
                    node4.add_child(Node::Leaf(leaf2), key_char);
                    count += 1;

                    *current = Node::Node4(node4);
                    break;
                }
                _ => {
                    let current_prefix_len = Art::calculate_prefix_mismatch(current, &key, depth);
                    if current_prefix_len != current.prefix_len() {
                        let mut node4 = Node4::new();
                        let key_char = *&key[depth + current_prefix_len];
                        node4.add_child(Node::Leaf(Leaf::new(key, value)), Some(key_char));
                        node4.meta.prefix_len = current_prefix_len;
                        node4.meta.partial = current.partial()[..current_prefix_len].iter().map(|i| i.clone()).collect();

                        // fix up current node
                        let old_node = replace(&mut *current, Node::Node4(node4)).unwrap_node4();
                        if let Some(mut node) = old_node {
                            // the reason we add +1 is because the key that we are trying to add
                            // has a prefix match of current_prefix_len and we are going to be using
                            // the +1 character as the pivot in the trie. e.g. AMD, AMDs, AMBs are inserted
                            // in order. So when AMBs is inserted, the current_prefix_len will match till AM
                            // so will be 2. The previous prefix len for AMD and AMDs would have been AMD, so
                            // now the pivot character for the new node would be D and this would mean that
                            // the prefix_len would be 0 for the old node4 and the prefix vec would be empty
                            // the ART paper uses memmove which will leave residual items in the partial vec
                            let key_char = node.meta.partial[current_prefix_len];
                            node.meta.prefix_len = node.prefix_len() - (current_prefix_len + 1);
                            node.meta.partial = node.meta.partial.iter().skip(current_prefix_len + 1).map(|i| *i).collect();

                            if let Node::Node4(n) = current {
                                n.add_child(Node::Node4(node), Some(key_char));
                            }
                            count += 1;
                            break;
                        } else {
                            // should not come here
                        }

                        break;
                    }
                    break;
                }
            }
        }

        self.size += count;
    }

    fn calculate_prefix_mismatch(node: &Node, key: &Vec<u8>, depth: usize) -> usize {
        // match from depth..max_match_len
        let max_match_len = min(min(MAX_PREFIX, node.prefix_len()), key.len() - depth);
        node.match_key(key, max_match_len, depth).unwrap_or(0)
    }
}

impl Node {
    fn prefix_len(&self) -> usize {
        match self {
            Node::Node4(node) => {
                node.prefix_len()
            }
            _ => { 0 }
        }
    }

    fn partial(&self) -> &Vec<u8> {
        match self {
            Node::Node4(node) => { node.partial() }
            _ => { unimplemented!() }
        }
    }

    fn match_key(&self, key: &Vec<u8>, from: usize, depth: usize) -> Option<usize> {
        match self {
            Node::Node4(node) => {
                node.match_key(key, from, depth)
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn unwrap_node4(self) -> Option<Node4> {
        match self {
            Node::Node4(node4) => { Some(node4) }
            _ => None
        }
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
                prefix_len: 0,
                partial: Vec::with_capacity(MAX_PREFIX),
            },
            children: BTreeMap::new(),
        }
    }

    fn partial(&self) -> &Vec<u8> {
        &self.meta.partial
    }

    fn prefix_len(&self) -> usize {
        self.meta.prefix_len
    }

    fn match_key(&self, key: &Vec<u8>, from: usize, depth: usize) -> Option<usize> {
        Some(self.meta.partial.iter().zip(key.iter()).skip(depth).take_while(|(i, j)| i == j).count())
    }

    fn add_child(&mut self, node: Node, key_char: Option<u8>) {
        self.children.insert(key_char, Box::from(node));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node4;

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

    #[test]
    fn test_prefix_len_greater_than_prefix() {
        let mut art = Art::new();
        let mut items = Vec::new();
        items.push("A".repeat(10));
        items.push("A".repeat(20));
        let items: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        _insert(&mut art, &items);

        assert_eq!(art.len(), 2);

        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.meta.prefix_len, 10);
            assert_eq!(node.meta.partial.len(), MAX_PREFIX);
            assert_eq!(node.meta.partial, "A".repeat(MAX_PREFIX).as_bytes().to_vec());
            assert_eq!(node.children.len(), 2);

            // all nodes should be of type leaf
            for (key, child) in node.children.iter() {
                match child.borrow() {
                    Node::Leaf(c) => {}
                    _ => { panic!(" Node should be of type leaf") }
                }
            }

            let A = node.children.get(&None);
            let M = node.children.get(&Some(*"A".repeat(20).as_bytes().first().unwrap()));

            assert!(A.is_some());
            assert!(M.is_some());
        } else {
            // node is not of type node4 so fail
            panic!("Node should be of type node4 {:#?}", &art.root);
        }
    }

    #[test]
    fn test_insert_node4_same_prefix_as_existing() {
        let mut art = Art::new();
        let items = vec!["AMD", "AMDs", "AMBs"];
        _insert(&mut art, &items);
        let x: Vec<String> = items.iter().skip(4).map(|t| String::from(*t)).collect();

        assert_eq!(art.len(), 3);
        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.children.len(), 2);
            assert_eq!(node.meta.prefix_len, 2);
            assert_eq!(node.meta.partial.len(), 2);
            assert_eq!(node.meta.partial, ['A', 'M'].iter().map(|c| *c as u8).collect::<Vec<u8>>());
            let keys: Vec<Option<u8>> =  node.children.keys().map(|i| *i).collect();
            assert_eq!(keys, vec![Some('B' as u8), Some('D' as u8)]);

            let b_node = node.children.get(&Some('B' as u8));
            assert!(b_node.is_some());
            let b_node = b_node.unwrap().borrow();
            if let Node::Leaf(leaf) = b_node {
                assert_eq!(leaf.key, Vec::from("AMBs".as_bytes()));
            } else {
                panic!("b_node should be a leaf");
            }

            let d_node = node.children.get(&Some('D' as u8));
            assert!(d_node.is_some());
            let m_node = d_node.unwrap().borrow();
            if let Node::Node4(node4) = m_node {
                assert_eq!(node4.children.len(), 2);
                assert_eq!(node4.prefix_len(), 0);
                assert!(node4.partial().is_empty())
            } else {
                panic!("d_node should be node4");
            }
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
