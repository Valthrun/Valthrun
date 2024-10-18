use std::{
    env,
    fs::File,
    io::BufReader,
    path::PathBuf,
    str::FromStr,
};

use anyhow::Context;
use cs2_schema_definition::SchemaScope;

fn main() -> anyhow::Result<()> {
    let mut schema =
        BufReader::new(File::open("./cs2_schema.json").context("failed to open cs2_schema.json")?);
    let schema_scopes = serde_json::from_reader::<_, Vec<SchemaScope>>(&mut schema)
        .context("failed to parse schema")?;

    let dest_path = PathBuf::from_str(&env::var("OUT_DIR")?)?;
    cs2_schema_definition::emit_to_dir(&dest_path, &schema_scopes)?;

    println!("Target dir: {}", dest_path.display());
    println!("cargo:rerun-if-changed=cs2_schema.json");
    Ok(())
}
