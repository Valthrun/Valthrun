use std::{
    io,
    process::Command,
};

use winres::WindowsResource;

fn main() -> io::Result<()> {
    {
        let git_hash = String::from_utf8(
            Command::new("git")
                .args(&["rev-parse", "HEAD"])
                .output()?
                .stdout,
        )
        .expect("the git hash to be utf-8");

        println!("cargo:rustc-env=GIT_HASH={}", &git_hash[0..7]);
    }

    {
        let mut resource = WindowsResource::new();
        resource.set_icon("./resources/app-icon.ico");
        resource.compile()?;
    }
    Ok(())
}
