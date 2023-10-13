# How to Use Valthrun
To use Valthrun, you will need to follow several steps to set up the required components and run the Valthrun overlay. 
Here is a guide on how to use Valthrun:

## Required Files
Before you can use Valthrun, you must acquire two essential components:

1. **Kernel Driver (`vulthrun-driver.sys`)**  
The kernel driver is the crucial part of Valthrun. 
It supports undetected arbitrary read operations on the Counter-Strike 2 process and prevents other software like VAC from detecting these operations. 
You can obtain the latest release of the kernel driver from the GitHub release page.  

2. **Valthrun Overlay (`controller.exe`)**  
The Valthrun overlay, provided as `controller.exe`, is the user interface for Valthrun.  
It allows you to interact with and control the Valthrun software.

You can download the latest releases of both the kernel driver and the CS2 overlay from the [GitHub release page](https://github.com/Valthrun/Valthrun/releases).
  
## Starting the Overlay
Once you have acquired the necessary files mentioned above, follow these steps to run the Valthrun overlay:

1. **Load the Kernel Driver (`vulthrun-driver.sys`)**  
The kernel driver is the centerpiece of Valthrun. 
It enables undetected arbitrary read operations on the Counter-Strike 2 process and prevents detection by other software, such as VAC.
There are multiple ways to load the kernel driver. 
For detailed instructions on how to load the kernel driver, refer to the documentation [here](010_getting-started/020_driver.md).

2. **Ensure Counter-Strike 2 is Running**  
Before starting the Valthrun overlay, make sure that Counter-Strike 2 (CS2) is running. 
If CS2 is not already running, launch the game, as the controller will not run if CS2 is not running.

3. **Start the Overlay (`controller.exe`)**  
Once the kernel driver has been successfully loaded, and CS2 is up and running, 
you can start the Valthrun overlay by running `controller.exe`. 
If everything is set up correctly, you should see a terminal window.

Here's an example of what the overlay's interface might look like:
![Screenshot of Success](../../_media/screenshot_controller_success.png)

With these steps completed, you are now ready to use Valthrun and take advantage of its gameplay enhancements for Counter-Strike 2.  
Be sure to consult the project's documentation and support resources for more detailed information on its functionality and usage.