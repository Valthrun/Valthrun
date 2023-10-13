![Valthrun CS2 Logo](./_media/logo.svg)
<p align="right">
<a href="https://discord.gg/ecKbpAPW5T">
<img src="https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield">
</a>
</p>

Valthrun is an open source external Counter-Strike 2 read only kernel-level gameplay enhancer.  
That's a lot of descriptive words, but what does each of them mean?  
- `Valthrun` The name of this project
- `Open Source` This application is open source and for everyone to learn from
- `external` We do not inject any DLLs into the target process
- `read only` We do not write to the CS2 process in any way, therefore being impossible to detect by scanning the process memory
- `kernel` We do not use any user level WinAPIs in order to get information from the CS2 process
  
This project is mainly a fun example for exploring the Windows Kernel with [Rust](https://www.rust-lang.org) and exploring the world of game enhancements :)

# WARNING <!-- {docsify-ignore-all} --> 
Valthrun is **not** plug 'n play.  
Please read [How to use](#how-to-use) carefully and try troubleshooting issues on your own.  
The goal is to achieve maximum stealth in order to avoid being detected.
  
# Features
Due to Valthrun being read-only (as of now), there are limitations on what features are possible to implement (eg. skin changer).
Regardless of this limitation, Valthrun supports the following features:  

- Player ESP
  Two modes are supported: `Skeleton` and `Boxes`
  - Configurable colors to distinguish between enemy and team players
  - ESP includes player health
- Bomb Info
  - Time until the bomb detonation
  - Defuser info such as a defuse timer
  - Bomb site where the bomb is located
- Trigger Bot
- Stream proof by default

To access Valthruns settings overlay press `PAUSE`.

## Planned Features
- Aimbot
- Spectator info
  - List of player currently watching you / the observer target
- Player competitive ranks / wins

# VAC
The same considerations as mentioned in [this link](https://github.com/dretax/GarHal_CSGO#starting-driver) have been taken into account.  
With these precautions and some minor improvements, such as omitting the Valthrun identifier and using xor encryption for strings, the driver/overlay should avoid VAC detection. However, I must clarify that I haven't extensively studied VAC, so my conclusion is speculative. Personally, I have been using a C based driver/overlay like this with CSGO for several years without ever getting VAC banned. But be aware of overwatch!  
With VAC live being enabled now, use this with caution. As always take the necessary precautions into consideration.

# Screenshots
![](./_media/showcase_01.png)
![](./_media/showcase_02.png)

# Help
You can find help on the official Valthrun Discord server:  
[![Discord Shield](https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield)](https://discord.gg/ecKbpAPW5T)
