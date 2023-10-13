# Path to driver contains non ascii characters or spaces
A common issue with Windows paths are spaces and non ascii characters.  
Non ascii characters are all characters which are nether `0-9`, `a-z` or `A-Z`. 
An example for this would be cyrillic (`Привет`), Chineese or Korean.  

Paths containing these non ascii characters are difficult to deal with and could lead to  
unexpected issues while trying to manually map the driver.  
A recommanded path, containing all required files for Valthrun would be `C:\Valthrun`.  
This path does not contain any non ascii characters, nether contains any spaces.