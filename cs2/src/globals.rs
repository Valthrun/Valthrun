use crate::PtrCStr;


#[repr(C)]
#[derive(Debug, Default)]
pub struct Globals {
    // Some time which
    pub time_1: f32,
    pub frame_count_1: u32,

    unknown_0: f32,
    unknown_1: f32,
    unknown_2: f32,
    unknown_3: u32,
    unknown_4: u32,
    unknown_5: f32,

    unknown_6: u64, // Some function
    unknown_7: f32,
    
    pub time_2: f32,
    pub time_3: f32,
    
    unknown_8: f32,
    unknown_9: f32,
    unknown_10: u32,
    pub frame_count_2: u32,
    pub two_tick_time: f32, // Assuming CS runs on 128 tick
}
const _: [u8; 0x48] = [0; std::mem::size_of::<Globals>()];

#[repr(C)]
pub struct EngineBuildInfo {
    pub revision: PtrCStr,
    pub build_date: PtrCStr,
    pub build_time: PtrCStr,
    unknown_zero: u64,
    pub product_name: PtrCStr,
}