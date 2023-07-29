$cleanName = "valthrun-driver"

Write-Host "Stopping & deleting driver"
sc.exe stop $cleanName
sc.exe delete $cleanName