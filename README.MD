![Valthrun CS2 Logo](./logo.svg)
<p align="right">
<img src="https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield">
</p>
Valthrun an open source external Counter-Strike 2 read only kernel cheat.  
That's a lot of descriptive words, but what does each of them mean?  
- Valthrun; The name of this project
- Open Source; This application is open source and for everbody to study
- external; We do not inject any ddls into the target process
- Counter-Strike 2; This cheat is created for Counter Strike 2
- read only; We do not write to the CS2 process in any way, therefore being impossible to detect by scanning the process memory
- kernel; We do not use any user level WinAPIs in order to get information from the CS2 process
  
Note:  
This project has been inspired by [GarHal_CSGO](https://github.com/WolverinDEV/GarHal_CSGO).  
As GarHal this is a project for exploring the Windows Kernel with [Rust](https://www.rust-lang.org).

# Overlay Features
The basis for the overlay has been build but it's currently pretty limited in terms of functionality.  
Currently supported features are such as drawing player skeletons and their collision box.  
  
To access Valthruns settings overlay press `NUM 0` (the zero on the numpad).  
Attention: Ensure you have `NUMLOCK` enabled!  

## Planned Features
- Improved PlayerESP
  - Different teammate / opponent colors
  - Weapon name
  - Player health
- Bomb Info
  - Time 'till detonation
  - Time 'till defusal
  - Bomb location
- Spectator info
  - List of player currently watching you / the observer target
- Player competitive ranks / wins

Mess around:
- `KeUserModeCallback`
https://www.unknowncheats.me/forum/anti-cheat-bypass/347320-communication-via-callbacks.html
- Invoking Shellcode within CS (send text message, change player name etc.)

# Build
## 0. Prerequiresits
The following prerequisites must be fullfilled to build this project
- Visual Studio 2022 (located at `%ProgramFiles%\Microsoft Visual Studio\2022\Community`)
- Rust
- Windows Kernel Development Kit

## 1. Kernel Driver
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


# Starting Overlay
The following steps need to be done for starting/activating the overlay.
1. Load the kernel driver
The overlay requires the kernel driver for all memory reads and writes.
Therefore we need to load the driver before we can continue.
TODO!  
Helpfull guide: https://github.com/dretax/GarHal_CSGO#starting-driver
  
2. Start Counter-Strike 2
After loading the driver you can start Counter-Strike 2.

3. Start Overlay
With an open instance of Counter-Strike 2 you can start the overlay.  
  
When you want extra/verbose output, start the overlay from the terminal and previously set the `RUST_LOG` environment variable.  
Example for executing the overlay with trace output:  
```ps1
$env:RUST_LOG="trace"
.\controller.exe
```

# Updating offsets for Counter-Strike 2
TODO: Create documentation on this one, as well where and what offsets are used!  
May put this into a seperate file.

# VAC
The same considerations as mentioned in [this link](https://github.com/dretax/GarHal_CSGO#starting-driver) have been taken into account.  
With these precautions and some minor improvements, such as omitting the varhal identifier and using xor-ing for strings, the driver/overlay should avoid VAC detection. However, I must clarify that I haven't extensively studied VAC, so my conclusion is speculative. Personally, I have been using a C based driver/overlay like this with CSGO for several years without ever getting VAC banned. But be aware of overwatch!  
Unfortunately, since I'm not part of the closed CS2 beta,  
I haven't had the opportunity to test the overlay with VAC live,  
but I assume it should work similarly well.

# Supported Windows Versions
All recent windows versions should be supported as all functions / struct offsets are resolved dynamically.  
If you encounter an issue, please submit an issue detailing your Windows version and which error you encounter.  
  
# Screenshots
![](https://img.did.science/2023_07_30_17_28_23.png)
![](https://img.did.science/2023_07_31_01_38_46.png)

# Help
You can find help on the official Valthrun Discord server:  
![Discord Shield](https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield)  