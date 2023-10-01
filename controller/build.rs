use std::io;

use winres::WindowsResource;

fn main() -> io::Result<()> {
    let mut resource = WindowsResource::new();
    resource.set_icon("../logo-icon.ico");
    #[cfg(feature = "exe-manifest")]
    {
        const MANIFEST: &'static str = include_str!("./manifest.xml");
        resource.set_manifest(MANIFEST);
    }

    resource.compile()?;
    Ok(())
}
