# Uninstall.ps1 — remove BSARGeom shortcuts and its install folder.
$Name = "BSARGeom"
$Dest = Join-Path $env:LOCALAPPDATA $Name

Write-Host "`n=== Uninstalling $Name ===`n"

Write-Host -NoNewline "Removing shortcuts..."
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$Desktop   = [Environment]::GetFolderPath("Desktop")
$Links = @(
    (Join-Path $StartMenu "BSARGeom.lnk"),
    (Join-Path $StartMenu "Uninstall BSARGeom.lnk"),
    (Join-Path $Desktop   "BSARGeom.lnk")
)
foreach ($link in $Links) {
    if (Test-Path $link) { Remove-Item $link -Force }
}
Write-Host " ...done"

# Remove the install folder via a detached batch so this script isn't running
# from inside the folder it deletes.
Write-Host -NoNewline "Removing files...    "
$TempBatch = [System.IO.Path]::Combine($env:TEMP, "DeleteBSARGeom.bat")
$MaxRetries = 10
$BatchContent = "@echo off`r`n" +
"set Folder=""$Dest""`r`n" +
"set /a Counter=0`r`n" +
":TryDelete`r`n" +
"rmdir /S /Q %Folder% 2>nul`r`n" +
"if exist %Folder% (`r`n" +
"    set /a Counter+=1`r`n" +
"    if %Counter% GEQ $MaxRetries (`r`n" +
"        echo Impossible to remove %Folder% folder after $MaxRetries attempts.`r`n" +
"        goto End`r`n" +
"    )`r`n" +
"    timeout /t 1 /nobreak >nul`r`n" +
"    goto TryDelete`r`n" +
")`r`n" +
":End`r`n" +
"del ""%~f0"""

Set-Content -Path $TempBatch -Value $BatchContent -Encoding ASCII
Start-Process -FilePath $TempBatch -WindowStyle Hidden
Write-Host " ..done"

Write-Host "`n=== BSARGeom uninstallation complete ===`n"
