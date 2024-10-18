use anyhow::anyhow;
use cs2_schema_cutl::PtrCStr;
use cs2_schema_generated::cs2::client::CModelState;
use raw_struct::{
    builtins::{
        Array,
        Ptr64,
    },
    raw_struct,
    FromMemoryView,
};

// Sig source: https://www.unknowncheats.me/forum/3725362-post1.html
// https://www.unknowncheats.me/forum/3713485-post262.html
#[raw_struct(size = 0x200)]
pub struct CModel {
    #[field(offset = 0x160)]
    pub bone_count: u32,

    #[field(offset = 0x168)]
    pub bone_names: Ptr64<[PtrCStr]>,

    /* UC sig does not work */
    #[field(offset = 0x180)]
    pub bone_parents: Ptr64<[u16]>,

    /* 85 D2 78 16 3B 91 */
    #[field(offset = 0x1B0)]
    pub bone_flags: Ptr64<[u32]>,
}

#[raw_struct(size = 0x20)]
pub struct CBoneStateData {
    #[field(offset = 0x00)]
    pub position: [f32; 3],

    #[field(offset = 0x0C)]
    pub scale: f32,

    #[field(offset = 0x10)]
    pub rotation: [f32; 4],
}

pub trait CModelStateEx {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr64<Ptr64<()>>>;
    fn bone_state_data(&self) -> anyhow::Result<Ptr64<dyn Array<dyn CBoneStateData>>>;
}

impl CModelStateEx for dyn CModelState {
    fn m_hModel(&self) -> anyhow::Result<Ptr64<Ptr64<()>>> {
        Ptr64::read_object(self.object_memory(), 0xA0).map_err(|e| anyhow!(e))
    }

    fn bone_state_data(&self) -> anyhow::Result<Ptr64<dyn Array<dyn CBoneStateData>>> {
        Ptr64::read_object(self.object_memory(), 0x80).map_err(|e| anyhow!(e))
    }
}
