use radar_shared::RadarState;

use super::RadarGenerator;

pub struct DummyRadarGenerator;

impl RadarGenerator for DummyRadarGenerator {
    fn generate_state(&mut self) -> anyhow::Result<RadarState> {
        let state = RadarState {
            world_name: "de_dust2".to_string(),
            c4_entities: Vec::new(),
            planted_c4: None,
            player_pawns: Vec::new(),
            local_controller_entity_id: None,
        };
        Ok(state)
    }
}
