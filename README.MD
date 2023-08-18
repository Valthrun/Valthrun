![Valthrun CS2 Logo](./logo.svg)
<p align="right">
<a href="https://discord.gg/ecKbpAPW5T">
<img src="https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield">
</a>
</p>

Valthrun an open source external Counter-Strike 2 read only kernel gameplay enhancer.  
That's a lot of descriptive words, but what does each of them mean?  
- `Valthrun` The name of this project
- `Open Source` This application is open source and for everbody to study
- `external` We do not inject any ddls into the target process
- `Counter-Strike 2` The game we're enhancing
- `read only` We do not write to the CS2 process in any way, therefore being impossible to detect by scanning the process memory.
- `kernel` We do not use any user level WinAPIs in order to get information from the CS2 process
  
This project is mainly a fun example for exploring the Windows Kernel with [Rust](https://www.rust-lang.org) and exploring the world of game enhancements :)

# Features
Valthrun provides a variety of external tools to enchant your gameplay experience.  
As of now, Valthrun only reads memory and never writes anything. This implies some limitations as which features could be provided.  
An example would be a skin changer which is not be possible without actively modifying the CS2 game state (e.g. writing into it's memory).  
  
Regardless of this limitation, Valthrun supports the following features:  
- Player ESP
  Two modes are supported: `Skeleton` and `Boxes`
  - Configurable colors to distinguish between enemy and team players
- Bomb Info
  - Time until the bomb detonation
  - Defuser info such as how long until the defuse has succeeded
  - Bomb site where the bomb is located
- Trigger Bot

To access Valthruns settings overlay press `PAUSE`.

## Planned Features
- Aim Bot
- Spectator info
  - List of player currently watching you / the observer target
- Player competitive ranks / wins
  (I don't have access to the closed beta... so this has to wait a little)

Mess around:
- `KeUserModeCallback`
https://www.unknowncheats.me/forum/anti-cheat-bypass/347320-communication-via-callbacks.html
- Invoking Shellcode within CS (send text message, change player name etc.)

# Installation
Valthrun does not need to be installed.  
In order to run Valthrun you only need the kernel driver (`valthrun-driver.sys`) and the controller (`controller.exe`).  
  
Both can be accquired by downloading the precompiled release from GitHub.  
Click here: TODO!  
  
Once downloaded read [How to use](#how-to-use) to get started!

# How to use
The following steps need to be done for starting/activating the overlay.
1. Load the kernel driver  
   The overlay requires the kernel driver for all memory reads and writes.
   Therefore we need to load the driver before we can start the overlay. It's not strictly required to load the driver before starting CS2 but logically it makes sense to start with the driver.
     
   There are two common options to load the kernel driver:
   1. Manually map the driver using a vulnerable driver
      The Valthrun kernel driver fully supports manual driver mapping using a any   vulnerable driver. In order to detect the manual mapping the second driver entry argument (the   registry patch) must be a null ptr.  
      The kernel driver will automatically allocate a new driver for io communication.  

   2. Setting Windows into test-signing mode  
      TODO: Explain how to set Windows into test signing
      Helpfull guide: https://github.com/dretax/GarHal_CSGO#starting-driver

      In order to manually map the Valthrun driver you can use tools like  
      - [kdmapper](https://github.com/TheCruZ/kdmapper)
      - [KDU](https://github.com/hfiref0x/KDU)
   
2. Start Counter-Strike 2  
After loading the driver you can start Counter-Strike 2 as usual.  

3. Start Overlay  
With an open instance of Counter-Strike 2 you can start the overlay.  
Attention: You *must* start the overlay as administrator!  
  
To explore all command line options of the controller, start the controller from the terminal and pass `-h` to display the command line interface:
```ps1
.\controller.exe -h
```

When you want extra/verbose output, start the overlay from the terminal and previously set the `RUST_LOG` environment variable.  
Example for executing the overlay with trace output:  
```ps1
$env:RUST_LOG="trace"
.\controller.exe
```

# VAC
The same considerations as mentioned in [this link](https://github.com/dretax/GarHal_CSGO#starting-driver) have been taken into account.  
With these precautions and some minor improvements, such as omitting the valthrun identifier and using xor-ing for strings, the driver/overlay should avoid VAC detection. However, I must clarify that I haven't extensively studied VAC, so my conclusion is speculative. Personally, I have been using a C based driver/overlay like this with CSGO for several years without ever getting VAC banned. But be aware of overwatch!  
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
[![Discord Shield](https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield)](https://discord.gg/ecKbpAPW5T)