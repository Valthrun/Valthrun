mod mesh;
use anyhow::Context;
use map_tracer::KV3Value;
pub use mesh::*;

mod utils;
pub use utils::*;

mod hull;
pub use hull::*;

mod shape;
pub use shape::*;

#[derive(Debug)]
pub struct CollisionAttribute {
    pub group_id: u64,
    pub group_name: String,

    pub interact_as: Vec<u64>,
    pub interact_as_name: Vec<String>,

    pub interact_exclude: Vec<u64>,
    pub interact_exclude_name: Vec<String>,

    pub interact_with: Vec<u64>,
    pub interact_with_name: Vec<String>,
}

impl CollisionAttribute {
    fn parse_array_u64(object: &KV3Value, key: &str) -> anyhow::Result<Vec<u64>> {
        object
            .get(key)
            .with_context(|| format!("missing {}", key))?
            .as_array()
            .with_context(|| format!("expected {} to be an array", key))?
            .into_iter()
            .map(|value| value.as_u64())
            .try_collect::<Vec<_>>()
            .with_context(|| format!("failed to convert {} into Vec<u64>", key))
    }

    fn parse_array_string(object: &KV3Value, key: &str) -> anyhow::Result<Vec<String>> {
        object
            .get(key)
            .with_context(|| format!("missing {}", key))?
            .as_array()
            .with_context(|| format!("expected {} to be an array", key))?
            .into_iter()
            .map(|value| value.as_str().map(|str| str.to_string()))
            .try_collect::<Vec<_>>()
            .with_context(|| format!("failed to convert {} into Vec<String>", key))
    }

    pub fn parse(value: &KV3Value) -> anyhow::Result<Self> {
        let group_id = value
            .get("m_CollisionGroup")
            .context("missing m_CollisionGroup")?
            .as_u64()
            .context("expected m_CollisionGroup to be an u64")?;

        let group_name = value
            .get("m_CollisionGroupString")
            .context("missing m_CollisionGroupString")?
            .as_str()
            .context("expected m_CollisionGroupString to be a string")?
            .to_string();

        let interact_as = Self::parse_array_u64(value, "m_InteractAs")?;
        let interact_as_name = Self::parse_array_string(value, "m_InteractAsStrings")?;

        let interact_exclude = Self::parse_array_u64(value, "m_InteractExclude")?;
        let interact_exclude_name = Self::parse_array_string(value, "m_InteractExcludeStrings")?;

        let interact_with = Self::parse_array_u64(value, "m_InteractWith")?;
        let interact_with_name = Self::parse_array_string(value, "m_InteractWithStrings")?;

        Ok(Self {
            group_id,
            group_name,

            interact_as,
            interact_as_name,

            interact_exclude,
            interact_exclude_name,

            interact_with,
            interact_with_name,
        })
    }
}
