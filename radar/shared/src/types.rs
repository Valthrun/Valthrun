use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RadarSettings {
    pub show_team_players: bool,
    pub show_enemy_players: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BombDefuser {
    /// Total time remaining for a successful bomb defuse
    pub time_remaining: f32,

    /// The defusers player name
    pub player_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum C4State {
    /// Bomb is dropped
    Dropped,

    /// Bomb is carried
    Carried,

    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        time_detonation: f32,

        /// Current bomb defuser
        defuse: Option<BombDefuser>,
    },

    /// Bomb has detonated
    Detonated,

    /// Bomb has been defused
    Defused,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RadarState {
    pub players: Vec<RadarPlayerInfo>,
    pub bomb: Option<RadarBombInfo>,
    pub world_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RadarPlayerInfo {
    pub controller_entity_id: u32,
    pub team_id: u8,

    pub player_health: i32,
    pub player_has_defuser: bool,
    pub player_name: String,
    pub weapon: u16,
    pub player_flashtime: f32,

    pub position: [f32; 3],
    pub rotation: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RadarBombInfo {
    pub position: [f32; 3],
    pub state: C4State,

    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    pub bomb_site: Option<u8>,
}
