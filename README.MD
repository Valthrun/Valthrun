# Valthrun CS2
Valthrun an open source CS2 external read only kernel cheat.  
That's a lot of descriptive words, but what does each of them mean?  
- Valthrun; The name of this project
- Open Source; This application is open source and for everbody to study
- CS2; This cheat is created for Counter Strike 2
- external; We do not inject anything into CS2
- read only; We do not write to the CS2 process in any way, therefore being impossible to detect by scanning the process memory
- kernel; We do not use any user level WinAPIs in order to get information from the CS2 process
  
Note:  
This project has been inspired by [GarHal_CSGO](https://github.com/dretax/GarHal_CSGO).  
As GarHal this is a lerning project!
  
# Overlay
The overlay is currently in development and not fully ready.

# Build
## Kernel Module
```ps1
# Switch to the kernel driver directory
cd kernel/driver

# Create a release overlay build
# The result will be located at "../target/x86_64-pc-windows-msvc/release/valthrun-driver.sys"
cargo make --profile production sign
```

## Overlay
```ps1
# Create a release overlay build
# The result will be located at "target/release/controller"
cargo build --release
```
    
### Attention  
As long as https://github.com/rust-lang/rust/issues/111540 has not been finalized,  
the controller build file will contain `valthrun` as well as the workspace path in the final executable.  
To omit any traces of valthrun and your build path you need to set the RUSTFLAGS remap path prefix flags accordingly.
```ps1
$WorkspaceCargo=$(cargo locate-project --workspace --message-format=plain)
$env:RUSTFLAGS="-Clink-arg=/PDBALTPATH:C:\build\application.pdb --remap-path-prefix=$($WorkspaceCargo.TrimEnd("Cargo.toml"))=[src] --remap-path-prefix=$env:CARGO_HOME\registry\src\=[crates.io]"
```
Build the overlay afterwards and `valthrun` not the build path (which might include the word `valthrun`) should be contained within the binary.  


# Using
1. Load Driver
2. Start CS2
3. Start Overlay
When you want extra/verbose output, start the overlay from the terminal and previously set the `RUST_LOG` environment variable.  
Example for executing the overlay with trace output:  
```ps1
$env:RUST_LOG="trace"
.\controller.exe
```

# Screenshots
![](https://img.did.science/2023_07_30_17_28_23.png)