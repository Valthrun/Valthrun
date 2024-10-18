use std::marker::{
    self,
};

use raw_struct::{
    builtins::{
        Array,
        Ptr64,
    },
    raw_struct,
};

#[raw_struct(size = 0x20)]
pub struct UtlRBTree<T>
where
    T: marker::Copy + Send + Sync + 'static,
{
    #[field(offset = 0x00)]
    pub elements: Ptr64<dyn Array<dyn UtlRBTreeNode<T>>>,

    // pub element_capacity: u16 = 0x08,
    #[field(offset = 0x16)]
    pub highest_entry: u16,
}

#[raw_struct(memory = "([u8; 0x08], T)")]
pub struct UtlRBTreeNode<T>
where
    T: marker::Copy + Send + Sync + 'static,
{
    #[field(offset = 0x00)]
    pub left: i16,

    #[field(offset = 0x02)]
    pub right: i16,

    #[field(offset = 0x04)]
    pub parent: i16,

    #[field(offset = 0x06)]
    pub tag: i16,

    #[field(offset = 0x08)]
    pub value: T,
}
