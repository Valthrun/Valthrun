use std::{
    ffi::CStr,
    ops::Deref,
};

use anyhow::{
    Context,
    Result,
};
use cs2_schema_cutl::EntityHandle;
use cs2_schema_generated::cs2::client::{
    CCSPlayer_ItemServices,
    CGameSceneNode,
    CSkeletonInstance,
    C_BaseEntity,
    C_BasePlayerPawn,
    C_CSPlayerPawn,
    C_CSPlayerPawnBase,
    C_EconEntity,
};
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    schema::{
        CBoneStateData,
        CModelStateEx,
    },
    CS2Model,
    StateCS2Memory,
    StateEntityList,
    WeaponId,
};

#[derive(Debug, Clone)]
pub struct StatePawnInfo {
    pub controller_entity_id: Option<u32>,
    pub pawn_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_has_defuser: bool,
    pub player_name: Option<String>,
    pub weapon: WeaponId,
    pub player_flashtime: f32,

    pub position: nalgebra::Vector3<f32>,
    pub rotation: f32,
}

impl State for StatePawnInfo {
    type Parameter = EntityHandle<dyn C_CSPlayerPawn>;

    fn create(states: &StateRegistry, handle: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let Some(player_pawn) = entities.entity_from_handle(&handle) else {
            anyhow::bail!("entity does not exists")
        };
        let player_pawn = player_pawn
            .value_copy(memory.view())?
            .context("player pawn nullptr")?;

        let player_health = player_pawn.m_iHealth()?;

        let controller_handle = player_pawn.m_hController()?;
        let current_controller = entities.entity_from_handle(&controller_handle);

        let player_team = player_pawn.m_iTeamNum()?;
        let player_name = if let Some(identity) = &current_controller {
            let player_controller = identity
                .value_reference(memory.view_arc())
                .context("nullptr")?;
            Some(
                CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
                    .context("player name missing nul terminator")?
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            None
        };

        let player_has_defuser = player_pawn
            .m_pItemServices()?
            .value_reference(memory.view_arc())
            .context("m_pItemServices nullptr")?
            .cast::<dyn CCSPlayer_ItemServices>()
            .m_bHasDefuser()?;

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .value_reference(memory.view_arc())
            .context("game screen node nullptr")?
            .cast::<dyn CSkeletonInstance>()
            .copy()?;

        let position =
            nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

        let weapon = player_pawn
            .m_pClippingWeapon()?
            .value_reference(memory.view_arc());
        let weapon_type = if let Some(weapon) = weapon {
            weapon
                .m_AttributeManager()?
                .m_Item()?
                .m_iItemDefinitionIndex()?
        } else {
            WeaponId::Knife.id()
        };

        let player_flashtime = player_pawn.m_flFlashBangTime()?;

        Ok(Self {
            controller_entity_id: if controller_handle.is_valid() {
                Some(controller_handle.get_entity_index())
            } else {
                None
            },
            pawn_entity_id: handle.get_entity_index(),

            team_id: player_team,

            player_name,
            player_has_defuser,
            player_health,
            weapon: WeaponId::from_id(weapon_type).unwrap_or(WeaponId::Unknown),
            player_flashtime,

            position,
            rotation: player_pawn.m_angEyeAngles()?[1],
        })
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}

#[derive(Debug, Clone)]
pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
}

impl TryFrom<&dyn CBoneStateData> for BoneStateData {
    type Error = anyhow::Error;

    fn try_from(value: &dyn CBoneStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            position: nalgebra::Vector3::from_row_slice(&value.position()?),
        })
    }
}

#[derive(Debug, Clone)]
pub struct StatePawnModelInfo {
    pub model_address: u64,
    pub bone_states: Vec<BoneStateData>,
}

impl State for StatePawnModelInfo {
    type Parameter = EntityHandle<dyn C_CSPlayerPawn>;

    fn create(states: &StateRegistry, handle: Self::Parameter) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;
        let Some(player_pawn) = entities.entity_from_handle(&handle) else {
            anyhow::bail!("entity does not exists")
        };
        let player_pawn = player_pawn
            .value_copy(memory.view())?
            .context("player pawn nullptr")?;

        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .value_reference(memory.view_arc())
            .context("game screen node nullptr")?
            .cast::<dyn CSkeletonInstance>()
            .copy()?;

        let model_address = game_screen_node
            .m_modelState()?
            .m_hModel()?
            .read_value(memory.view())?
            .context("m_hModel nullptr")?
            .address;

        let model = states.resolve::<CS2Model>(model_address)?;
        let bone_states = game_screen_node
            .m_modelState()?
            .bone_state_data()?
            .elements_copy(memory.view(), 0..model.bones.len())?
            .into_iter()
            .map(|bone| bone.deref().try_into())
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            bone_states,
            model_address,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayerPawnState {
    Alive,
    Dead,
}

impl State for PlayerPawnState {
    type Parameter = EntityHandle<dyn C_CSPlayerPawn>;

    fn create(
        states: &utils_state::StateRegistry,
        handle: Self::Parameter,
    ) -> anyhow::Result<Self> {
        let memory = states.resolve::<StateCS2Memory>(())?;
        let entities = states.resolve::<StateEntityList>(())?;

        let player_pawn = match entities.entity_from_handle::<dyn C_CSPlayerPawn>(&handle) {
            Some(identity) => identity
                .value_reference(memory.view_arc())
                .context("entity nullptr")?,
            None => return Ok(Self::Dead),
        };

        let player_health = player_pawn.m_iHealth()?;
        if player_health <= 0 {
            return Ok(Self::Dead);
        }

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .value_reference(memory.view_arc())
            .context("m_pGameSceneNode nullptr")?
            .cast::<dyn CSkeletonInstance>();
        if game_screen_node.m_bDormant()? {
            return Ok(Self::Dead);
        }

        Ok(Self::Alive)
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
