use cs2_schema_cutl::PtrCStr;
use raw_struct::{
    builtins::{
        Array,
        Ptr64,
    },
    raw_struct,
};

#[raw_struct(size = 0x48)]
pub struct ConVar {
    #[field(offset = 0x00)]
    pub name: PtrCStr,

    #[field(offset = 0x20)]
    pub description: PtrCStr,

    #[field(offset = 0x2C)]
    pub n_change_count: u32,

    #[field(offset = 0x40)]
    pub n_value: u32,

    #[field(offset = 0x48)]
    pub n_value_min: u32,

    #[field(offset = 0x50)]
    pub n_value_default: u32,

    #[field(offset = 0x40)]
    pub fl_value: f32,

    #[field(offset = 0x48)]
    pub fl_value_min: f32,

    #[field(offset = 0x50)]
    pub fl_value_default: f32,
}

#[raw_struct(size = 0x10)]
pub struct CCVarEntry {
    #[field(offset = 0x00)]
    pub value: Ptr64<dyn ConVar>,
}

#[raw_struct(size = 0x10)]
pub struct CCVar {
    #[field(offset = 0x40)]
    pub entries: Ptr64<dyn Array<dyn CCVarEntry>>,

    #[field(offset = 0x48)]
    pub entries_capacity: u64,

    #[field(offset = 0x52)]
    pub entries_count: u16,
}
