use anyhow::Context;
use cs2_schema_declaration::{
    define_schema,
    PtrCStr,
};
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CS2HandleState,
    CS2Offsets,
};

define_schema! {
    pub struct EngineBuildInfo[0x28] {
        pub revision: PtrCStr = 0x00,
        pub build_date: PtrCStr = 0x08,
        pub build_time: PtrCStr = 0x10,
        /* pub unknown_zero: u64 */
        pub product_name: PtrCStr = 0x20,
    }

    pub struct Globals[0x50] {
        pub time_1: f32 = 0x00,
        pub frame_count_1: u32 = 0x04,

        pub max_player_count: u32 = 0x10,

        pub time_2: f32 = 0x34,
        pub time_3: f32 = 0x38,

        pub frame_count_2: u32 = 0x48,
        pub two_tick_time: f32 = 0x4C,
    }
}

impl State for Globals {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let cs2 = states.resolve::<CS2HandleState>(())?;
        let offsets = states.resolve::<CS2Offsets>(())?;

        cs2.reference_schema::<Globals>(&[offsets.globals, 0])?
            .cached()
            .with_context(|| obfstr!("failed to read globals").to_string())
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
