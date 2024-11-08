use std::time::Duration;

use anyhow::{
    anyhow,
    Context,
};
use cs2_schema_cutl::{
    CStringUtil,
    PtrCStr,
};
use obfstr::obfstr;
use raw_struct::{
    FromMemoryView,
    Reference,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::CModel,
    CS2Handle,
    StateCS2Handle,
    StateCS2Memory,
};

pub enum BoneFlags {
    FlagNoBoneFlags = 0x0,
    FlagBoneflexdriver = 0x4,
    FlagCloth = 0x8,
    FlagPhysics = 0x10,
    FlagAttachment = 0x20,
    FlagAnimation = 0x40,
    FlagMesh = 0x80,
    FlagHitbox = 0x100,
    FlagBoneUsedByVertexLod0 = 0x400,
    FlagBoneUsedByVertexLod1 = 0x800,
    FlagBoneUsedByVertexLod2 = 0x1000,
    FlagBoneUsedByVertexLod3 = 0x2000,
    FlagBoneUsedByVertexLod4 = 0x4000,
    FlagBoneUsedByVertexLod5 = 0x8000,
    FlagBoneUsedByVertexLod6 = 0x10000,
    FlagBoneUsedByVertexLod7 = 0x20000,
    FlagBoneMergeRead = 0x40000,
    FlagBoneMergeWrite = 0x80000,
    FlagAllBoneFlags = 0xfffff,
    BlendPrealigned = 0x100000,
    FlagRigidlength = 0x200000,
    FlagProcedural = 0x400000,
}

#[derive(Debug, Clone, Default)]
pub struct Bone {
    pub name: String,
    pub flags: u32,
    pub parent: Option<usize>,
}

#[derive(Debug, Default)]
pub struct CS2Model {
    pub name: String,
    pub bones: Vec<Bone>,

    pub vhull_min: nalgebra::Vector3<f32>,
    pub vhull_max: nalgebra::Vector3<f32>,

    pub vview_min: nalgebra::Vector3<f32>,
    pub vview_max: nalgebra::Vector3<f32>,
}

impl State for CS2Model {
    type Parameter = u64;

    fn create(states: &StateRegistry, address: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let cs2 = states.resolve::<StateCS2Handle>(())?;
        let mut result: Self = Default::default();

        let name = PtrCStr::read_object(memory.view(), address + 0x08)
            .map_err(|e| anyhow!(e))?
            .read_string(memory.view())?
            .context("model name nullptr")?;

        result.name = name;
        log::debug!(
            "{} {} at {:X}. Caching.",
            obfstr!("Reading player model"),
            result.name,
            address
        );

        result.do_read(&cs2, address)?;
        Ok(result)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Timed(Duration::from_secs(60))
    }
}

impl CS2Model {
    fn do_read(&mut self, cs2: &CS2Handle, address: u64) -> anyhow::Result<()> {
        let memory = cs2.create_memory_view();
        let memory_view = &*memory;

        [
            self.vhull_min,
            self.vhull_max,
            self.vview_min,
            self.vview_max,
        ] = cs2.read_sized::<[nalgebra::Vector3<f32>; 4]>(address + 0x18)?;

        let model = Reference::<dyn CModel>::new(memory.clone(), address);
        let bone_count = model.bone_count()? as usize;
        if bone_count > 6000 {
            anyhow::bail!(
                "{} ({})",
                obfstr!("model contains too many bones"),
                bone_count
            );
        }

        log::trace!("Reading {} bones", bone_count);
        let model_bone_flags = model.bone_flags()?.elements(memory_view, 0..bone_count)?;
        let model_bone_parent_index = model.bone_parents()?.elements(memory_view, 0..bone_count)?;
        let bone_names_ptr = model.bone_names()?.elements(memory_view, 0..bone_count)?;

        self.bones.clear();
        self.bones.reserve(bone_count);
        for bone_index in 0..bone_count {
            let name = bone_names_ptr[bone_index]
                .read_string(memory_view)?
                .context("missing bone name")?;
            let parent_index = model_bone_parent_index[bone_index];
            let flags = model_bone_flags[bone_index];

            self.bones.push(Bone {
                name: name.clone(),
                parent: if parent_index as usize >= bone_count {
                    None
                } else {
                    Some(parent_index as usize)
                },
                flags,
            });
        }
        Ok(())
    }
}

impl Drop for CS2Model {
    fn drop(&mut self) {
        log::debug!("Removing cached model {}", self.name);
    }
}
