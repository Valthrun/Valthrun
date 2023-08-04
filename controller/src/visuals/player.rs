use std::sync::Arc;

use anyhow::Context;
use cs2::{EntityHandle, Module, offsets_manual, CS2Model};
use cs2_schema::offsets;
use obfstr::obfstr;

use crate::Application;

pub struct PlayerInfo {
    pub local: bool,
    pub player_health: i32,
    pub player_name: String,
    pub position: nalgebra::Vector3<f32>,
 
    pub debug_text: String,

    pub model: Arc<CS2Model>,
    pub bone_states: Vec<BoneStateData>,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BoneStateData {
    pub position: nalgebra::Vector3<f32>,
    pub scale: f32,
    pub rotation: nalgebra::Vector4<f32>,
}
const _: [u8; 0x20] = [0; std::mem::size_of::<BoneStateData>()];

pub fn read_player_info(ctx: &mut Application) -> anyhow::Result<()> {
    ctx.players.clear();
    ctx.players.reserve(16);

    let local_player_controller = ctx
        .cs2_entities
        .get_local_player_controller()?
        .with_context(|| obfstr!("missing local player controller").to_string())?;

    for player_controller in ctx.cs2_entities.get_player_controllers()? {
        let player_pawn_handle = ctx
            .cs2
            .read::<EntityHandle>(
                Module::Absolute,
                &[player_controller + offsets::client::CCSPlayerController::m_hPlayerPawn],
            )
            .with_context(|| obfstr!("failed to read player pawn handle").to_string())?;

        if !player_pawn_handle.is_valid() {
            continue;
        }

        let player_health = ctx
            .cs2
            .read::<i32>(
                Module::Absolute,
                &[player_controller + offsets::client::CCSPlayerController::m_iPawnHealth],
            )
            .with_context(|| obfstr!("failed to read player controller pawn health").to_string())?;
        if player_health <= 0 {
            continue;
        }

        let player_pawn = ctx
            .cs2_entities
            .get_by_handle(&player_pawn_handle)?
            .with_context(|| obfstr!("missing player pawn for player controller").to_string())?;

        /* Will be an instance of CSkeletonInstance */
        let game_sceen_node = ctx.cs2.read::<u64>(
            Module::Absolute,
            &[player_pawn + offsets::client::C_BaseEntity::m_pGameSceneNode],
        )?;

        let player_dormant = ctx.cs2.read::<bool>(
            Module::Absolute,
            &[game_sceen_node + offsets::client::CGameSceneNode::m_bDormant],
        )?;
        if player_dormant {
            continue;
        }

        let player_name = ctx.cs2.read_string(
            Module::Absolute,
            &[player_controller + offsets::client::CBasePlayerController::m_iszPlayerName],
            Some(128),
        )?;

        let position = ctx.cs2.read::<nalgebra::Vector3<f32>>(
            Module::Absolute,
            &[game_sceen_node + offsets::client::CGameSceneNode::m_vecAbsOrigin],
        )?;

        let model = ctx.cs2.read::<u64>(
            Module::Absolute,
            &[
                game_sceen_node
                + offsets::client::CSkeletonInstance::m_modelState /* model state */
                + offsets::client::CModelState::m_hModel, /* CModel* */
                0,
            ],
        )?;

        let model = ctx.model_cache.lookup(model)?;
        let bone_states = ctx.cs2.read_vec::<BoneStateData>(
            Module::Absolute,
            &[
                game_sceen_node
                + offsets::client::CSkeletonInstance::m_modelState /* model state */
                + offsets_manual::client::CModelState::BONE_STATE_DATA,
                0, /* read the whole array */
            ],
            model.bones.len(),
        )?;

        ctx.players.push(PlayerInfo {
            local: player_controller == local_player_controller,
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