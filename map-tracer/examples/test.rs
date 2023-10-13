use std::{
    fs::File,
    io::{
        BufReader,
        Cursor,
        Write,
    },
};

use anyhow::Context;
use map_tracer::{
    KV3Value,
    Resource,
    VPKArchiveReader,
};

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    log::debug!("Open file!");
    let file = File::open("G:\\git\\rust\\valthrun\\map-tracer\\assets\\de_mirage.vpk")?;
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
    let mut physics_data = KV3Value::parse(&mut physics_data)?;

    // for part in physics_data.get("m_parts").context("missing parts")?.as_array().context("expected array")? {
    //     log::debug!("Part {:X}", part.get("m_nFlags").context("missing flags")?.as_u64().context("expected u64")?);
    //     let hulls = part.get("m_rnShape").context("missing shape")?.get("m_hulls").context("missing hulls")?.as_array().context("expected array")?;
    //     log::debug!(" Hulls: {}", hulls.len());
    // }

    {
        let mut file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open("map_info.json")?;

        let payload = serde_json::to_string_pretty(&physics_data)?;
        file.write_all(payload.as_bytes())?;
    }
    Ok(())
}
