# Loading the Valthrun Driver (`valthrun-driver.sys`)  
Loading or mapping a driver can be a complex and challenging task.  
Typically, drivers are signed with an "Extended Validation Code Sign Certificate."  
However, acquiring these certificates can be difficult, and they can be easily blocked by third-party software like VAC (Valve Anti-Cheat).  
As a result, loading the Valthrun driver requires unconventional methods.

## Supported Windows Versions  
Valthrun aims to be compatible with a wide range of Windows versions.  
All recent Windows versions are expected to be supported, as the functions and struct offsets are resolved dynamically.  
The latest Windows 10 and Windows 11 versions, such as 22H2, have been tested successfully.  
User feedback also suggests that Windows versions as far back as 20H2 are working.  
If you encounter any issues, please submit an issue report that includes your Windows version and a description of the error you encounter.

## Overview of Mapping Methods
Here is a quick overview of various methods that can be used to load the Valthrun driver:

| Method | Complexity | Success Rate |
| --- | --- | --- |
| [Manual Mapping via KD-Mapper](010_getting-started/010_mapping-method/010_kdmapper.md) | Medium | High |
| [Manual Mapping via KDU](010_getting-started/010_mapping-method/020_kdu.md) | Medium | Medium |
| [Using Windows Test-Signing](010_getting-started/010_mapping-method/030_test-signing.md) | Low | Very High |
| [Manual Mapping via Other Mappers](010_getting-started/010_mapping-method/040_other-mappers.md) | Unknown | Unknown |

Each method has its own level of complexity and success rate.  
The choice of which method to use will depend on your specific needs and the compatibility of your system.  
Further details about each mapping method can be found in the linked resources.

## Mapping with Community-Made Scripts
In addition to the manual methods of mapping the Valthrun driver, there are community-made scripts available that simplify the process.  
These scripts are often developed and shared by members of the Valthrun community.  
While these scripts can be convenient, it's important to note that they are not officially provided by Valthrun, and their success rate may vary. 
  
- [Valthrunners' Script](010_getting-started/010_mapping-method/110_community_script_valthrunner.md)
  
TODO: Add the other scripts