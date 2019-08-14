use crate::Leaf;

impl Leaf {
    pub(crate) fn new(new_key: Vec<u8>, new_value: Vec<u8>) -> Self {
        Leaf {
            key: new_key,
            value: new_value,
        }
    }

    pub(crate) fn key_char(&self, depth: usize) -> usize {
        if self.key.len() - 1 < depth {
            // TODO fix this
            0
        } else {
            self.key[depth] as usize
        }
    }
}
