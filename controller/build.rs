use std::{
    io::{
        self,
        ErrorKind,
    },
    path::Path,
    process::Command,
};

use chrono::Utc;
use winres::WindowsResource;

const APP_MANIFEST: &'static str = r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <description>Valthrun Overlay</description>
  <assemblyIdentity type="win32" name="dev.wolveringer.valthrun.overlay" version="0.4.5.0" />
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
      <security>
          <requestedPrivileges>
              <requestedExecutionLevel level="asInvoker" uiAccess="false" />
          </requestedPrivileges>
      </security>
  </trustInfo>
  <asmv3:application xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
    <asmv3:windowsSettings xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">
      <dpiAware>True/PM</dpiAware>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
"#;

fn main() -> io::Result<()> {
    {
        let git_hash = if Path::new("../.git").exists() {
            match { Command::new("git").args(&["rev-parse", "HEAD"]).output() } {
                Ok(output) => String::from_utf8(output.stdout).expect("the git hash to be utf-8"),
                Err(error) => {
                    if error.kind() == ErrorKind::NotFound {
                        panic!("\n\nBuilding the controller requires git.exe to be installed and available in PATH.\nPlease install https://gitforwindows.org.\n\n");
                    }

                    return Err(error);
                }
            }
        } else {
            "0000000".to_string()
        };

        if git_hash.len() < 7 {
            panic!("Expected the git hash to be at least seven characters long");
        }

        let build_time = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        println!("cargo:rustc-env=GIT_HASH={}", &git_hash[0..7]);
        println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    }

    {
        let mut resource = WindowsResource::new();
        resource.set_icon("./resources/app-icon.ico");
        resource.set_manifest(APP_MANIFEST);
        resource.compile()?;
    }
    Ok(())
}
