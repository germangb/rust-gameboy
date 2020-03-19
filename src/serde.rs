use serde::{Deserializer, Serializer};

pub fn ser_boxed_8k<S: Serializer>(t: &Box<[u8; 0x2000]>, ser: S) -> Result<S::Ok, S::Error> {
    unimplemented!()
}

pub fn de_boxed_8k<'de, D: Deserializer<'de>>(de: D) -> Result<Box<[u8; 0x2000]>, D::Error> {
    unimplemented!()
}

pub fn ser_vec_8k<S: Serializer>(t: &Vec<[u8; 0x2000]>, ser: S) -> Result<S::Ok, S::Error> {
    unimplemented!()
}

pub fn de_vec_8k<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<[u8; 0x2000]>, D::Error> {
    unimplemented!()
}
