use std::io;

use winres::WindowsResource;

fn main() -> io::Result<()> {
    WindowsResource::new()
        .set_icon("../logo-icon.ico")
        .set_manifest(r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
        <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
        </security>
        </trustInfo>
        </assembly>
        "#)
        .compile()?;

    Ok(())
}
