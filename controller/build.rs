use std::io;

use winres::WindowsResource;

const MANIFEST: &'static str = include_str!("./manifest.xml");
fn main() -> io::Result<()> {
    let mut resource = WindowsResource::new();
    resource.set_icon("../logo-icon.ico");
    resource.compile()?;

    Ok(())
}
