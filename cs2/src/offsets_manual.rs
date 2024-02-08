//! Manual defined offsets which can not be deducted by the CS2 schema.

pub mod client {
    // Sig source: https://www.unknowncheats.me/forum/3725362-post1.html
    // https://www.unknowncheats.me/forum/3713485-post262.html
    #[allow(non_snake_case)]
    pub mod CModel {
        /* 85 D2 78 16 3B 91. Offset is array of u32 */
        pub const BONE_FLAGS: u64 = 0x1B0;

        /* 85 D2 78 25 3B 91. Offset is array of *const i8 */
        pub const BONE_NAME: u64 = 0x168;

        /* UC sig does not work. Offset is array of u16 */
        pub const BONE_PARENT: u64 = 0x180;
    }
}
