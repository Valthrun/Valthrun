use std::io;

use winres::WindowsResource;

const MANIFEST: &'static str = include_str!("./manifest.xml");
fn main() -> io::Result<()> {
    // let execution_level = if cfg!(feature = "require-administrator") {
    //     "requireAdministrator"
    // } else {
    //     "asInvoker"
    // };

    // let mut resource = WindowsResource::new();
    // resource.set_icon("../logo-icon.ico");
    // resource.set_manifest(&MANIFEST.replace("{{execution_level}}", execution_level));
    // resource.compile()?;

    Ok(())
}
