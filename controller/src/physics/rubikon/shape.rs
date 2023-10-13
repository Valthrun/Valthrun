use anyhow::Context;
use map_tracer::KV3Value;

use super::{
    Hull,
    Mesh,
};

pub struct Shape {
    pub collision_attribute_indices: Vec<u64>,

    pub hulls: Vec<Hull>,
    pub meshes: Vec<Mesh>,
    /* currently not in use */
    /* m_capsules: Vec<MCapsules>, */
    /* currently not in use */
    /* m_spheres: Vec<Sphere>, */
}

impl Shape {
    // value is a MRnShape object
    pub fn parse(value: &KV3Value) -> anyhow::Result<Self> {
        let collision_attribute_indices = value
            .get("m_CollisionAttributeIndices")
            .context("missing m_CollisionAttributeIndices")?
            .as_array()
            .context("expected an array for m_CollisionAttributeIndices")?
            .iter()
            .map(|value| value.as_u64())
            .try_collect::<Vec<_>>()
            .context("failed to parse all m_CollisionAttributeIndices as u64")?;

        let meshes = value
            .get("m_meshes")
            .context("missing meshes")?
            .as_array()
            .context("meshes should be an array")?
            .iter()
            .map(|mesh_info| Mesh::parse(mesh_info))
            .try_collect::<Vec<_>>()?;

        let hulls = value
            .get("m_hulls")
            .context("missing hulls")?
            .as_array()
            .context("hulls should be an array")?
            .iter()
            .map(|hull_info| Hull::parse(hull_info))
            .try_collect::<Vec<_>>()?;

        Ok(Self {
            collision_attribute_indices,
            meshes,
            hulls,
        })
    }
}
