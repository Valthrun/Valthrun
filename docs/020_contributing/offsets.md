# Offsets
## What are offsets
In computer science, an offset within an array or other data structure object is an integer indicating the distance (displacement) between the beginning of the object and a given element or point, presumably within the same object [^4].
Valthrun heavily uses offsets to retrieve data from different CS2 structures to enhance your gameplay experience.

As adding, reordering or removing [variables](https://en.wikipedia.org/wiki/Variable_(computer_science)) will cause these offsets to change, each CS2 update will result in slightly changed offsets.
Therefore the Valthrun controller needs to be updated as well [^3].

## Use of offsets and the CS2 schema system
CS2 has a convenient system, to retrieve all offsets for variables which are shared between the server and the client.  
These variables previously known as `netvars` are now called schema variables.

Most information required for gameplay enhancements can be acquired by retrieving these shared variables.
Therefore most likely updating these offsets is sufficient.

In some circumstances, local client data needs to be accessed. Examples of such data are players' bone states and the current crosshair entity ID [^1]. 
To resolve these offsets, Valthrun mostly relies on  [pattern scanning](https://www.unknowncheats.me/forum/general-programming-and-reversing/133228-implement-pattern-scanning-obtain-offsets-dynamically.html).
Although pattern scanning is a quite reliable method for resolving offsets across multiple versions, patterns can break and have to be updated. Creating a pattern might also just not be feasible [^2].

Because of this some offsets are [hard coded](https://en.wikipedia.org/wiki/Hard_coding).  
Most of these hard-coded offsets are unlikely to change but if they change they have to be updated manually.

As a rule of thumb:  
Hard-coded offsets are unlikely to change with updates.
Offsets that are subject to change with updates are either retrieved by the schema system or by pattern scanning.

## Updating schema-based offsets
Most likely to change offsets can be retrieved by using the CS2 schema system.
Based on the information this system provides we can automatically generate all class and function definitions.
The source, containing all these definitions can be found [here](https://github.com/WolverinDEV/Valthrun/blob/master/cs2-schema/generated/cs2_schema.json). Updating this file is quite easy:

1. Dump the current CS2 schema  
  Dump the current schema to "cs2_schema.json".
  
  Attention:  
  This requires the game to be running and the kernel driver to be loaded!
```ps
.\controller.exe dump-schema cs2_schema.json
```  
  
2. Update the `cs2_schema.json`  
Replace the `cs2_schema.json` located at `cs2-schema/generated/cs2_schema.json` with the newly dumped schema.
  
3. Recompile the controller  
Recompile the controller as described [here](https://github.com/WolverinDEV/Valthrun/blob/master/BUILD.MD#2-overlay).
  
Most likely you'll be good to go and ready for the next update of CS2.  
If the Valthrun still behaves badly or generates an error, it might be an indication that some of the hard-coded offsets have changed.
If that's the case you either track down the issue (by analyzing and debugging the source code) and resolve the new offset but digging into CS2 memory or just wait for somebody else to do the work for you :)  
(PS: You may buy him a coffee tough :P)
  
[^1]: Please note that the crosshair entity id was a shared variable in CSGO but isn't in CS2!
[^2]: or the developer was just too lazy at that moment to implement a pattern...
[^3]: The kernel driver only provides basic memory read functionality and does not need to know anything about the target application.
Therefore the kernel driver will *never* be affected by any CS2 update.
[^4]: Offset (computer_science) ([Wikipedia](https://en.wikipedia.org/wiki/Offset_(computer_science)))
