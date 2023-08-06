use std::{sync::Arc, ffi::CStr};

use anyhow::Context;
use cs2::{Module, CS2Model, PCStrEx};
use cs2_schema::{cs2::client::{CSkeletonInstance, CModelState}, Ptr, SchemaValue, define_schema, MemoryHandle};
use obfstr::obfstr;

use crate::Application;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TeamType {
    Local,
    Enemy,
    Friendly
}

pub struct PlayerInfo {
    pub team_type: TeamType,

    pub player_health: i32,
    pub player_name: String,
    pub position: nalgebra::Vector3<f32>,
 
    pub debug_text: String,

    pub model: Arc<CS2Model>,
    pub bone_states: Vec<BoneStateData>,
}

pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
}

impl TryFrom<CBoneStateData> for BoneStateData {
    type Error = anyhow::Error;

    fn try_from(value: CBoneStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            position: nalgebra::Vector3::from_row_slice(&value.position()?)
        })
    }
}

define_schema! {
    pub struct CBoneStateData[0x20] {
        pub position: [f32; 3] = 0x00,
        pub scale: f32 = 0x0C,
        pub rotation: [f32; 4] = 0x10,
    }
}

trait CModelStateEx {
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>>;
    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>>;
}

impl CModelStateEx for CModelState {
    #[allow(non_snake_case)]
    fn m_hModel(&self) -> anyhow::Result<Ptr<Ptr<()>>> {
        SchemaValue::from_memory(&self.memory, self.offset + 160)
    }

    fn bone_state_data(&self) -> anyhow::Result<Ptr<[CBoneStateData]>> {
        SchemaValue::from_memory(&self.memory, self.offset + 0x80)
    }
}

pub fn read_player_info(ctx: &mut Application) -> anyhow::Result<()> {
    ctx.players.clear();
    ctx.players.reserve(16);

    let local_player_controller = ctx
        .cs2_entities
        .get_local_player_controller()?
        .reference_schema()
        .with_context(|| obfstr!("failed to read local player controller").to_string())?;

    let local_team = local_player_controller.m_iPendingTeamNum()?;

    let player_controllers = ctx.cs2_entities.get_player_controllers()?;
    for player_controller in player_controllers {
        let player_controller = player_controller.read_schema()?;
        
        let player_pawn = player_controller.m_hPlayerPawn()?;
        if !player_pawn.is_valid() {
            continue;
        }

        let player_pawn = ctx.cs2_entities.get_by_handle(&player_pawn)?
            .context("missing player pawn")?
            .read_schema()?;
        
        let player_health = player_pawn.m_iHealth()?;
        if player_health <= 0 {
            continue;
        }

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn.m_pGameSceneNode()?
            .cast::<CSkeletonInstance>()
            .reference_schema()?;
        if game_screen_node.m_bDormant()? {
            continue;
        }

        let player_team = player_controller.m_iTeamNum()?;
        let player_name = CStr::from_bytes_until_nul(&player_controller.m_iszPlayerName()?)
            .context("player name missing nul terminator")?
            .to_str()
            .context("invalid player name")?
            .to_string();
        
        let position = nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

        let model = game_screen_node.m_modelState()?
            .m_hModel()?
            .read_schema()?
            .address()?;

        // let model = ctx.cs2.read::<u64>(
        //     Module::Absolute,
        //     &[
        //         game_sceen_node
        //         + offsets::client::CSkeletonInstance::m_modelState /* model state */
        //         + offsets::client::CModelState::m_hModel, /* CModel* */
        //         0,
        //     ],
        // )?;

        let model = ctx.model_cache.lookup(model)?;
        let bone_states = game_screen_node.m_modelState()?
            .bone_state_data()?
            .read_entries(model.bones.len())?
            .into_iter()
            .map(|bone| bone.try_into())
            .try_collect()?;

        let team_type = if player_controller.m_bIsLocalPlayerController()? { 
            TeamType::Local 
        } else if local_team == player_team {
            TeamType::Friendly
        } else {
            TeamType::Enemy
        };

        ctx.players.push(PlayerInfo {
            team_type,
            player_name,
            player_health,
            position,

            debug_text: "".to_string(),

            bone_states,
            model: model.clone(),
        });
    }

    Ok(())
}