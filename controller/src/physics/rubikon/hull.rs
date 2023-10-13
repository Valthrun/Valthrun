use std::io::Cursor;

use anyhow::Context;
use byteorder::LittleEndian;
use map_tracer::KV3Value;

use super::ReadVecEx;

pub struct Hull {
    pub user_friendly_name: String,
    pub collision_attribute_index: u64,
    pub surface_property_index: u64,

    pub v_max_bounds: nalgebra::Vector3<f64>,
    pub v_min_bounds: nalgebra::Vector3<f64>,

    pub centroid: nalgebra::Vector3<f64>,
    pub orthographic_areas: nalgebra::Vector3<f64>,

    pub vertices: Vec<nalgebra::Vector3<f32>>,
}

impl Hull {
    pub fn parse(value: &KV3Value) -> anyhow::Result<Self> {
        let user_friendly_name = value
            .get("m_UserFriendlyName")
            .context("missing m_UserFriendlyName")?
            .as_str()
            .context("expected a String for m_UserFriendlyName")?
            .to_string();
        let collision_attribute_index = value
            .get("m_nCollisionAttributeIndex")
            .context("missing m_nCollisionAttributeIndex")?
            .as_u64()
            .context("expected a String for m_nCollisionAttributeIndex")?;
        let surface_property_index = value
            .get("m_nSurfacePropertyIndex")
            .context("missing m_nSurfacePropertyIndex")?
            .as_u64()
            .context("expected a String for m_nSurfacePropertyIndex")?;

        let hull2 = value.get("m_Hull").context("missing m_Hull")?;
        let bounds = hull2.get("m_Bounds").context("missing m_Bounds")?;

        let v_max_bounds = bounds
            .get("m_vMaxBounds")
            .context("missing m_vMaxBounds")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vMaxBounds")?;
        let v_min_bounds = bounds
            .get("m_vMinBounds")
            .context("missing m_vMinBounds")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vMinBounds")?;

        let centroid = hull2
            .get("m_vCentroid")
            .context("missing m_vCentroid")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vCentroid")?;
        let orthographic_areas = hull2
            .get("m_vOrthographicAreas")
            .context("missing m_vOrthographicAreas")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vOrthographicAreas")?;

        let vertices = hull2
            .get("m_Vertices")
            .context("missing m_Vertices")?
            .as_binary()
            .context("expected m_Vertices to be binary")?
            .chunks_exact(12)
            .map(|buffer| Cursor::new(buffer).read_vec3_f32::<LittleEndian>())
            .try_collect::<Vec<_>>()
            .context("failed to read vertices data")?;

        Ok(Self {
            user_friendly_name,
            collision_attribute_index,
            surface_property_index,

            centroid,
            v_max_bounds,
            v_min_bounds,

            orthographic_areas,

            vertices,
        })
    }
}
