use std::ffi::CStr;

use anyhow::Context;
use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    EntitySystem,
    Globals,
    PlayerPawnState,
    StateCurrentMap,
};
use cs2_schema_generated::cs2::{
    client::{
        CEntityIdentity,
        C_PlantedC4,
        C_C4,
    },
    globals::CSWeaponState_t,
};
use obfstr::obfstr;
use radar_shared::{
    BombDefuser,
    C4State,
    RadarBombInfo,
    RadarPlayerInfo,
    RadarSettings,
    RadarState,
};
use utils_state::StateRegistry;

pub trait RadarGenerator: Send {
    fn generate_state(&mut self, settings: &RadarSettings) -> anyhow::Result<RadarState>;
}

trait BombData {
    fn read_bomb_data(&self, generator: &CS2RadarGenerator) -> anyhow::Result<RadarBombInfo>;
}

impl BombData for C_C4 {
    fn read_bomb_data(&self, _generator: &CS2RadarGenerator) -> anyhow::Result<RadarBombInfo> {
        let position = self.m_pGameSceneNode()?.read_schema()?.m_vecAbsOrigin()?;

        if self.m_iState()? as u32 == CSWeaponState_t::WEAPON_NOT_CARRIED as u32 {
            return Ok(RadarBombInfo {
                position,
                state: C4State::Dropped,
                bomb_site: None,
            });
        }

        Ok(RadarBombInfo {
            position,
            state: C4State::Carried,
            bomb_site: None,
        })
    }
}

impl BombData for C_PlantedC4 {
    fn read_bomb_data(&self, generator: &CS2RadarGenerator) -> anyhow::Result<RadarBombInfo> {
        let globals = generator.states.resolve::<Globals>(())?;
        let entities = generator.states.resolve::<EntitySystem>(())?;

        let position = self.m_pGameSceneNode()?.read_schema()?.m_vecAbsOrigin()?;
        let bomb_site = Some(self.m_nBombSite()? as u8);

        if self.m_bBombDefused()? {
            return Ok(RadarBombInfo {
                position,
                state: C4State::Defused,
                bomb_site,
            });
        }

        let time_blow = self.m_flC4Blow()?.m_Value()?;
        if time_blow <= globals.time_2()? {
            return Ok(RadarBombInfo {
                position,
                bomb_site,
                state: C4State::Detonated,
            });
        }

        let is_defusing = self.m_bBeingDefused()?;
        let defusing = if is_defusing {
            let time_defuse = self.m_flDefuseCountDown()?.m_Value()?;

            let handle_defuser = self.m_hBombDefuser()?;
            let defuser = entities
                .get_by_handle(&handle_defuser)?
                .with_context(|| obfstr!("missing bomb defuser player pawn").to_string())?
                .entity()?
                .reference_schema()?;

            let defuser_controller = defuser.m_hController()?;
            let defuser_controller = entities
                .get_by_handle(&defuser_controller)?
                .with_context(|| obfstr!("missing bomb defuser controller").to_string())?
                .entity()?
                .reference_schema()?;

            let defuser_name = CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
                .ok()
                .map(CStr::to_string_lossy)
                .unwrap_or("Name Error".into())
                .to_string();

            Some(BombDefuser {
                time_remaining: time_defuse - globals.time_2()?,
                player_name: defuser_name,
            })
        } else {
            None
        };

        Ok(RadarBombInfo {
            position,
            state: C4State::Active {
                time_detonation: time_blow - globals.time_2()?,
                defuse: defusing,
            },
            bomb_site,
        })
    }
}

pub struct CS2RadarGenerator {
    states: StateRegistry,
}

impl CS2RadarGenerator {
    pub fn new(states: StateRegistry) -> anyhow::Result<Self> {
        Ok(Self { states })
    }

    fn generate_player_info(
        &self,
        player_pawn: &CEntityIdentity,
    ) -> anyhow::Result<Option<RadarPlayerInfo>> {
        let player_info = self
            .states
            .resolve::<PlayerPawnState>(player_pawn.handle::<()>()?.get_entity_index())?;

        match &*player_info {
            PlayerPawnState::Alive(info) => Ok(Some(RadarPlayerInfo {
                controller_entity_id: info.controller_entity_id,

                player_name: info.player_name.clone(),
                player_flashtime: info.player_flashtime,
                player_has_defuser: info.player_has_defuser,
                player_health: info.player_health,

                position: [info.position.x, info.position.y, info.position.z],
                rotation: info.rotation,

                team_id: info.team_id,
                weapon: info.weapon.id(),
            })),
            _ => Ok(None),
        }
    }
}

impl RadarGenerator for CS2RadarGenerator {
    fn generate_state(&mut self, _settings: &RadarSettings) -> anyhow::Result<RadarState> {
        self.states.invalidate_states();

        let current_map = self.states.resolve::<StateCurrentMap>(())?;
        let mut radar_state = RadarState {
            players: Vec::with_capacity(16),
            world_name: current_map
                .current_map
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or("<empty>")
                .to_string(),
            bomb: None,
        };

        let entities = self.states.resolve::<EntitySystem>(())?;
        let class_name_cache = self.states.resolve::<ClassNameCache>(())?;

        for entity_identity in entities.all_identities() {
            let entity_class =
                match class_name_cache.lookup(&entity_identity.entity_class_info()?)? {
                    Some(entity_class) => entity_class,
                    None => {
                        log::warn!(
                            "Failed to get entity class info {:X}",
                            entity_identity.memory.address,
                        );
                        continue;
                    }
                };

            match entity_class.as_str() {
                "C_CSPlayerPawn" => match self.generate_player_info(entity_identity) {
                    Ok(Some(info)) => radar_state.players.push(info),
                    Ok(None) => {}
                    Err(error) => {
                        log::warn!(
                            "Failed to generate player pawn ESP info for {}: {:#}",
                            entity_identity.handle::<()>()?.get_entity_index(),
                            error
                        );
                    }
                },
                "C_C4" | "C_PlantedC4" => {
                    let bomb_ptr: Box<dyn BombData> = match entity_class.as_str() {
                        "C_C4" => Box::new(entity_identity.entity_ptr::<C_C4>()?.read_schema()?),
                        "C_PlantedC4" => {
                            Box::new(entity_identity.entity_ptr::<C_PlantedC4>()?.read_schema()?)
                        }
                        _ => unreachable!(),
                    };

                    if let Ok(bomb_data) = bomb_ptr.read_bomb_data(self) {
                        radar_state.bomb = Some(bomb_data);
                    }
                }
                _ => {}
            }
        }

        Ok(radar_state)
    }
}
