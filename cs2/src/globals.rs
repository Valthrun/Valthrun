use cs2_schema_declaration::define_schema;

define_schema! {
    pub struct Globals[0x48] {
        pub time_1: f32 = 0x00,
        pub frame_count_1: u32 = 0x04,

        pub max_player_count: u32 = 0x10,

        pub time_2: f32 = 0x2C,
        pub time_3: f32 = 0x30,

        pub frame_count_2: u32 = 0x40,
        pub two_tick_time: f32 = 0x44,
    }
}
