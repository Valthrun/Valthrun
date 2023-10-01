use std::io;

use winres::WindowsResource;

const MANIFEST: &'static str = include_str!("./manifest.xml");
fn main() -> io::Result<()> {
    WindowsResource::new()
        .set_icon("../logo-icon.ico")
        .set_manifest(MANIFEST)
        .compile()?;

    Ok(())
}
