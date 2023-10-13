use std::{
    fs::File,
    io::{
        BufReader,
        Cursor,
    },
    path::{
        Path,
        PathBuf,
    },
    time::Instant,
};

use anyhow::Context;
use map_tracer::{
    KV3Value,
    Resource,
    VPKArchiveReader,
};

pub mod rubikon;

mod ray_trace;
pub use ray_trace::*;

pub struct WorldPhysics {
    world_name: String,
    world_file_path: PathBuf,

    collision_attributes: Vec<rubikon::CollisionAttribute>,
    shape: rubikon::Shape,
}

impl WorldPhysics {
    fn extract_world_physics_data(path: &Path) -> anyhow::Result<KV3Value> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = VPKArchiveReader::new(reader)?;

        let physics_path = archive
            .entries()
            .keys()
            .find(|entry| entry.ends_with("world_physics.vphys_c"))
            .context("missing world physics entry")?
            .to_string();

        let physics_data = archive.read_entry(&physics_path)?;
        let physics_data = Cursor::new(physics_data);
        let mut physics_resource = Resource::new(physics_data)?;
        let data_block_index = physics_resource
            .blocks()
            .iter()
            .position(|block| block.block_type == "DATA")
            .context("failed to find work physics data block")?;

        let physics_data = physics_resource.read_block(data_block_index)?;
        let mut physics_data = Cursor::new(physics_data);
        Ok(KV3Value::parse(&mut physics_data)?)
    }

    // "../map-tracer/assets/de_mirage.vpk"
    pub fn load(world_file_path: PathBuf) -> anyhow::Result<WorldPhysics> {
        let world_name = world_file_path
            .file_name()
            .context("path is missing a file name")?
            .to_str()
            .context("world file name contains invalid characters")?
            .to_string();

        log::debug!(
            "Loading {} from {}",
            world_name,
            world_file_path.to_string_lossy()
        );
        let time_begin = Instant::now();

        let physics_data = Self::extract_world_physics_data(&world_file_path)?;
        let collision_attributes = physics_data
            .get("m_collisionAttributes")
            .context("missing m_collisionAttributes")?
            .as_array()
            .context("expected m_collisionAttributes to be an array")?
            .into_iter()
            .map(|v| rubikon::CollisionAttribute::parse(v))
            .try_collect::<Vec<_>>()
            .context("failed to parse m_collisionAttributes as Vec<CollisionAttribute>")?;

        let map_parts = physics_data
            .get("m_parts")
            .context("missing parts")?
            .as_array()
            .context("expected array")?;
        if map_parts.len() != 1 {
            anyhow::bail!("expected only one map part");
        }
        let map_part = map_parts.get(0).unwrap();

        let shape =
            rubikon::Shape::parse(map_part.get("m_rnShape").context("shape to be an object")?)
                .context("failed to parse world shape")?;

        // for mesh in &shape.meshes {
        //     log::debug!("Mesh");
        //     log::debug!("  collision_attribute_index = {:?}", collision_attributes.get(mesh.collision_attribute_index).context("missing collision attributes for mesh")?);
        //     log::debug!("  m_CollisionAttributeIndices.len() = {:?}", shape.collision_attribute_indices.len());
        //     log::debug!(" Mesh {} {}", mesh.triangles.len(), mesh.surface_property_index);
        // }

        log::debug!("Loaded {} within {:?}", world_name, time_begin.elapsed());
        Ok(Self {
            world_name,
            world_file_path,

            collision_attributes,
            shape,
        })
    }
}

impl WorldPhysics {
    pub fn world_name(&self) -> &str {
        &self.world_name
    }

    pub fn world_file_path(&self) -> &Path {
        &self.world_file_path
    }

    pub fn shape(&self) -> &rubikon::Shape {
        &self.shape
    }

    pub fn collision_attributes(&self) -> &Vec<rubikon::CollisionAttribute> {
        &self.collision_attributes
    }

    pub fn trace(&self, ray: &Ray) -> Option<RayHit> {
        let mut ray_hit: Option<RayHit> = None;

        /* trace hulls */
        {
            // TODO: Support for hulls
        }

        /* trace meshes */
        {
            let meshes = self.shape.meshes.iter().filter(|mesh| {
                if let Some(attribute) = self
                    .collision_attributes
                    .get(mesh.collision_attribute_index)
                {
                    attribute.group_name == "Default"
                } else {
                    false
                }
            });

            for mesh in meshes {
                if let Some(hit) = ray.trace(&mesh) {
                    if let Some(current_hit) = &ray_hit {
                        if current_hit.t < hit.t {
                            continue;
                        }
                    }

                    ray_hit = Some(hit);
                }
            }
        }

        ray_hit
    }
}
