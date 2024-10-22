use cs2_schema_cutl::PtrCStr;
use raw_struct::raw_struct;

#[raw_struct(size = 0x28)]
pub struct EngineBuildInfo {
    #[field(offset = 0x00)]
    pub revision: PtrCStr,

    #[field(offset = 0x08)]
    pub build_date: PtrCStr,

    #[field(offset = 0x10)]
    pub build_time: PtrCStr,

    /* pub unknown_zero: u64 */
    #[field(offset = 0x20)]
    pub product_name: PtrCStr,
}

#[raw_struct(size = 0x50)]
pub struct Globals {
    #[field(offset = 0x00)]
    pub time_1: f32,

    #[field(offset = 0x04)]
    pub frame_count_1: u32,

    #[field(offset = 0x10)]
    pub max_player_count: u32,

    #[field(offset = 0x34)]
    pub time_2: f32,

    #[field(offset = 0x38)]
    pub time_3: f32,

    #[field(offset = 0x48)]
    pub frame_count_2: u32,

    #[field(offset = 0x4C)]
    pub two_tick_time: f32,
}
