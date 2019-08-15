use crate::Node256;

impl Node256 {
    pub(crate) fn max_leaf_index(&self) -> usize {
        // 0-255 + 1 for leaf
        255 + 1
    }
}
