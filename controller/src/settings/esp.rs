use cs2::{
    WeaponId,
    WEAPON_FLAG_TYPE_GRENADE,
    WEAPON_FLAG_TYPE_MACHINE_GUN,
    WEAPON_FLAG_TYPE_PISTOL,
    WEAPON_FLAG_TYPE_RIFLE,
    WEAPON_FLAG_TYPE_SHOTGUN,
    WEAPON_FLAG_TYPE_SMG,
    WEAPON_FLAG_TYPE_SNIPER_RIFLE,
};
use obfstr::obfstr;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Color(u32);
impl Color {
    pub fn as_u8(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub fn as_f32(&self) -> [f32; 4] {
        self.as_u8()
            .map(|channel| (channel as f32) / (u8::MAX as f32))
    }

    pub const fn from_u8(value: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(value))
    }

    pub const fn from_f32(value: [f32; 4]) -> Self {
        Self::from_u8([
            (value[0] * 255.0) as u8,
            (value[1] * 255.0) as u8,
            (value[2] * 255.0) as u8,
            (value[3] * 255.0) as u8,
        ])
    }

    pub fn set_alpha_u8(&mut self, alpha: u8) {
        let mut value = self.as_u8();
        value[3] = alpha;
        *self = Self::from_u8(value);
    }

    pub fn set_alpha_f32(&mut self, alpha: f32) {
        let mut value = self.as_u8();
        value[3] = (alpha * 255.0) as u8;
        *self = Self::from_u8(value);
    }
}

