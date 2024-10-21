use std::ffi::CStr;

use anyhow::Context;
use cs2_schema_generated::cs2::client::{
    C_PlantedC4,
    C_C4,
};
use obfstr::obfstr;
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};

use crate::{
    CEntityIdentityEx,
    ClassNameCache,
    EntitySystem,
    Globals,
};

#[derive(Debug)]
pub struct BombDefuser {
    /// Totoal time remaining for a successfull bomb defuse
    pub time_remaining: f32,

    /// The defusers player name
    pub player_name: String,
}

#[derive(Debug)]
pub enum PlantedC4State {
    /// Bomb is currently actively ticking
    Active {
        /// Time remaining (in seconds) until detonation
        time_detonation: f32,
        /// Current bomb position
        bomb_position: nalgebra::Vector3<f32>,
    },

    /// Bomb has detonated
    Detonated,

    /// Bomb has been defused
    Defused,

    /// Bomb has not been planted
    NotPlanted {
        /// Entity ID of C4 Owner
        c4_owner_entity_index: u32,
        /// Current bomb position
        bomb_position: nalgebra::Vector3<f32>,
    },
}

/// Information about the currently active planted C4
pub struct PlantedC4 {
    /// Planted bomb site
    /// 0 = A
    /// 1 = B
    pub bomb_site: u8,

    /// Current state of the planted C4
    pub state: PlantedC4State,

    /// Current bomb defuser
    pub defuser: Option<BombDefuser>,
}

impl State for PlantedC4 {
    type Parameter = ();

    fn create(states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        let globals = states.resolve::<Globals>(())?;
        let entities = states.resolve::<EntitySystem>(())?;
        let class_name_cache = states.resolve::<ClassNameCache>(())?;

        for entity_identity in entities.all_identities().iter() {
            let class_name = class_name_cache
                .lookup(&entity_identity.entity_class_info()?)
                .context("class name")?;

            if class_name.map(|name| name == "C_C4").unwrap_or(false) {
                /* Bomb not planted. */
                let bomb_not_activated = entity_identity
                    .entity_ptr::<C_C4>()?
                    .read_schema()
                    .context("bomb schame")?;

                let c4_owner = bomb_not_activated.m_hOwnerEntity()?.get_entity_index();
                let bomb_screen_node = bomb_not_activated.m_pGameSceneNode()?.read_schema()?;
                let bomb_pos_np = nalgebra::Vector3::<f32>::from_column_slice(
                    &bomb_screen_node.m_vecAbsOrigin()?,
                );
                return Ok(Self {
                    bomb_site: 0,
                    defuser: None,
                    state: PlantedC4State::NotPlanted {
                        c4_owner_entity_index: c4_owner,
                        bomb_position: bomb_pos_np,
                    },
                });
            }

            if !class_name
                .map(|name| name == "C_PlantedC4")
                .unwrap_or(false)
            {
                /* Entity isn't the planted bomb. */

                continue;
            }
            /* Bomb is Planted */
            let bomb = entity_identity
                .entity_ptr::<C_PlantedC4>()?
                .read_schema()
                .context("bomb schame")?;

            if !bomb.m_bC4Activated()? {
                /* This bomb hasn't been activated (yet) */
                continue;
            }
            let game_screen_node = bomb.m_pGameSceneNode()?.read_schema()?;

            let bomb_pos =
                nalgebra::Vector3::<f32>::from_column_slice(&game_screen_node.m_vecAbsOrigin()?);

            let bomb_site = bomb.m_nBombSite()? as u8;
            if bomb.m_bBombDefused()? {
                return Ok(Self {
                    bomb_site,
                    defuser: None,
                    state: PlantedC4State::Defused,
                });
            }

            let time_blow = bomb.m_flC4Blow()?.m_Value()?;

            if time_blow <= globals.time_2()? {
                return Ok(Self {
                    bomb_site,
                    defuser: None,
                    state: PlantedC4State::Detonated,
                });
            }

            let is_defusing = bomb.m_bBeingDefused()?;
            let defusing = if is_defusing {
                let time_defuse = bomb.m_flDefuseCountDown()?.m_Value()?;

                let handle_defuser = bomb.m_hBombDefuser()?;
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

                let defuser_name =
                    CStr::from_bytes_until_nul(&defuser_controller.m_iszPlayerName()?)
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

            return Ok(Self {
                bomb_site,
                defuser: defusing,
                state: PlantedC4State::Active {
                    time_detonation: time_blow - globals.time_2()?,
                    bomb_position: bomb_pos,
                },
            });
        }

        return Ok(Self {
            bomb_site: 0,
            defuser: None,
            state: PlantedC4State::NotPlanted {
                c4_owner_entity_index: 0,
                bomb_position: nalgebra::Vector3::zeros(),
            },
        });
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Volatile
    }
}
