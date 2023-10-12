# Disable certain Windows security features
By default Windows has enabled a wide varitee of features aiming to protect the kernel from beingtampered with.  
I highly recommand keeping them enabled, but for Valthrun to work a few must be disabled.  
- Core Isolation
- MSFT Driver Block List
- Virtualization Based Security
- Disable hypervisor

## Disable Core Isolation
For details please visit:  
https://support.microsoft.com/en-us/windows/a-driver-can-t-load-on-this-device-8eea34e5-ff4b-16ec-870d-61a4a43b3dd5
  
TODO: Why?

## MSFT Driver Block List & Microsoft Vulnerable Driver Blocklist
KDMapper output: `NTSTATUS (0xC0000428): Windows cannot verify the digital signature for this file`
https://www.thewindowsclub.com/microsoft-vulnerable-driver-blocklist-in-windows
https://community.amd.com/t5/drivers-software/amd-driver-problem/m-p/474646#M144661

ATTENTION: You have to restart your PC afterwards.
TODO: Why?

## Virtualization Based Security
https://www.makeuseof.com/windows-11-disable-vbs/

TODO: Why?

## Disable HyperVisor
Run as admin in a cmd:
```
bcdedit /set hypervisorlaunchtype off
```