impl From<[u8; 4]> for Color {
    fn from(value: [u8; 4]) -> Self {
        Self::from_u8(value)
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Self::from_f32(value)
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
#[serde(tag = "type", content = "options")]
pub enum EspColor {
    HealthBasedRainbow,
    HealthBased { max: Color, min: Color },
    Static { value: Color },
    DistanceBased,
}

impl Default for EspColor {
    fn default() -> Self {
        Self::Static {
            value: Color::from_f32([1.0, 1.0, 1.0, 1.0]),
        }
    }
}

impl EspColor {
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Static {
            value: Color::from_f32([r, g, b, a]),
        }
    }

    /// Calculate the target color.
    /// Health should be in [0.0;1.0]
    pub fn calculate_color(&self, health: f32, distance: f32) -> [f32; 4] {
        match self {
            Self::Static { value } => value.as_f32(),
            Self::HealthBased { max, min } => {
                let min_rgb = min.as_f32();
                let max_rgb = max.as_f32();

                [
                    min_rgb[0] + (max_rgb[0] - min_rgb[0]) * health,
                    min_rgb[1] + (max_rgb[1] - min_rgb[1]) * health,
                    min_rgb[2] + (max_rgb[2] - min_rgb[2]) * health,
                    min_rgb[3] + (max_rgb[3] - min_rgb[3]) * health,
                ]
            }
            Self::HealthBasedRainbow => {
                let sin_value = |offset: f32| {
                    (2.0 * std::f32::consts::PI * health * 0.75 + offset).sin() * 0.5 + 1.0
                };
                let r: f32 = sin_value(0.0);
                let g: f32 = sin_value(2.0 * std::f32::consts::PI / 3.0);
                let b: f32 = sin_value(4.0 * std::f32::consts::PI / 3.0);
                [r, g, b, 1.0]
            }
            Self::DistanceBased => {
                let max_distance = 80.0;
                let min_distance = 0.0;

                let color_near = [1.0, 0.0, 0.0, 0.75];
                let color_far = [0.0, 1.0, 0.0, 0.75];

                let t = (distance - min_distance) / (max_distance - min_distance);
                let t = t.clamp(0.0, 1.0);

                [
                    color_near[0] + t * (color_far[0] - color_near[0]),
                    color_near[1] + t * (color_far[1] - color_near[1]),
                    color_near[2] + t * (color_far[2] - color_near[2]),
                    0.75,
                ]
            }
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EspColorType {
    Static,
    HealthBased,
    HealthBasedRainbow,
    DistanceBased,
}

impl EspColorType {
    pub fn from_esp_color(color: &EspColor) -> Self {
        match color {
            EspColor::Static { .. } => Self::Static,
            EspColor::HealthBased { .. } => Self::HealthBased,
            EspColor::HealthBasedRainbow => Self::HealthBasedRainbow,
            EspColor::DistanceBased => Self::DistanceBased,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EspHealthBar {
    None,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EspBoxType {
    /// Disabled player box
    None,

    /// 2D player box
    Box2D,

    /// 3D player box
    Box3D,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EspTracePosition {
    None,
    TopLeft,
    TopCenter,
    TopRight,
    Center,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EspHeadDot {
    None,
    Filled,
    NotFilled,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct EspPlayerSettings {
    pub box_type: EspBoxType,
    pub box_color: EspColor,
    pub box_width: f32,

    pub skeleton: bool,
    pub skeleton_color: EspColor,
    pub skeleton_width: f32,

    pub health_bar: EspHealthBar,
    pub health_bar_width: f32,

    pub tracer_lines: EspTracePosition,
    pub tracer_lines_color: EspColor,
    pub tracer_lines_width: f32,

    pub info_name: bool,
    pub info_name_color: EspColor,

    pub info_distance: bool,
    pub info_distance_color: EspColor,

    pub near_players: bool,
    pub near_players_distance: f32,

    pub info_weapon: bool,
    pub info_weapon_color: EspColor,

    pub info_hp_text: bool,
    pub info_hp_text_color: EspColor,

    pub info_flag_kit: bool,
    pub info_flag_flashed: bool,
    pub info_flags_color: EspColor,

    pub head_dot: EspHeadDot,
    pub head_dot_color: EspColor,
    pub head_dot_thickness: f32,
    pub head_dot_base_radius: f32,
    pub head_dot_z: f32,
}

const ESP_COLOR_FRIENDLY: EspColor = EspColor::from_rgba(0.0, 1.0, 0.0, 0.75);
const ESP_COLOR_ENEMY: EspColor = EspColor::from_rgba(1.0, 0.0, 0.0, 0.75);
impl EspPlayerSettings {
    pub fn new(target: &EspSelector) -> Self {
        let color = match target {
            EspSelector::PlayerTeam { enemy } => {
                if *enemy {
                    ESP_COLOR_ENEMY
                } else {
                    ESP_COLOR_FRIENDLY
                }
            }
            EspSelector::PlayerTeamVisibility { enemy, .. } => {
                if *enemy {
                    ESP_COLOR_ENEMY
                } else {
                    ESP_COLOR_FRIENDLY
                }
            }
            _ => EspColor::from_rgba(1.0, 1.0, 1.0, 0.75),
        };

        Self {
            box_type: EspBoxType::None,
            box_color: color.clone(),
            box_width: 3.0,

            skeleton: true,
            skeleton_color: color.clone(),
            skeleton_width: 3.0,

            health_bar: EspHealthBar::None,
            health_bar_width: 10.0,

            tracer_lines: EspTracePosition::None,
            tracer_lines_color: color.clone(),
            tracer_lines_width: 1.0,

            info_distance: false,
            info_distance_color: color.clone(),

            near_players: false,
            near_players_distance: 20.0,

            info_hp_text: false,
            info_hp_text_color: color.clone(),

            info_name: false,
            info_name_color: color.clone(),

            info_weapon: false,
            info_weapon_color: color.clone(),

            info_flag_kit: false,
            info_flag_flashed: false,
            info_flags_color: color.clone(),

            head_dot: EspHeadDot::None,
            head_dot_color: color.clone(),
            head_dot_thickness: 2.0,
            head_dot_base_radius: 3.0,
            head_dot_z: 1.0,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct EspChickenSettings {
    pub box_type: EspBoxType,
    pub box_color: EspColor,

    pub skeleton: bool,
    pub skeleton_color: EspColor,

    pub info_owner: bool,
    pub info_owner_color: EspColor,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct EspWeaponSettings {
    pub draw_box: bool,
    pub draw_box_color: EspColor,

    pub info_name: bool,
    pub info_name_color: EspColor,
}

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
#[serde(tag = "type")]
pub enum EspConfig {
    Player(EspPlayerSettings),
    Chicken(EspChickenSettings),
    Weapon(EspWeaponSettings),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum EspWeaponType {
    Pistol,
    Shotgun,
    SMG,
    Rifle,
    SniperRifle,
    MachineGun,
    Grenade,
}

impl EspWeaponType {
    pub fn display_name(&self) -> String {
        match self {
            Self::Pistol => "Pistol".to_string(),
            Self::Shotgun => "Shotgun".to_string(),
            Self::SMG => "SMG".to_string(),
            Self::Rifle => "Rifle".to_string(),
            Self::SniperRifle => "Sniper Rifle".to_string(),
            Self::MachineGun => "Machine Gun".to_string(),
            Self::Grenade => "Grenade".to_string(),
        }
    }

    pub fn config_key(&self) -> &'static str {
        match self {
            Self::Pistol => "pistol",
            Self::Shotgun => "shotgun",
            Self::SMG => "smg",
            Self::Rifle => "rifle",
            Self::SniperRifle => "sniper-rifle",
            Self::MachineGun => "machine-gun",
            Self::Grenade => "grenade",
        }
    }

    pub fn weapons(&self) -> Vec<WeaponId> {
        let flag = match self {
            Self::Pistol => WEAPON_FLAG_TYPE_PISTOL,
            Self::Shotgun => WEAPON_FLAG_TYPE_SHOTGUN,
            Self::SMG => WEAPON_FLAG_TYPE_SMG,
            Self::Rifle => WEAPON_FLAG_TYPE_RIFLE,
            Self::SniperRifle => WEAPON_FLAG_TYPE_SNIPER_RIFLE,
            Self::MachineGun => WEAPON_FLAG_TYPE_MACHINE_GUN,
            Self::Grenade => WEAPON_FLAG_TYPE_GRENADE,
        };

        WeaponId::all_weapons()
            .into_iter()
            .filter(|weapon| (weapon.flags() & flag) > 0)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum EspSelector {
    None,

    Player,
    PlayerTeam {
        enemy: bool,
    },
    PlayerTeamVisibility {
        enemy: bool,
        visible: bool,
    },

    Chicken,

    Weapon,
    WeaponGroup {
        group: EspWeaponType,
    },
    WeaponSingle {
        group: EspWeaponType,
        target: WeaponId,
    },
}

impl EspSelector {
    pub fn config_key(&self) -> String {
        match self {
            EspSelector::None => "invalid".to_string(),
            EspSelector::Player => "player".to_string(),
            EspSelector::PlayerTeam { enemy } => {
                format!("player.{}", if *enemy { "enemy" } else { "friendly" },)
            }
            EspSelector::PlayerTeamVisibility { enemy, visible } => format!(
                "player.{}.{}",
                if *enemy { "enemy" } else { "friendly" },
                if *visible { "visible" } else { "occluded" }
            ),
            EspSelector::Chicken => "chicken".to_string(),

            EspSelector::Weapon => format!("weapon"),
            EspSelector::WeaponGroup { group } => format!("weapon.{}", group.config_key()),
            EspSelector::WeaponSingle { group, target } => {
                format!("weapon.{}.{}", group.config_key(), target.name())
            }
        }
    }

    pub fn config_display(&self) -> String {
        match self {
            EspSelector::None => "None".to_string(),

            EspSelector::Player => "Player".to_string(),
            EspSelector::PlayerTeam { enemy } => {
                if *enemy {
                    "Enemy".to_string()
                } else {
                    "Friendly".to_string()
                }
            }
            EspSelector::PlayerTeamVisibility { visible, .. } => {
                if *visible {
                    "Visible".to_string()
                } else {
                    "Occluded".to_string()
                }
            }

            EspSelector::Chicken => "Chicken".to_string(),

            EspSelector::Weapon => "Weapons".to_string(),
            EspSelector::WeaponGroup { group } => group.display_name(),
            EspSelector::WeaponSingle { target, .. } => target.display_name().to_string(),
        }
    }

    pub fn config_title(&self) -> String {
        match self {
            EspSelector::None => obfstr!("ESP Configuration").to_string(),

            EspSelector::Player => obfstr!("Enabled ESP for all players").to_string(),
            EspSelector::PlayerTeam { enemy } => format!(
                "{} {} players",
                obfstr!("Enabled ESP for"),
                if *enemy { "enemy" } else { "friendly" }
            ),
            EspSelector::PlayerTeamVisibility { enemy, visible } => format!(
                "{} {} {} players",
                obfstr!("Enabled ESP for"),
                if *visible { "visible" } else { "occluded" },
                if *enemy { "enemy" } else { "friendly" }
            ),

            EspSelector::Chicken => obfstr!("Enabled ESP for chickens").to_string(),

            EspSelector::Weapon => obfstr!("Enabled ESP for all weapons").to_string(),
            EspSelector::WeaponGroup { group } => {
                format!(
                    "{} {}",
                    obfstr!("Enabled ESP for"),
                    group.display_name().to_lowercase()
                )
            }
            EspSelector::WeaponSingle { target, .. } => {
                format!(
                    "{} {}",
                    obfstr!("Enabled ESP for weapon"),
                    target.display_name()
                )
            }
        }
    }

    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::None => None,

            Self::Player => None,
            Self::PlayerTeam { .. } => Some(Self::Player),
            Self::PlayerTeamVisibility { enemy, .. } => Some(Self::PlayerTeam { enemy: *enemy }),

            Self::Chicken => None,

            Self::Weapon => None,
            Self::WeaponGroup { .. } => Some(Self::Weapon),
            Self::WeaponSingle { group, .. } => Some(Self::WeaponGroup { group: *group }),
        }
    }

    pub fn children(&self) -> Vec<Self> {
        match self {
            EspSelector::None => vec![],
            EspSelector::Player => vec![
                EspSelector::PlayerTeam { enemy: false },
                EspSelector::PlayerTeam { enemy: true },
            ],
            /* Currently disable visibility as we do not have a proper vis check */
            EspSelector::PlayerTeam { .. } => vec![],
            // EspSelector::PlayerTeam { enemy } => vec![
            //     EspSelector::PlayerTeamVisibility {
            //         enemy: *enemy,
            //         visible: true,
            //     },
            //     EspSelector::PlayerTeamVisibility {
            //         enemy: *enemy,
            //         visible: false,
            //     },
            // ],
            EspSelector::PlayerTeamVisibility { .. } => vec![],
            EspSelector::Chicken => vec![],

            EspSelector::Weapon => vec![
                EspSelector::WeaponGroup {
                    group: EspWeaponType::Pistol,
                },
                EspSelector::WeaponGroup {
                    group: EspWeaponType::SMG,
                },
                EspSelector::WeaponGroup {
                    group: EspWeaponType::Shotgun,
                },
                EspSelector::WeaponGroup {
                    group: EspWeaponType::Rifle,
                },
                EspSelector::WeaponGroup {
                    group: EspWeaponType::SniperRifle,
                },
                EspSelector::WeaponGroup {
                    group: EspWeaponType::Grenade,
                },
            ],
            EspSelector::WeaponGroup { group } => group
                .weapons()
                .into_iter()
                .map(|weapon| EspSelector::WeaponSingle {
                    group: *group,
                    target: weapon,
                })
                .collect(),
            EspSelector::WeaponSingle { .. } => vec![],
        }
    }
}
