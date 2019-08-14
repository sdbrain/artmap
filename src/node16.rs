use crate::Node16;

impl Node16 {
    pub(crate) fn max_leaf_index(&self) -> usize {
        // 0-15 elems + 1 leaf
        15 + 1
    }
}