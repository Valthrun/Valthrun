use std::{
    env,
    fs::File,
    io::BufReader,
};

use anyhow::Context;
use cs2_schema_definition::SchemaScope;

fn main() -> anyhow::Result<()> {
    let mut args = env::args().into_iter();
    let _application_name = args.next();

    let Some(src_file) = args.next() else {
        anyhow::bail!("please specify a source file");
    };
    let Some(dst_dir) = args.next() else {
        anyhow::bail!("please specify a destination directory");
    };

    println!("Reading schema");
    let mut reader = BufReader::new(File::open(src_file).context("open")?);
    let schema_scopes =
        serde_json::from_reader::<_, Vec<SchemaScope>>(&mut reader).context("parse schema")?;

    println!("Emitting Rust definition");
    cs2_schema_definition::emit_to_dir(dst_dir, &schema_scopes)?;

    println!("Success");
    Ok(())
}
