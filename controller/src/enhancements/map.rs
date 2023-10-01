use std::{fs::File, io::{BufReader, Cursor, Read}};

use anyhow::Context;
use byteorder::{ReadBytesExt, LittleEndian, ByteOrder};
use imgui::ImColor32;
use map_tracer::{VPKArchiveReader, Resource, read_kv3, KV3Value};

use crate::{view::ViewController, settings::AppSettings};

use super::Enhancement;

pub struct MapVis {
    meshes: Vec<Mesh>
}

trait ReadVecEx {
    fn read_vec3_f32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<f32>>;
    fn read_vec3_u32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<u32>>;
}

impl<T> ReadVecEx for T
where
    T: Read
{
    fn read_vec3_f32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<f32>> {
        let x = self.read_f32::<B>()?;
        let y = self.read_f32::<B>()?;
        let z = self.read_f32::<B>()?;
        Ok(nalgebra::Vector3::new(x, y, z))
    }
    
    fn read_vec3_u32<B: ByteOrder>(&mut self) -> std::io::Result<nalgebra::Vector3<u32>> {
        let x = self.read_u32::<B>()?;
        let y = self.read_u32::<B>()?;
        let z = self.read_u32::<B>()?;
        Ok(nalgebra::Vector3::new(x, y, z))
    }
}

#[derive(Debug)]
struct MeshNode {
    pub min: nalgebra::Vector3<f32>,
    pub max: nalgebra::Vector3<f32>,

    /// 0xC0000000 -> Node is end node
    pub children: u32,
    pub triangle_offset: u32
}

impl MeshNode {
    pub fn parse<R>(buffer: &mut R) -> anyhow::Result<Self> 
    where
        R: Read
    {
        let min = buffer.read_vec3_f32::<LittleEndian>()?;
        let children = buffer.read_u32::<LittleEndian>()?;

        let max = buffer.read_vec3_f32::<LittleEndian>()?;
        let triangle_offset = buffer.read_u32::<LittleEndian>()?;
        Ok(Self {
            min,
            max,

            children,
            triangle_offset
        })
    }
}

struct Mesh {
    v_min: nalgebra::Vector3<f64>,
    v_max: nalgebra::Vector3<f64>,

    materials: Vec<u32>,
    nodes: Vec<MeshNode>,
    vertices: Vec<nalgebra::Vector3<f32>>,
    triangles: Vec<nalgebra::Vector3<u32>>,

    orthographic_areas: nalgebra::Vector3<f64>,
}

impl Mesh {
    pub fn parse(value: &KV3Value) -> anyhow::Result<Self> {
        let mesh2 = value.get("m_Mesh").context("missing m_Mesh")?;
        
        let v_min = mesh2.get("m_vMin").context("missing m_vMin")?.as_vec3_f64().context("expected a Vec3 for m_vMin")?;
        let v_max = mesh2.get("m_vMax").context("missing m_vMax")?.as_vec3_f64().context("expected a Vec3 for m_vMax")?;

        let materials = mesh2.get("m_Materials").context("missing m_Materials")?
            .as_array().context("expected an array for m_Materials")?
            .iter()
            .map(|value| value.as_u32())
            .try_collect::<Vec<_>>()
            .context("failed to parse all m_Materials as u32")?;

        let nodes = mesh2.get("m_Nodes").context("missing m_Nodes")?
            .as_binary().context("expected m_Nodes to be binary")?
            .chunks_exact(32)
            .map(|entry| MeshNode::parse(&mut Cursor::new(entry)))
            .try_collect::<Vec<_>>()
            .context("failed to read node data")?;

        let vertices = mesh2.get("m_Vertices").context("missing m_Vertices")?
            .as_binary().context("expected m_Vertices to be binary")?
            .chunks_exact(12)
            .map(|buffer| Cursor::new(buffer).read_vec3_f32::<LittleEndian>())
            .try_collect::<Vec<_>>()
            .context("failed to read vertices data")?;

        let triangles = mesh2.get("m_Triangles").context("missing m_Triangles")?
            .as_binary().context("expected m_Triangles to be binary")?
            .chunks_exact(12)
            .map(|buffer| Cursor::new(buffer).read_vec3_u32::<LittleEndian>())
            .try_collect::<Vec<_>>()
            .context("failed to read triangles data")?;

        let orthographic_areas = mesh2.get("m_vOrthographicAreas").context("missing m_vOrthographicAreas")?
            .as_vec3_f64().context("expected a Vec3 for m_vOrthographicAreas")?;

        for node in &nodes[0..50.min(nodes.len())] {
            log::debug!("{:?} {:X}", node, node.children);
        }
        log::debug!("materials = {}, nodes = {}, vertices = {}, triangles = {}", materials.len(), nodes.len(), vertices.len(), triangles.len());
        Ok(Self {
            v_min,
            v_max,

            materials,
            nodes,
            vertices,
            triangles,

            orthographic_areas
        })
    }
}

