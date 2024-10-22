use serde::{
    Deserialize,
    Serialize,
};
use typescript_type_def::TypeDef;

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "camelCase")]
pub struct BombDefuser {
    /// Total time remaining for a successful bomb defuse
    pub time_remaining: f32,

    /// Total time (in seconds) for the defusal
    pub time_total: f32,

    /// The defusers player name
    pub player_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "kebab-case", tag = "state")]
pub enum PlantedC4State {
    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        #[serde(rename = "timeDetonation")]
        time_detonation: f32,

        /// Total time (in seconds) for the detonation
        #[serde(rename = "timeTotal")]
        time_total: f32,

        /// Current bomb defuser
        defuser: Option<BombDefuser>,
    },

    /// Bomb has detonated
    Detonated {},

    /// Bomb has been defused
    Defused {},
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "camelCase")]
pub struct RadarState {
    pub world_name: String,
    pub player_pawns: Vec<RadarPlayerPawn>,

    pub planted_c4: Option<RadarPlantedC4>,
    pub c4_entities: Vec<RadarC4>,

    pub local_controller_entity_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "camelCase")]
pub struct RadarPlayerPawn {
    pub controller_entity_id: Option<u32>,
    pub pawn_entity_id: u32,
    pub team_id: u8,

    pub player_name: String,
    pub player_health: i32,
    pub player_has_defuser: bool,
    pub player_flashtime: f32,

    pub weapon: u16,

    pub position: [f32; 3],
    pub rotation: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "camelCase")]
pub struct RadarPlantedC4 {
    pub position: [f32; 3],

    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    pub bomb_site: u8,

    pub state: PlantedC4State,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypeDef)]
#[serde(rename_all = "camelCase")]
pub struct RadarC4 {
    pub entity_id: u32,
    pub position: [f32; 3],
    pub owner_entity_id: Option<u32>,
}
