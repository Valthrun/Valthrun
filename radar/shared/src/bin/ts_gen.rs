use std::fs::File;

use anyhow::Context;
use radar_shared::protocol::{
    C2SMessage,
    HandshakeMessage,
    S2CMessage,
};
use typescript_type_def::{
    write_definition_file_from_type_infos,
    DefinitionFileOptions,
    TypeDef,
};

fn main() -> anyhow::Result<()> {
    let Some(target) = std::env::args().skip(1).next() else {
        anyhow::bail!("please provide a target path")
    };

    let mut options = DefinitionFileOptions::default();
    options.root_namespace = None;
    options.header = Some(
        r#"// *** DO NOT EDIT ***"
// This file has been automatically generated.
// Invoke ts_gen in radar/shared to regenerate this file"#,
    );

    let mut output = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&target)
        .context("open target")?;

    let definitions = &[
        &S2CMessage::INFO,
        &C2SMessage::INFO,
        &HandshakeMessage::INFO,
    ];
    write_definition_file_from_type_infos(&mut output, options, definitions)?;

    println!("Definitions written to {}", target);
    Ok(())
}
