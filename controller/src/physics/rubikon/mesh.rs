use std::io::{
    Cursor,
    Read,
};

use anyhow::Context;
use byteorder::{
    LittleEndian,
    ReadBytesExt,
};
use map_tracer::KV3Value;

use super::ReadVecEx;

#[derive(Debug)]
pub struct MeshNode {
    pub min: nalgebra::Vector3<f32>,
    pub max: nalgebra::Vector3<f32>,

    /// 0xC0000000 -> Node is end node
    pub children: u32,
    pub triangle_offset: u32,
}

impl MeshNode {
    pub fn flags(&self) -> u8 {
        (self.children >> 28) as u8
    }

    pub fn children(&self) -> usize {
        (self.children & 0x0FFFFFFF) as usize
    }

    pub fn is_leaf(&self) -> bool {
        self.flags() == 0x0C
    }

    pub fn contains(&self, point: &nalgebra::Vector3<f32>) -> bool {
        self.min.x <= point.x
            && point.x <= self.max.x
            && self.min.y <= point.y
            && point.y <= self.max.y
            && self.min.z <= point.z
            && point.z <= self.max.z
    }
}

impl MeshNode {
    pub fn parse<R>(buffer: &mut R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let min = buffer.read_vec3_f32::<LittleEndian>()?;
        let children = buffer.read_u32::<LittleEndian>()?;

        let max = buffer.read_vec3_f32::<LittleEndian>()?;
        let triangle_offset = buffer.read_u32::<LittleEndian>()?;
        Ok(Self {
            min,
            max,

            children,
            triangle_offset,
        })
    }
}

pub struct Mesh {
    pub user_friendly_name: String,
    pub collision_attribute_index: usize,
    pub surface_property_index: usize,

    pub v_min: nalgebra::Vector3<f64>,
    pub v_max: nalgebra::Vector3<f64>,

    pub materials: Vec<u32>,
    pub nodes: Vec<MeshNode>,
    pub vertices: Vec<nalgebra::Vector3<f32>>,
    pub triangles: Vec<nalgebra::Vector3<u32>>,

    pub orthographic_areas: nalgebra::Vector3<f64>,
}

impl Mesh {
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
            .context("expected a String for m_nCollisionAttributeIndex")?
            as usize;
        let surface_property_index = value
            .get("m_nSurfacePropertyIndex")
            .context("missing m_nSurfacePropertyIndex")?
            .as_u64()
            .context("expected a String for m_nSurfacePropertyIndex")?
            as usize;

        let mesh2 = value.get("m_Mesh").context("missing m_Mesh")?;
        let v_min = mesh2
            .get("m_vMin")
            .context("missing m_vMin")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vMin")?;
        let v_max = mesh2
            .get("m_vMax")
            .context("missing m_vMax")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vMax")?;

        let materials = mesh2
            .get("m_Materials")
            .context("missing m_Materials")?
            .as_array()
            .context("expected an array for m_Materials")?
            .iter()
            .map(|value| value.as_u32())
            .try_collect::<Vec<_>>()
            .context("failed to parse all m_Materials as u32")?;

        let nodes = mesh2
            .get("m_Nodes")
            .context("missing m_Nodes")?
            .as_binary()
            .context("expected m_Nodes to be binary")?
            .chunks_exact(32)
            .map(|entry| MeshNode::parse(&mut Cursor::new(entry)))
            .try_collect::<Vec<_>>()
            .context("failed to read node data")?;

        let vertices = mesh2
            .get("m_Vertices")
            .context("missing m_Vertices")?
            .as_binary()
            .context("expected m_Vertices to be binary")?
            .chunks_exact(12)
            .map(|buffer| Cursor::new(buffer).read_vec3_f32::<LittleEndian>())
            .try_collect::<Vec<_>>()
            .context("failed to read vertices data")?;

        let triangles = mesh2
            .get("m_Triangles")
            .context("missing m_Triangles")?
            .as_binary()
            .context("expected m_Triangles to be binary")?
            .chunks_exact(12)
            .map(|buffer| Cursor::new(buffer).read_vec3_u32::<LittleEndian>())
            .try_collect::<Vec<_>>()
            .context("failed to read triangles data")?;

        let orthographic_areas = mesh2
            .get("m_vOrthographicAreas")
            .context("missing m_vOrthographicAreas")?
            .as_vec3_f64()
            .context("expected a Vec3 for m_vOrthographicAreas")?;

        log::debug!(
            "materials = {}, nodes = {}, vertices = {}, triangles = {}",
            materials.len(),
            nodes.len(),
            vertices.len(),
            triangles.len()
        );
        Ok(Self {
            user_friendly_name,
            surface_property_index,
            collision_attribute_index,

            v_min,
            v_max,

            materials,
            nodes,
            vertices,
            triangles,

            orthographic_areas,
        })
    }

    pub fn resolve_node(
        &self,
        point: &nalgebra::Vector3<f32>,
    ) -> anyhow::Result<Option<&MeshNode>> {
        let mut current_index = 0;
        let mut current_node = self
            .nodes
            .get(current_index)
            .context("missing initial node")?;
        if !current_node.contains(point) {
            return Ok(None);
        }

        let mut iterations = 0;
        while iterations < 1000 {
            iterations += 1;
            if current_node.is_leaf() {
                /* end reached */
                return Ok(Some(current_node));
            }

            let next_a = self
                .nodes
                .get(current_index + 1)
                .with_context(|| format!("missing next node A {}", current_index + 1))?;
            if next_a.contains(point) {
                current_index = current_index + 1;
                current_node = next_a;
                continue;
            }

            let next_b = self
                .nodes
                .get(current_index + current_node.children())
                .with_context(|| {
                    format!(
                        "missing next node A {}",
                        current_index + current_node.children()
                    )
                })?;
            if next_b.contains(point) {
                current_index = current_index + current_node.children();
                current_node = next_b;
                continue;
            }

            return Ok(Some(current_node));
        }

        anyhow::bail!("node depth too deep or it includes a circle")
    }
}
