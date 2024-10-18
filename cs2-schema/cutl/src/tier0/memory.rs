use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

#[raw_struct(size = 0x10)]
pub struct CUtlMemory<T>
where
    T: Send + Sync + 'static,
{
    #[field(offset = 0x00)]
    pub buffer: Ptr64<[T]>,

    #[field(offset = 0x08)]
    pub allocation_count: u32,

    #[field(offset = 0x0C)]
    pub grow_size: u32,
}
