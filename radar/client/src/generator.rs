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
    WeaponId,
};
use cs2_schema_declaration::Ptr;
use cs2_schema_generated::cs2::client::{
    CCSPlayer_ItemServices,
    CSkeletonInstance,
    C_CSPlayerPawn,
};
use radar_shared::{
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
}

impl CS2RadarGenerator {
    pub fn new(handle: Arc<CS2Handle>) -> anyhow::Result<Self> {
        let offsets = Arc::new(CS2Offsets::resolve_offsets(&handle)?);
        let class_name_cache = ClassNameCache::new(handle.clone());
        let entity_system = EntitySystem::new(handle.clone(), offsets.clone());

        Ok(Self {
            handle,
            offsets,

            class_name_cache,
            entity_system,
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

impl RadarGenerator for CS2RadarGenerator {
    fn generate_state(&mut self, _settings: &RadarSettings) -> anyhow::Result<RadarState> {
        let mut radar_state = RadarState {
            players: Vec::with_capacity(16),
            world_name: get_current_map(&self.handle, self.offsets.network_game_client_instance)?
                .unwrap_or_else(|| "<empty>".to_string()),
        };

        self.entity_system.read_entities()?;
        let entities = self.entity_system.all_identities().to_vec();

        self.class_name_cache.update_cache(&entities)?;

        for entity_identity in entities {
            let entity_class = self
                .class_name_cache
                .lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

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

        Ok(radar_state)
    }
}
