use std::ffi::CStr;

use anyhow::{
    Context,
    Result,
};
use cs2_schema_declaration::{
    define_schema,
    Ptr,
};
use cs2_schema_generated::{
    cs2::client::{
        CCSPlayer_ItemServices,
        CModelState,
        CSkeletonInstance,
        C_CSPlayerPawn,
    },
    EntityHandle,
};
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
};

use crate::{
    CS2Model,
    EntitySystem,
    WeaponId,
};

#[derive(Debug, Clone)]
pub struct PlayerPawnInfo {
    pub controller_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_has_defuser: bool,
    pub player_name: String,
    pub weapon: WeaponId,
    pub player_flashtime: f32,

    pub position: nalgebra::Vector3<f32>,
    pub rotation: f32,

    pub model_address: u64,
    pub bone_states: Vec<BoneStateData>,
}

define_schema! {
    pub struct CBoneStateData[0x20] {
        pub position: [f32; 3] = 0x00,
        pub scale: f32 = 0x0C,
        pub rotation: [f32; 4] = 0x10,
    }
}

trait CModelStateEx {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>>;
    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>>;
}

impl CModelStateEx for CModelState {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>> {
        self.memory.reference_schema(0xA0)
    }

    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>> {
        self.memory.reference_schema(0x80)
    }
}

#[derive(Debug, Clone)]
pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
}

impl TryFrom<CBoneStateData> for BoneStateData {
    type Error = anyhow::Error;

    fn try_from(value: CBoneStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            position: nalgebra::Vector3::from_row_slice(&value.position()?),
        })
    }
}

#[derive(Debug, Clone)]
pub enum PlayerPawnState {
    Alive(PlayerPawnInfo),
    Dead,
}

impl State for PlayerPawnState {
    type Parameter = u32;

    fn create(
        states: &utils_state::StateRegistry,
        pawn_entity_index: Self::Parameter,
    ) -> anyhow::Result<Self> {
        let entities = states.resolve::<EntitySystem>(())?;

        let player_pawn = match entities
            .get_by_handle::<C_CSPlayerPawn>(&EntityHandle::from_index(pawn_entity_index))?
        {
            Some(identity) => identity
                .entity()?
                .read_schema()
                .with_context(|| obfstr!("failed to read player pawn data").to_string())?,
            None => return Ok(Self::Dead),
        };

        let player_health = player_pawn.m_iHealth()?;
        if player_health <= 0 {
            return Ok(Self::Dead);
        }

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .cast::<CSkeletonInstance>()
            .read_schema()?;
        if game_screen_node.m_bDormant()? {
            return Ok(Self::Dead);
        }

        let controller_handle = player_pawn.m_hController()?;
        let current_controller = entities.get_by_handle(&controller_handle)?;

        let player_team = player_pawn.m_iTeamNum()?;
        let player_name = if let Some(identity) = &current_controller {
            let player_controller = identity.entity()?.reference_schema()?;
            CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
                .context("player name missing nul terminator")?
                .to_str()
                .context("invalid player name")?
                .to_string()
        } else {
            /*
             * This is the case for pawns which are not controllel by a player controller.
             * An example would be the main screen player pawns.
             *
             * Note: We're assuming, that uncontroller player pawns are neglectable while being in a match as the do not occurr.
             * Bots (and controller bots) always have a player pawn controller.
             */
            // log::warn!(
            //     "Handle at address {:p} has no valid controller!",
            //     &controller_handle
            // );
            return Ok(Self::Dead);
        };

        let player_has_defuser = player_pawn
            .m_pItemServices()?
            .cast::<CCSPlayer_ItemServices>()
            .reference_schema()?
            .m_bHasDefuser()?;

        let position =
            nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

        let model_address = game_screen_node
            .m_modelState()?
            .m_hModel()?
            .read_schema()?
            .address()?;

        let model = states.resolve::<CS2Model>(model_address)?;
        let bone_states = game_screen_node
            .m_modelState()?
            .bone_state_data()?
            .read_entries(model.bones.len())?
            .into_iter()
            .map(|bone| bone.try_into())
            .collect::<Result<Vec<_>>>()?;

        let weapon = player_pawn.m_pClippingWeapon()?.try_read_schema()?;
        let weapon_type = if let Some(weapon) = weapon {
            weapon
                .m_AttributeManager()?
                .m_Item()?
                .m_iItemDefinitionIndex()?
        } else {
            WeaponId::Knife.id()
        };

        let player_flashtime = player_pawn.m_flFlashBangTime()?;

        Ok(Self::Alive(PlayerPawnInfo {
            controller_entity_id: controller_handle.get_entity_index(),
            team_id: player_team,

            player_name,
            player_has_defuser,
            player_health,
            weapon: WeaponId::from_id(weapon_type).unwrap_or(WeaponId::Unknown),
            player_flashtime,

            position,
            rotation: player_pawn.m_angEyeAngles()?[1],

            bone_states,
            model_address,
        }))
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
