# AMD driver issues
Some AMD users may experience some issues when trying to run the Valthrun controller.  
This issue manifest in various error messages, but two common ones are:
- `Unable to find a Vulkan driver`
- `Failed to load vulkan-1.dll (os error 14001)`
- The overlay is just black instead of being transparent

The precise cause of these issues remains unknown, leading to an absence of a universally accepted solution.  
However, various methods have been reported by some users as effective in addressing these problems.  

# Potential Solutions

## 1. **Downgrading AMD Driver to 22.11.2**
For some affected users, downgrading the AMD driver to version 22.11.2 has proven to be a viable solution.  
To perform this downgrade, users can obtain the driver from the official AMD website or via the following link:
[AMD Driver 22.11.2](https://www.amd.com/de/support/kb/release-notes/rn-rad-win-22-11-2)

Discord discussion:  
https://discord.com/channels/1135362291311849693/1135362291311849698/1154795646344241303

## 2. **Using AMD's Pro Drivers**
For certain users encountering this issue, employing AMD's professional-grade drivers may offer a solution.  
To install the AMD pro drivers, use DDU (Display Driver Uninstaller) and install the AMD pro drivers.

## 3. **Copying vulkan-1.dll from Chrome**
Another approach that has been successful for some users is copying the "vulkan-1.dll" file from their local installation of the Google Chrome web browser and pasting it into the directory where the "controller.exe" for Valthrun is located. 
This method has resolved the issue for some, making it a potential workaround for those experiencing driver-related problems.  
  
The `vulkan-1.dll` shipped with Chrome can usually be found in the following folder:  
`C:\Program Files (x86)\Google\Chrome\Application\` followed by a folder with the Chrome version number.  
  
**Note:**  
If this does not solves your issue, ensure you're deleting the `vulkan-1.dll` located along with the `controller.exe`.  
Keeping the `vulkan-1.dll` may cause some issues on its own.