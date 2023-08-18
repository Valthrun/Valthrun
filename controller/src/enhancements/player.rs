use std::{sync::Arc, ffi::CStr};

use anyhow::Context;
use cs2::{CS2Model, BoneFlags};
use cs2_schema_declaration::{define_schema, Ptr};
use cs2_schema_generated::cs2::client::{CSkeletonInstance, CModelState};
use obfstr::obfstr;

use crate::{settings::AppSettings, view::ViewController};

use super::Enhancement;

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

pub struct PlayerESP {
    players: Vec<PlayerInfo>
}

impl PlayerESP {
    pub fn new() -> Self {
        PlayerESP { players: Default::default() }
    }
}

impl Enhancement for PlayerESP {
    fn update(&mut self, ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        self.players.clear();
        if !ctx.settings.esp_boxes && !ctx.settings.esp_skeleton {
            return Ok(());
        }

        self.players.reserve(16);
    
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
                .read_schema()?;
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
    
            self.players.push(PlayerInfo {
                team_type,
                player_name,
                player_health,
                position,
    
                bone_states,
                model: model.clone(),
            });
        }
    
        Ok(())
    }

    fn render(&self, settings: &AppSettings, ui: &imgui::Ui, view: &ViewController) {
        let draw = ui.get_window_draw_list();
        for entry in self.players.iter() {
            if matches!(&entry.team_type, TeamType::Local) {
                continue;
            }

            let esp_color = if entry.team_type == TeamType::Enemy {
                &settings.esp_color_enemy
            } else {
                &settings.esp_color_team
            };
            if settings.esp_skeleton && entry.team_type != TeamType::Local {
                let bones = entry.model.bones.iter()
                    .zip(entry.bone_states.iter());

                for (bone, state) in bones {
                    if (bone.flags & BoneFlags::FlagHitbox as u32) == 0 {
                        continue;
                    }

                    let parent_index = if let Some(parent) = bone.parent {
                        parent
                    } else {
                        continue;
                    };

                    let parent_position = match view.world_to_screen(&entry.bone_states[parent_index].position, true)
                    {
                        Some(position) => position,
                        None => continue,
                    };
                    let bone_position =
                        match view.world_to_screen(&state.position, true) {
                            Some(position) => position,
                            None => continue,
                        };

                    draw.add_line(
                        parent_position,
                        bone_position,
                        *esp_color,
                    )
                        .thickness(settings.esp_skeleton_thickness)
                        .build();
                }
            }

            if settings.esp_boxes && entry.team_type != TeamType::Local {
                view.draw_box_3d(
                    &draw,
                    &(entry.model.vhull_min + entry.position),
                    &(entry.model.vhull_max + entry.position),
                    (*esp_color).into(),
                    settings.esp_boxes_thickness
                );
            }
        }
    }
}