use crate::{Art, Leaf, Node, Node4, NodeMeta, MAX_PREFIX};
use hashbrown::HashMap;
use std::borrow::Borrow;
use std::cmp::min;
use std::mem::replace;
use std::ops::DerefMut;
use xi_rope::compare::ne_idx;

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

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn equals(one: &[u8], two: &[u8]) -> bool {
        if one.len() != two.len() {
            false
        } else {
            // use simd to compare..
            let res = ne_idx(one, two);
            match res {
                Some(_) => false,
                None => true,
            }
        }
    }

    pub fn search(&self, key: &[u8]) -> Option<&[u8]> {
        let mut stack: Vec<&Node> = Vec::new();
        stack.push(self.root.borrow());
        let mut depth: usize = 0;
        loop {
            let current = match stack.pop() {
                Some(item) => item,
                None => break,
            };

            match current {
                Node::Leaf(leaf) => {
                    if Art::equals(leaf.key.as_slice(), key) {
                        return Some(&leaf.value);
                    } else {
                        break;
                    }
                }
                _ => {}
            }

            if current.prefix_len() > 0 {
                let prefix_len = current.prefix_match(key, depth);
                // prefix does not match, stop
                if prefix_len
                    != min(
                    min(MAX_PREFIX, current.prefix_len()),
                    current.partial().len(),
                )
                {
                    break;
                }
                depth += current.prefix_len();
            }

            // go to next child if present
            let child = current.find_child(key, depth);
            match child {
                Some(node) => {
                    stack.push(node);
                    depth += 1;
                }
                None => break,
            }
        }
        None
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let mut current = self.root.deref_mut();
        let mut depth = 0;
        let mut count = 0;

        loop {
            match current {
                Node::None => {
                    let leaf = Box::new(Node::Leaf(Leaf::new(key, value)));
                    self.root = leaf;
                    count += 1;
                    break;
                }
                Node::Leaf(leaf) => {
                    // replace value if the key is same
                    if leaf.key.eq(&key) {
                        leaf.value = value;
                        break;
                    }

                    // upgrade the leaf to Node4
                    let mut node4 = Node4::new();

                    // compute prefix
                    let prefix_len = Art::longest_common_prefix(&leaf.key, &key, depth);
                    node4.meta.partial = Art::calculate_partial(&key, depth, prefix_len);
                    node4.meta.prefix_len = prefix_len;
                    // push the depth by prefix len
                    depth = depth + prefix_len;

                    // add the leaves to the new node 4

                    let leaf = leaf.clone();
                    let key_char = leaf.key_char(depth);
                    node4.add_child(Box::new(Node::Leaf(leaf)), key_char);

                    let leaf2 = Leaf::new(key, value);
                    let key_char = leaf2.key_char(depth);
                    node4.add_child(Box::new(Node::Leaf(leaf2)), key_char);

                    *current = Node::Node4(node4);
                    count += 1;
                    break;
                }
                _ => {
                    let current_prefix_len = current.prefix_match_deep(&key, depth);

                    // prefix matches so have to find a child with current_prefix_len + 1 byte match and
                    // continue the traversal.
                    if current_prefix_len >= current.prefix_len() {
                        // move the char pointer by the prefix to find the next child that correspond to the byte
                        // e.g. A, AMD, AMDs; depth = 0 would be A but that is the common prefix. The next child
                        // would be at M, so doing depth += prefix_len would move the pointer to M
                        depth += current_prefix_len;

                        if !current.child_exists(&key, depth) {
                            let key_char = match key.get(depth) {
                                Some(ch) => Some(*ch),
                                None => None,
                            };

                            let leaf = Box::new(Node::Leaf(Leaf::new(key, value)));
                            current.add_child(leaf, key_char);
                            count += 1;
                            break;
                        }
                        current = current.find_child_mut(&key, depth).unwrap();
                        depth += 1;
                        continue;
                    }

                    // create a new node to split at current_prefix_len
                    let mut node4 = Node4::new();
                    node4.meta.prefix_len = current_prefix_len;
                    node4.meta.partial = current.partial()[..min(current_prefix_len, MAX_PREFIX)]
                        .iter()
                        .copied()
                        .collect();

                    // fix up current node
                    let new_prefix_len = current.prefix_len() - (current_prefix_len + 1);
                    current.set_prefix_len(new_prefix_len);
                    if current_prefix_len <= MAX_PREFIX {
                        let new_partial = current
                            .partial()
                            .iter()
                            .skip(current_prefix_len + 1)
                            .copied()
                            .take(min(current.prefix_len(), MAX_PREFIX))
                            .collect();
                        // extract the key char before munging the partial
                        let key_char = current.partial()[current_prefix_len];
                        current.set_partial(new_partial);

                        // place old current as a child under
                        let old_node = replace(&mut *current, Node::Node4(node4));
                        current.add_child(Box::new(old_node), Some(key_char));
                    } else {
                        let leaf = current.minimum();
                        let (key_char, new_partial) = if let Node::Leaf(leaf) = leaf {
                            let new_partial: Vec<u8> = leaf
                                .key
                                .iter()
                                .skip(depth + current_prefix_len)
                                .take(min(current.prefix_len(), MAX_PREFIX))
                                .map(|e| *e)
                                .collect();
                            let key_char = leaf.key[depth + current_prefix_len];
                            (key_char, new_partial)
                        } else {
                            panic!("Should not be here");
                        };
                        current.set_partial(new_partial);
                        // place old current as a child under
                        let old_node = replace(&mut *current, Node::Node4(node4));
                        current.add_child(Box::new(old_node), Some(key_char));
                    }

                    let key_char = key[depth + current_prefix_len];
                    current.add_child(Box::new(Node::Leaf(Leaf::new(key, value))), Some(key_char));
                    count += 1;
                    break;
                }
            }
        }

        self.size += count;
    }

    fn calculate_partial(key: &Vec<u8>, depth: usize, prefix_len: usize) -> Vec<u8> {
        let mut partial: Vec<u8> = Vec::new();
        let max_partial = min(prefix_len, MAX_PREFIX);

        for (i, key) in key.iter().skip(depth).enumerate() {
            if i >= max_partial {
                break;
            }
            partial.push(*key);
        }
        partial
    }

    fn longest_common_prefix(key1: &Vec<u8>, key2: &Vec<u8>, depth: usize) -> usize {
        let max_compare = min(key1.len(), key2.len());
        let mut prefix_len = depth;

        for i in depth..max_compare {
            let i = i as usize;
            if key1[i] != key2[i] {
                break;
            }
            prefix_len += 1;
        }
        prefix_len - depth
    }
}

