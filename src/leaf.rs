use crate::Leaf;

impl Leaf {
    pub(crate) fn new(new_key: Vec<u8>, new_value: Vec<u8>) -> Self {
        Leaf {
            key: new_key,
            value: new_value,
        }
    }
}
