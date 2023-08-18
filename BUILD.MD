# Building Valthrun
## 0. Prerequiresits
The following prerequisites must be fullfilled to build this project
- Visual Studio 2022 (located at `%ProgramFiles%\Microsoft Visual Studio\2022\Community`)
- Rust
- Windows Kernel Development Kit

## 1. Kernel Driver
Currently the kernel driver is private. Use the prebuild kernel driver.  
```ps1
# Switch to the kernel driver directory
cd kernel/driver

# Create a release build.
# The result will be located at "../target/x86_64-pc-windows-msvc/release/valthrun-driver.sys"
cargo make --profile production sign
```

## 2. Overlay
```ps1
# Create a release overlay build
# The result will be located at "target/release/controller"
cargo build --release
```
    
#### Attention  
As long as https://github.com/rust-lang/rust/issues/111540 has not been finalized,  
the controller build file will contain `valthrun` as well as the workspace path in the final executable.  
As far as I'm conserned VAC does not actively checks for certain strings so this should not be an issue.
But if you want to these traces anyways you need to set the RUSTFLAGS remap path prefix flags accordingly.
```ps1
$WorkspaceCargo=$(cargo locate-project --workspace --message-format=plain)
$env:RUSTFLAGS="-Clink-arg=/PDBALTPATH:C:\build\application.pdb --remap-path-prefix=$($WorkspaceCargo.TrimEnd("Cargo.toml"))=[src] --remap-path-prefix=$env:CARGO_HOME\registry\src\=[crates.io]"
```
Build the overlay afterwards and `valthrun` not the build path (which might include the word `valthrun`) should be contained within the binary.  


# Updating signatures/offsets for Counter-Strike 2
Many offsets are dynamically resolved through signatures.  
But signatures may break with feature versions of the game and therefore need to be updated. Currently *most* signatures can be found [here](https://github.com/WolverinDEV/Valthrun/blob/master/cs2/src/offsets.rs).   
Some time I may write a guide on how to find them, but right now you ether know how to do that, or just need to wait for somebody else to update them.
