use std::marker::PhantomData;

use cs2_schema_declaration::{
    MemoryHandle,
    Ptr,
    SchemaValue,
};

// UtlRBTree has the following layout:
// pub struct UtlRBTree<T> {
//    pub elements: Ptr<[UtlRBTreeNode<T>]> = 0x00,
//    pub element_capacity: u16 = 0x08,
//    pub highest_entry: u16 = 0x16,
// }
pub struct UtlRBTree<T> {
    memory: MemoryHandle,
    _dummy: PhantomData<T>,
}

impl<T> UtlRBTree<T> {
    pub fn elements(&self) -> anyhow::Result<Ptr<[UtlRBTreeNode<T>]>> {
        self.memory.reference_schema(0x00)
    }

    pub fn highest_entry(&self) -> anyhow::Result<u16> {
        self.memory.reference_schema(0x16)
    }
}

impl<T: SchemaValue> UtlRBTree<T> {
    pub fn value(&self) -> anyhow::Result<T> {
        self.memory.reference_schema(0x08)
    }
}

impl<T: SchemaValue> SchemaValue for UtlRBTree<T> {
    fn value_size() -> Option<u64> {
        Some(0x20)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            memory,
            _dummy: Default::default(),
        })
    }
}

/// UtlRBTreeNode has the following layout:
/// struct UtlRBTreeNode<T> {
///      pub left: i16,  // 0x0000
///      pub right: i16, // 0x0002
///      pub parent: i16 // 0x0004
///      pub tag: i16,   // 0x0006
///      pub value: T    // 0x0008
/// }
/// N is most likel 256 (u8) large
pub struct UtlRBTreeNode<T> {
    memory: MemoryHandle,
    _dummy: PhantomData<T>,
}

impl<T> UtlRBTreeNode<T> {
    pub fn left_node(&self) -> anyhow::Result<i16> {
        self.memory.reference_schema(0x00)
    }

    pub fn right_node(&self) -> anyhow::Result<i16> {
        self.memory.reference_schema(0x02)
    }

    pub fn parent_node(&self) -> anyhow::Result<i16> {
        self.memory.reference_schema(0x04)
    }

    pub fn tag(&self) -> anyhow::Result<i16> {
        self.memory.reference_schema(0x06)
    }
}

impl<T: SchemaValue> UtlRBTreeNode<T> {
    pub fn value(&self) -> anyhow::Result<T> {
        self.memory.reference_schema(0x08)
    }
}

impl<T: SchemaValue> SchemaValue for UtlRBTreeNode<T> {
    fn value_size() -> Option<u64> {
        Some(T::value_size().expect("T to have a size") + 0x08)
    }

    fn from_memory(memory: MemoryHandle) -> anyhow::Result<Self> {
        Ok(Self {
            memory,
            _dummy: Default::default(),
        })
    }
}
