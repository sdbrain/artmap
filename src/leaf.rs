use crate::Leaf;
use std::borrow::Borrow;
use std::fmt::{Display, Error, Formatter};

impl Leaf {
    pub(crate) fn new(new_key: Vec<u8>, new_value: Vec<u8>) -> Self {
        Leaf {
            key: new_key,
            value: new_value,
        }
    }
}

impl Display for Leaf {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let as_char_vec = |vec: &Vec<u8>| -> Vec<char> { vec.iter().map(|i| *i as char).collect() };
        write!(
            f,
            "Leaf: {:?}={:?}",
            as_char_vec(self.key.borrow()),
            as_char_vec(self.value.borrow())
        )
    }
}
