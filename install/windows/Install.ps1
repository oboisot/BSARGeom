# Install.ps1 — copy BSARGeom into %LOCALAPPDATA% and create shortcuts.
$Source = "."
$Name   = "BSARGeom"
$Dest   = Join-Path $env:LOCALAPPDATA $Name

Write-Host "`n=== Installing $Name ===`n"

# Remove any previous install first.
if (Test-Path $Dest) {
    Write-Host "A version of BSARGeom already exists !"
    Write-Host "  -> Updating/Reinstalling BSARGeom in $Dest"
    Remove-Item $Dest -Recurse -Force
} else {
    Write-Host "Installing BSARGeom in $Dest"
}

try {
    Write-Host -NoNewline "Copying files...     "
    Copy-Item $Source $Dest -Recurse -Force
    Write-Host " ...done"
} catch {
    Write-Host "/!\ Copy error /!\ : $($_.Exception.Message)"
    Pause
    exit
}

Write-Host -NoNewline "Creating shortcuts..."
$Shell     = New-Object -ComObject WScript.Shell
$ExePath   = Join-Path $Dest "bsargeom.exe"
$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$Desktop   = [Environment]::GetFolderPath("Desktop")

# BSARGeom — Start Menu + Desktop (icon comes from the exe's embedded resource).
if (Test-Path $ExePath) {
    foreach ($dir in @($StartMenu, $Desktop)) {
        $Shortcut = $Shell.CreateShortcut((Join-Path $dir "BSARGeom.lnk"))
        $Shortcut.TargetPath       = $ExePath
        $Shortcut.WorkingDirectory = $Dest
        $Shortcut.IconLocation     = "$ExePath,0"
        $Shortcut.Save()
    }
}

# Uninstall/Reinstall entry — Start Menu.
$BatPath = Join-Path $Dest "InstallOrUninstall.bat"
if (Test-Path $BatPath) {
    $Shortcut = $Shell.CreateShortcut((Join-Path $StartMenu "Uninstall BSARGeom.lnk"))
    $Shortcut.TargetPath       = $BatPath
    $Shortcut.WorkingDirectory = $Dest
    $Shortcut.IconLocation     = "$ExePath,0"
    $Shortcut.Save()
}
Write-Host " ...done"

Write-Host "`n=== BSARGeom installation complete ===`n"
Write-Host "Thank you for installing BSARGeom !"
Write-Host "  |-> launch it from the Start Menu or the Desktop shortcut"
Write-Host "  |-> search for 'Uninstall BSARGeom' in the Windows search bar to Reinstall/Uninstall it"
