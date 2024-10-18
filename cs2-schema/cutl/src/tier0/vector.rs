use raw_struct::{
    builtins::Ptr64,
    raw_struct,
};

#[raw_struct(size = 0x10)]
pub struct CUtlVector<T>
where
    T: Send + Sync + 'static,
{
    #[field(offset = 0x00)]
    pub size: u32,

    #[field(offset = 0x08)]
    pub data: Ptr64<[T]>,
}
