use std::{
    ffi::CStr,
    sync::Arc,
};

use anyhow::Context;
use cs2::{
    get_current_map,
    CEntityIdentityEx,
    CS2Handle,
    CS2Offsets,
    ClassNameCache,
    EntitySystem,
    Globals,
    WeaponId,
};
use cs2_schema_declaration::Ptr;
use cs2_schema_generated::cs2::{
    client::{
        CCSPlayer_ItemServices,
        CSkeletonInstance,
        C_CSPlayerPawn,
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

use crate::RadarGenerator;

pub struct CS2RadarGenerator {
    handle: Arc<CS2Handle>,
    offsets: Arc<CS2Offsets>,
    class_name_cache: ClassNameCache,
    entity_system: EntitySystem,
    globals: Globals,
}

impl CS2RadarGenerator {
    pub fn new(handle: Arc<CS2Handle>) -> anyhow::Result<Self> {
        let offsets = Arc::new(CS2Offsets::resolve_offsets(&handle)?);
        let class_name_cache = ClassNameCache::new(handle.clone());
        let entity_system = EntitySystem::new(handle.clone(), offsets.clone());
        let globals = handle
            .reference_schema::<Globals>(&[offsets.globals, 0])?
            .cached()
            .with_context(|| obfstr!("failed to read globals").to_string())?;

        Ok(Self {
            handle,
            offsets,

            class_name_cache,
            entity_system,
            globals,
        })
    }

    fn generate_player_info(
        &mut self,
        player_pawn: &Ptr<C_CSPlayerPawn>,
    ) -> anyhow::Result<Option<RadarPlayerInfo>> {
        let player_pawn = player_pawn
            .read_schema()
            .with_context(|| "failed to read player pawn data".to_string())?;

        let player_health = player_pawn.m_iHealth()?;

        /* Will be an instance of CSkeletonInstance */
        let game_screen_node = player_pawn
            .m_pGameSceneNode()?
            .cast::<CSkeletonInstance>()
            .read_schema()?;

        // let dormant = game_screen_node.m_bDormant()?;
        let controller_handle = player_pawn.m_hController()?;
        let current_controller = self.entity_system.get_by_handle(&controller_handle)?;

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
             * This is the case for pawns which are not controller by a player controller.
             * An example would be the main screen player pawns.
             *
             * Note: We're assuming, that uncontrolled player pawns are negligible while being in a match as they do not occur.
             * Bots (and controller bots) always have a player pawn controller.
             */
            // log::warn!(
            //     "Handle at address {:p} has no valid controller!",
            //     &controller_handle
            // );
            return Ok(None);
        };

        let player_has_defuser = player_pawn
            .m_pItemServices()?
            .cast::<CCSPlayer_ItemServices>()
            .reference_schema()?
            .m_bHasDefuser()?;

        let position = game_screen_node.m_vecAbsOrigin()?;
        let rotation = player_pawn.m_angEyeAngles()?[1];

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

        Ok(Some(RadarPlayerInfo {
            controller_entity_id: controller_handle.get_entity_index(),

            player_name,
            player_health,
            player_flashtime,
            player_has_defuser,

            position,
            rotation,

            team_id: player_team,
            weapon: weapon_type,
        }))
    }
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
        if time_blow <= generator.globals.time_2()? {
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
            let defuser = generator
                .entity_system
                .get_by_handle(&handle_defuser)?
                .with_context(|| obfstr!("missing bomb defuser player pawn").to_string())?
                .entity()?
                .reference_schema()?;

            let defuser_controller = defuser.m_hController()?;
            let defuser_controller = generator
                .entity_system
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
                time_remaining: time_defuse - generator.globals.time_2()?,
                player_name: defuser_name,
            })
        } else {
            None
        };

        Ok(RadarBombInfo {
            position,
            state: C4State::Active {
                time_detonation: time_blow - generator.globals.time_2()?,
                defuse: defusing,
            },
            bomb_site,
        })
    }
}

impl RadarGenerator for CS2RadarGenerator {
    fn generate_state(&mut self, _settings: &RadarSettings) -> anyhow::Result<RadarState> {
        let mut radar_state = RadarState {
            players: Vec::with_capacity(16),
            world_name: get_current_map(&self.handle, self.offsets.network_game_client_instance)?
                .unwrap_or_else(|| "<empty>".to_string()),
            bomb: None,
        };
        self.entity_system.read_entities()?;
        let entities = self.entity_system.all_identities().to_vec();

        self.class_name_cache.update_cache(&entities)?;

        self.globals = self
            .handle
            .reference_schema::<Globals>(&[self.offsets.globals, 0])?
            .cached()
            .with_context(|| obfstr!("failed to read globals").to_string())?;

        for entity_identity in entities {
            let entity_class = self
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;

            match entity_class {
                Some(entity_class) => match entity_class.as_str() {
                    "C_CSPlayerPawn" => {
                        let player_pawn = entity_identity.entity_ptr::<C_CSPlayerPawn>()?;
                        match self.generate_player_info(&player_pawn) {
                            Ok(Some(info)) => radar_state.players.push(info),
                            Ok(None) => {}
                            Err(error) => {
                                log::warn!(
                                    "Failed to generate player pawn ESP info for {:X}: {:#}",
                                    player_pawn.address()?,
                                    error
                                );
                            }
                        }
                    }
                    "C_C4" | "C_PlantedC4" => {
                        let bomb_ptr: Box<dyn BombData> = match entity_class.as_str() {
                            "C_C4" => {
                                Box::new(entity_identity.entity_ptr::<C_C4>()?.read_schema()?)
                            }
                            "C_PlantedC4" => Box::new(
                                entity_identity.entity_ptr::<C_PlantedC4>()?.read_schema()?,
                            ),
                            _ => unreachable!(),
                        };

                        if let Ok(bomb_data) = bomb_ptr.read_bomb_data(self) {
                            radar_state.bomb = Some(bomb_data);
                        }
                    }
                    _ => {}
                },
                None => {
                    log::warn!(
                        "Failed to get entity class info {:X}",
                        entity_identity.memory.address,
                    );
                }
            }
        }

        Ok(radar_state)
    }
}
