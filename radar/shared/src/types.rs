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
pub struct RadarState {
    pub players: Vec<RadarPlayerInfo>,
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