impl MapVis {
    pub fn new() -> anyhow::Result<Self> {
        let file = File::open("../map-tracer/assets/de_mirage.vpk")?;
        let reader = BufReader::new(file);
        let mut archive = VPKArchiveReader::new(reader)?;

        let physics_path = archive.entries().keys()
            .find(|entry| entry.ends_with("world_physics.vphys_c"))
            .context("missing world physics entry")?
            .to_string();

        let physics_data = archive.read_entry(&physics_path)?;
        let physics_data = Cursor::new(physics_data);
        let mut physics_resource = Resource::new(physics_data)?;
        let data_block_index = physics_resource.blocks()
            .iter()
            .position(|block| block.block_type == "DATA")
            .context("failed to find work physics data block")?;
        let physics_data = physics_resource.read_block(data_block_index)?;
        let mut physics_data = Cursor::new(physics_data);
        let physics_data = read_kv3(&mut physics_data)?;

        let map_parts = physics_data.get("m_parts").context("missing parts")?.as_array().context("expected array")?;
        if map_parts.len() != 1 {
            anyhow::bail!("expected only one map part");
        }
        let map_part = map_parts.get(0).unwrap();

        let meshes = map_part
            .get("m_rnShape").context("shape to be an object")?
            .get("m_meshes").context("missing meshes")?
            .as_array().context("meshes should be an array")?;

        let meshes = meshes.iter()
            .map(|mesh_info| Mesh::parse(mesh_info))
            .try_collect::<Vec<_>>()?;

        // FIXME: Parse mesh flags & type!
        // for part in physics_data.get("m_parts").context("missing parts")?.as_array().context("expected array")? {
        //     log::debug!("Part {:X}", part.get("m_nFlags").context("missing flags")?.as_u64().context("expected u64")?);
        //     let hulls = part.get("m_rnShape").context("missing shape")?.get("m_hulls").context("missing hulls")?.as_array().context("expected array")?;
        //     log::debug!(" Hulls: {}", hulls.len());
        // }

        Ok(Self {
            meshes
        })
    }
}

impl Enhancement for MapVis {
    fn update(&mut self, _ctx: &crate::UpdateContext) -> anyhow::Result<()> {
        Ok(())
    }

    fn render(&self, _settings: &AppSettings, ui: &imgui::Ui, view: &ViewController) {
        let draw = ui.get_window_draw_list();
        // for node in self.mesh.nodes.iter() {
        //     view.draw_box_3d(&draw, &node.min, &node.max, ImColor32::from_rgb(0xFF, 0x0, 0x00), 1.0);
        // }
        for mesh in self.meshes.iter() {
            for triangle in mesh.triangles.iter() {
                if triangle.x as usize >= mesh.vertices.len() {
                    continue;
                }
                if triangle.y as usize >= mesh.vertices.len() {
                    continue;
                }  
                if triangle.z as usize >= mesh.vertices.len() {
                    continue;
                }

                let (point_a, dist) = match view.world_to_screen2(&mesh.vertices[triangle.x as usize], true) {
                    Some(p) => p,
                    None => continue,
                };
    
                let point_b = match view.world_to_screen(&mesh.vertices[triangle.y as usize], true) {
                    Some(p) => p,
                    None => continue,
                };
    
                let point_c = match view.world_to_screen(&mesh.vertices[triangle.z as usize], true) {
                    Some(p) => p,
                    None => continue,
                };
    
                draw.add_triangle(point_a, point_b, point_c, ImColor32::from_rgb((256.0 * dist.clamp(0.0, 5000.0) / 5000.0) as u8, 0x0, 0x00))
                    .thickness(1.0)
                    .build();
            }
        }
    }

    fn render_debug_window(&mut self, _settings: &mut AppSettings, _ui: &imgui::Ui) {
        
    }
}