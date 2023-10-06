use std::io;

use winres::WindowsResource;

fn main() -> io::Result<()> {
    let mut resource = WindowsResource::new();
    resource.set_icon("../logo-icon.ico");
    resource.compile()?;

    Ok(())
}
