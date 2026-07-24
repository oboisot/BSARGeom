@echo off
title BSARGeom Installer/Uninstaller

:MENU
cls
echo ===================================================
echo   Installer / Uninstaller for BSARGeom Application
echo ===================================================
echo.
echo 1. (Re-)Install
echo 2. Uninstall
echo 3. Exit
echo.
set /p choice="Please choose an option (1-3) : "

if "%choice%"=="1" goto INSTALL
if "%choice%"=="2" goto UNINSTALL
if "%choice%"=="3" exit

:INSTALL
echo (Re-)Installing BSARGeom...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install.ps1"
pause
exit

:UNINSTALL
echo Uninstalling BSARGeom...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Uninstall.ps1"
pause
exit
