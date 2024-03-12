use cs2::{
    CEntityIdentityEx,
    ClassNameCache,
    CurrentMapState,
    EntitySystem,
    PlayerPawnState,
};
use cs2_schema_generated::cs2::client::CEntityIdentity;
use radar_shared::{
    RadarPlayerInfo,
    RadarSettings,
    RadarState,
};
use utils_state::StateRegistry;

pub trait RadarGenerator {
    fn generate_state(&mut self, settings: &RadarSettings) -> anyhow::Result<RadarState>;
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

        let current_map = self.states.resolve::<CurrentMapState>(())?;
        let mut radar_state = RadarState {
            players: Vec::with_capacity(16),
            world_name: current_map
                .current_map
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or("<empty>")
                .to_string(),
        };

        let entities = self.states.resolve::<EntitySystem>(())?;
        let class_name_cache = self.states.resolve::<ClassNameCache>(())?;

        for entity_identity in entities.all_identities() {
            let entity_class = class_name_cache.lookup(&entity_identity.entity_class_info()?)?;
            if !entity_class
                .map(|name| *name == "C_CSPlayerPawn")
                .unwrap_or(false)
            {
                /* entity is not a player pawn */
                continue;
            }

            match self.generate_player_info(entity_identity) {
                Ok(Some(info)) => radar_state.players.push(info),
                Ok(None) => {}
                Err(error) => {
                    log::warn!(
                        "Failed to generate player pawn ESP info for entity {}: {:#}",
                        entity_identity.handle::<()>()?.get_entity_index(),
                        error
                    );
                }
            }
        }

        Ok(radar_state)
    }
}
