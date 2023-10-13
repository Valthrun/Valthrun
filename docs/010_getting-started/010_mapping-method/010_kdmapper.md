# Using KDMapper to load the Valthrun driver

[KDMapper](https://github.com/TheCruZ/kdmapper) is the recommended method for manually loading the Valthrun driver into memory.  
This simple tool exploits the `iqvw64e.sys` Intel driver to map non-signed drivers, allowing you to load any driver, including the Valthrun driver.  
To map the Valthrun driver with KDMapper, follow these steps:

1. **Obtain KDMapper**  
   Before you can use KDMapper, you need a working executable of KDMapper.  
   The official KDMapper repository does not provide a download link, so you have two options:
   - **Compile It Yourself (Recommended)**  
   For enhanced security and trustworthiness, it is recommended to compile KDMapper yourself.  
   You can find detailed instructions on how to compile KDMapper in the official [KDMapper repository](https://github.com/TheCruZ/kdmapper).  
   Compiling it yourself ensures that you have control over the source code and can verify its integrity.  
   
   - **Download a Precompiled Version**  
   You can also find a precompiled version of KDMapper [here](https://github.com/valthrunner/Valthrun/releases/latest).  
   Please note that this precompiled version is **not offered by Valtrun** but is compiled and uploaded by the user @valthrunner.  
   When downloading precompiled software, exercise caution and ensure that you trust the source.

2. **Open a Command Line as Administrator**  
   To use KDMapper successfully, open a command line with administrator privileges.  
   You can do this by right-clicking on the Command Prompt or PowerShell and selecting "Run as administrator.".  

3. **Navigate to the Valthrun directory**  
   Before proceeding with the Valthrun driver loading process using KDMapper, make sure you are in the correct directory where kdmapper.exe and valthrun-driver.sys are located.  
   Use the cd command to navigate to the directory where these files are located, ensuring that KDMapper can access the required components for the driver loading procedure.

4. **Load `valthrun-driver.sys` with KDMapper**  
To load the Valthrun driver into memory, execute the following command in your command prompt or PowerShell:  
```bash
kdmapper.exe valthrun-driver.sys
```
  
If the operation is successful, the output should resemble the following:  
```
[<] Loading vulnerable driver, Name: SaBVbLkOxDxwTNNOsSPnmMW
[+] NtLoadDriver Status 0x0
[-] Can't find pattern
[+] PiDDBLock found with second pattern
[+] PiDDBLock Ptr 0xfffff80130674912
[+] PiDDBCacheTable Ptr 0xfffff80130568508
[+] PiDDBLock Locked
[+] Found Table Entry = 0xFFFFAC0ED06F4C40
[+] PiDDBCacheTable Cleaned
[+] g_KernelHashBucketList Found 0xFFFFF8013222C088
[+] g_HashCacheLock Locked
[!] g_KernelHashBucketList looks empty!
[+] MmUnloadedDrivers Cleaned: SaBVbLkOxDxwTNNOsSPnmMW
[+] Image base has been allocated at 0xFFFFD0876A42E000
[+] Skipped 0x1000 bytes of PE Header
[<] Calling DriverEntry 0xFFFFD0876A433B10
[+] Callback example called
[+] DriverEntry returned 0x0
[<] Unloading vulnerable driver
[+] NtUnloadDriver Status 0x0
[+] Vul driver data destroyed before unlink
[+] success
```
  
Ensure that the output contains the line: `[+] DriverEntry returned 0x0`.   
If this line is present, it indicates a successful loading of the Valthrun driver.   
However, if this line is **not found in the output**, it suggests that the **mapping process failed**.   
In such cases, please refer to the troubleshooting section for guidance on resolving the issue.

### Advantages
Using KDMapper is a quite straigt forward process.  
KDMapper is quite reliable and does not require a lot of trail and error.  
  
### Disadvantages
There are two main disadvantages to KDMapper.  