impl Node {
    fn prefix_match(&self, key: &[u8], depth: usize) -> usize {
        // match from depth..max_match_len
        let max_match_len = min(min(MAX_PREFIX, self.partial().len()), key.len() - depth);
        self.match_key(key, max_match_len, depth).unwrap_or(0)
    }

    fn prefix_match_deep(&self, key: &[u8], depth: usize) -> usize {
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

    fn minimum(&self) -> &Node {
        let mut tmp_node = self;
        loop {
            match tmp_node {
                Node::Leaf(_) => {
                    return tmp_node;
                }
                Node::Node4(node4) => match node4.children.get(&None) {
                    Some(node) => {
                        tmp_node = node.borrow();
                        continue;
                    }
                    None => {
                        let mut v = node4
                            .children
                            .keys()
                            .copied()
                            .collect::<Vec<Option<u8>>>();
                        v.sort_unstable();
                        tmp_node = node4.children.get(v.first().unwrap()).unwrap();
                    }
                },
                Node::None => {
                    panic!("should not be here");
                }
            }
        }
    }
    fn set_prefix_len(&mut self, new_prefix_len: usize) {
        match self {
            Node::Node4(node4) => {
                node4.meta.prefix_len = new_prefix_len;
            }
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    fn set_partial(&mut self, new_partial: Vec<u8>) {
        match self {
            Node::Node4(node4) => {
                node4.meta.partial = new_partial;
            }
            _ => {
                panic!("Prefix len is not applicable for node of this type");
            }
        }
    }

    fn add_child(&mut self, node: Box<Node>, key_char: Option<u8>) {
        match self {
            Node::Node4(node4) => {
                node4.add_child(node, key_char);
            }
            _ => {}
        }
    }

    fn child_exists(&self, key: &[u8], depth: usize) -> bool {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(key_char) = key.get(depth) {
                    node4.children.contains_key(&Some(*key_char))
                } else if key.len() == depth {
                    node4.children.contains_key(&None)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn find_child(&self, key: &[u8], depth: usize) -> Option<&Node> {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(ch) = key.get(depth) {
                    if let Some(child_node) = node4.children.get(&Some(*ch)) {
                        Some(child_node)
                    } else {
                        None
                    }
                } else if depth == key.len() {
                    if let Some(child_node) = node4.children.get(&None) {
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

    fn find_child_mut(&mut self, key: &[u8], depth: usize) -> Option<&mut Node> {
        // find the child that corresponds to key[depth]
        match self {
            Node::Node4(node4) => {
                // if key exists
                if let Some(ch) = key.get(depth) {
                    if let Some(child_node) = node4.children.get_mut(&Some(*ch)) {
                        Some(child_node.deref_mut())
                    } else {
                        None
                    }
                } else if key.len() == depth {
                    if let Some(child_node) = node4.children.get_mut(&None) {
                        Some(child_node.deref_mut())
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

    fn prefix_len(&self) -> usize {
        match self {
            Node::Node4(node) => node.prefix_len(),
            _ => 0,
        }
    }

    fn partial(&self) -> &[u8] {
        match self {
            Node::Node4(node) => node.partial(),
            _ => unimplemented!(),
        }
    }

    fn match_key(&self, key: &[u8], max_match_len: usize, depth: usize) -> Option<usize> {
        match self {
            Node::Node4(node) => node.match_key(key, max_match_len, depth),
            _ => unimplemented!(),
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
            children: HashMap::new(),
        }
    }

    fn partial(&self) -> &[u8] {
        &self.meta.partial
    }

    fn prefix_len(&self) -> usize {
        self.meta.prefix_len
    }

    fn match_key(&self, key: &[u8], max_match_len: usize, depth: usize) -> Option<usize> {
        let one = &self.meta.partial[0..max_match_len];
        let two = &key[depth..];
        ne_idx(one, two)

//        let mut idx = 0;
//        while idx < max_match_len {
//            if self.meta.partial[idx] != key[depth + idx] {
//                return Some(idx);
//            }
//            idx += 1;
//        }
//        Some(idx)
    }

    fn add_child(&mut self, node: Box<Node>, key_char: Option<u8>) {
        self.children.insert(key_char, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hashbrown::HashMap;
    use std::collections::VecDeque;
    use std::fs::{read, File};
    use std::io::{BufRead, BufReader};

    fn _insert(art: &mut Art, items: &Vec<&str>) {
        items.iter().for_each(|item| {
            art.insert(Vec::from(item.as_bytes()), Vec::from(item.as_bytes()));
            print_art(&art);
            println!("{}", "=".repeat(10));
        });
    }

    fn _insert_with_key_fn(art: &mut Art, items: &Vec<&str>, key_fn: fn(u8) -> u8) {
        items.iter().for_each(|item| {
            let x: Vec<u8> = item.as_bytes().iter().map(|x| key_fn(*x)).collect();
            art.insert(Vec::from(item.as_bytes()), x);
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
        // TODO add test cases for these cases
        let items = [
            "Congo",
            "Congregationalist",
            "Congregationalist's",
            "Congregationalists",
        ]
            .to_vec();
        //        let items = ["Ac", "Acropolis", "Acrux"].to_vec();
        //        let items = ["A", "AMD", "AMDs"].to_vec();
        _insert(&mut art, &items);

        for item in items.iter() {
            let res = art.search(&item.as_bytes().to_vec());
            if res.is_some() {
                let st = std::str::from_utf8(res.unwrap()).unwrap();
                println!("&st = {:#?}", &st);
            }
        }

        //        assert_eq!(art.len(), 2);
        //
        //        if let Node::Node4(node) = &art.root.borrow() {
        //            assert_eq!(node.meta.partial.len(), 1);
        //            assert_eq!(node.meta.partial, "A".as_bytes().to_vec());
        //            assert_eq!(node.children.len(), 2);
        //
        //            // all nodes should be of type leaf
        //            for (_, child) in node.children.iter() {
        //                match child.borrow() {
        //                    Node::Leaf(_) => {}
        //                    _ => panic!(" Node should be of type leaf"),
        //                }
        //            }
        //
        //            let A = node.children.get(&None);
        //            let M = node.children.get(&Some(*"M".as_bytes().first().unwrap()));
        //
        //            assert!(A.is_some());
        //            assert!(M.is_some());
        //        } else {
        //            // node is not of type node4 so fail
        //            panic!("Node should be of type node4 {:#?}", &art.root);
        //        }
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
            assert_eq!(
                node.meta.partial,
                "A".repeat(MAX_PREFIX).as_bytes().to_vec()
            );
            assert_eq!(node.children.len(), 2);

            // all nodes should be of type leaf
            for (_, child) in node.children.iter() {
                match child.borrow() {
                    Node::Leaf(_) => {}
                    _ => panic!(" Node should be of type leaf"),
                }
            }

            let A = node.children.get(&None);
            let M = node
                .children
                .get(&Some(*"A".repeat(20).as_bytes().first().unwrap()));

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
            assert_eq!(
                node.meta.partial,
                ['A', 'M'].iter().map(|c| *c as u8).collect::<Vec<u8>>()
            );
            let keys: Vec<Option<u8>> = node.children.keys().map(|i| *i).collect();
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

    #[test]
    fn test_insert_node4_same_prefix_as_existing_next_level() {
        let mut art = Art::new();
        let items = vec!["A", "AMD", "AMDs"];
        _insert(&mut art, &items);
        // size of the trie should be 3
        assert_eq!(art.len(), 3);

        // check the new nodes position and prefix relocation
        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.children.len(), 2);
            assert_eq!(node.meta.prefix_len, 1);
            assert_eq!(node.meta.partial.len(), 1);
            assert_eq!(
                node.meta.partial,
                ['A'].iter().map(|c| *c as u8).collect::<Vec<u8>>()
            );

            let keys: Vec<Option<u8>> = node.children.keys().map(|i| *i).collect();
            assert_eq!(keys, vec![None, Some('M' as u8)]);

            let m_node = node.children.get(&Some('M' as u8));
            assert!(m_node.is_some());
            let m_node = m_node.unwrap().borrow();

            if let Node::Node4(node) = m_node {
                assert_eq!(node.children.len(), 2);
                assert_eq!(node.meta.prefix_len, 1);
                assert_eq!(node.meta.partial.len(), 1);
                assert_eq!(
                    node.meta.partial,
                    ['D'].iter().map(|c| *c as u8).collect::<Vec<u8>>()
                );

                let keys: Vec<Option<u8>> = node.children.keys().map(|i| *i).collect();
                assert_eq!(keys, vec![None, Some('s' as u8)]);
            } else {
                panic!("m_node should be a node4");
            }
        } else {
            // node is not of type node4 so fail
            panic!("Node should be of type node4 {:#?}", &art.root);
        }
    }

    #[test]
    fn test_longest_common_prefix() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![1, 2, 3, 4];
        let depth = 0;

        let prefix_len = Art::longest_common_prefix(&v1, &v2, depth);
        let partial = Art::calculate_partial(&v1, depth, prefix_len);
        assert_eq!(prefix_len, 3);
        assert_eq!(partial, vec![1, 2, 3]);

        let depth = 1;
        let prefix_len = Art::longest_common_prefix(&v1, &v2, depth);
        let partial = Art::calculate_partial(&v1, depth, prefix_len);
        assert_eq!(prefix_len, 2);
        assert_eq!(partial, vec![2, 3]);

        let v1 = vec![1];
        let v2 = vec![1, 2, 3, 4];
        let depth = 0;
        let prefix_len = Art::longest_common_prefix(&v1, &v2, depth);
        let partial = Art::calculate_partial(&v1, depth, prefix_len);
        assert_eq!(prefix_len, 1);
        assert_eq!(partial, vec![1]);

        let a = "Acropolis".to_string().as_bytes().to_owned();
        let b = "Acrux".to_string().as_bytes().to_owned();
        let max_depth = min(a.len(), b.len());
        let mut depth = 0;
        let prefix_len = Art::longest_common_prefix(&a, &b, depth);
        let partial = Art::calculate_partial(&a, depth, prefix_len);

        assert_eq!(prefix_len, 3);
        assert_eq!(partial, "Acr".as_bytes());
    }

    #[test]
    fn test_insert_with_different_char_when_prefix_match_exists() {
        let mut art = Art::new();
        let items = vec!["A", "AMD", "ABDs"];
        _insert(&mut art, &items);
        assert_eq!(art.len(), 3);

        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.children.len(), 3);
            assert_eq!(node.meta.partial.len(), 1);
            assert_eq!(node.meta.prefix_len, 1);

            let keys: Vec<Option<u8>> = node.children.keys().map(|i| *i).collect();
            assert_eq!(keys, vec![None, Some('B' as u8), Some('M' as u8)]);
        } else {
            panic!("Should be a node 4");
        }
    }

    #[test]
    fn test_update_of_existing_keys() {
        let mut art = Art::new();
        let items = vec!["A", "AMD", "ABDs"];
        _insert(&mut art, &items);
        // use this function to transform the value
        let c_fn = |c: u8| c + 1;
        _insert_with_key_fn(&mut art, &items, c_fn);
        assert_eq!(art.len(), 3);

        if let Node::Node4(node) = &art.root.borrow() {
            assert_eq!(node.children.len(), 3);
            assert_eq!(node.meta.partial.len(), 1);
            assert_eq!(node.meta.prefix_len, 1);

            let keys: Vec<Option<u8>> = node.children.keys().map(|i| *i).collect();
            assert_eq!(keys, vec![None, Some('B' as u8), Some('M' as u8)]);

            for key in keys {
                let value = node.children.get(&key);
                if let Node::Leaf(leaf) = value.unwrap().borrow() {
                    let x: Vec<u8> = leaf.key.iter().map(|x| c_fn(*x)).collect();
                    assert_eq!(leaf.value, x);
                } else {
                    panic!("Should be a leaf ");
                }
            }
        } else {
            panic!("Should be a node 4");
        }
    }

    #[test]
    fn test_grow() {
        let mut art = Art::new();
        let mut items = VecDeque::new();
        for i in 1..100 {
            items.push_front(i as u8);
        }
        for item in items {
            let item = Vec::from(vec![item]);
            art.insert(item.clone(), item);
        }

        dbg!(art);
    }

    fn print_art(art: &Art) {
        let to_string = |digits: &Vec<u8>| -> String {
            let mut buffer = String::new();
            digits.iter().for_each(|c| {
                buffer.push(*c as char);
            });
            buffer
        };

        // dump tree
        let mut stack = Vec::new();
        stack.push((0 as i8, art.root.borrow()));
        let mut indent = 0;
        loop {
            let (current_char, current) = match stack.pop() {
                Some(item) => {
                    if item.0 == -1 {
                        indent -= 5;
                        continue;
                    }
                    item
                }
                None => break,
            };

            match current {
                Node::Leaf(leaf) => {
                    println!(
                        "{tag:>indent$}Leaf: {key:?} = {val:?}",
                        indent = indent,
                        tag = "",
                        key = to_string(&leaf.key),
                        val = to_string(&leaf.value)
                    );
                }
                Node::Node4(node4) => {
                    // print node metadata
                    println!(
                        "{tag:>indent$} char={char} Node4({clen}) {keys:?} - ({plen}) [{partial:?}]",
                        indent = indent,
                        tag = "",
                        char = current_char as u8 as char,
                        clen = node4.children.len(),
                        keys = &node4.children.keys().map(|k| {
                            match k {
                                Some(v) => *v,
                                None => 0
                            }
                        }).map(|k| k as char).collect::<Vec<char>>(),
                        plen = node4.meta.prefix_len,
                        partial = &node4.meta.partial.iter().map(|c| *c as char).collect::<Vec<char>>()
                    );

                    if node4.meta.prefix_len < MAX_PREFIX
                        && node4.meta.partial.len() != node4.meta.prefix_len
                    {
                        eprintln!(
                            "{tag:>indent$} Error: partial len does not match prefix len",
                            indent = indent,
                            tag = ""
                        );
                    }

                    // push a marker for dealing with indentation
                    indent += 5;
                    let x: i8 = -1;
                    stack.push((x, &Node::None));

                    //queue up the nodes for visiting
                    for (character, child_node) in node4.children.iter() {
                        stack.push((character.unwrap_or(0) as i8, child_node.borrow()));
                    }
                }
                Node::None => {
                    dbg!("should not be here");
                    break;
                }
            }
        }
    }

    #[test]
    fn test_small_batch_insert() {
        let f_name = "/tmp/words.txt";
        let mut art = Art::new();
        insert_from_file(&mut art, f_name);
        // TODO add asserts
    }

    #[test]
    fn test_search() {
        let f_name = "/tmp/words.txt";
        let mut art = Art::new();
        insert_from_file(&mut art, f_name);

        // check if you can find all the words
        let fil = File::open(f_name).unwrap();
        let reader = BufReader::new(fil);
        for line in reader.lines() {
            if line.is_ok() {
                let line = line.unwrap();
                let line = line.trim();
                let res = art.search(&line.as_bytes().to_vec());
                if res.is_none() {
                    println!("could not find {}", line);
                }
            }
        }
    }

    #[test]
    fn test_simple_search() {
        let mut art = Art::new();
        let items = vec!["A", "AMD", "AMDs"];
        _insert(&mut art, &items);

        for item in items {
            let res = art.search(&item.as_bytes().to_vec());
            println!("res = {:#?}", res);
        }
    }

    fn insert_from_file(art: &mut Art, f_name: &str) {
        let fil = File::open(f_name).unwrap();
        let mut reader = BufReader::new(fil);
        loop {
            let mut buffer = String::new();
            let x = reader.read_line(&mut buffer);
            if x.is_err() || buffer.is_empty() {
                break;
            }
            art.insert(
                buffer.trim().clone().as_bytes().to_vec(),
                buffer.trim().clone().as_bytes().to_vec(),
            );
        }
    }

    #[test]
    fn test_simd_string_match() {
        let a = "david".to_string();
        let b = "brainard".to_string();
        let c = "davidbrainard".to_string();
        let d = "davibrainard".to_string();

        let res = Art::equals(a.as_bytes(), a.as_bytes());
        assert_eq!(true, res);

        let res = Art::equals(a.as_bytes(), b.as_bytes());
        assert_eq!(false, res);

        let res = Art::equals(a.as_bytes(), c.as_bytes());
        assert_eq!(false, res);

        let res = Art::equals(a.as_bytes(), d.as_bytes());
        assert_eq!(false, res);
    }
}
