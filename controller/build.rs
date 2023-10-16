use std::{
    io::{
        self,
        ErrorKind,
    },
    path::Path,
    process::Command,
};

use winres::WindowsResource;

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

        println!("cargo:rustc-env=GIT_HASH={}", &git_hash[0..7]);
    }

    {
        let mut resource = WindowsResource::new();
        resource.set_icon("./resources/app-icon.ico");
        resource.compile()?;
    }
    Ok(())
}
