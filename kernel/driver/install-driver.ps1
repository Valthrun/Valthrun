$cleanName = "valthrun-driver"

Write-Host "Stopping & deleting driver"
sc.exe stop $cleanName
sc.exe delete $cleanName

Write-Host "Installing & starting driver ($pwd\..\target\x86_64-pc-windows-msvc\debug\$cleanName.sys)"
sc.exe create $cleanName type= kernel start= demand error= normal binPath= $pwd\..\target\x86_64-pc-windows-msvc\debug\$cleanName.sys DisplayName= $cleanName
sc.exe start $cleanName