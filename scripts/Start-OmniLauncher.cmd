@echo off
setlocal
set "PATH=C:\Program Files\nodejs;%USERPROFILE%\.cargo\bin;%PATH%"
cd /d "%~dp0"
if "%~1"=="--dev" (
    npm.cmd run tauri dev
) else if exist "%~dp0src-tauri\target\debug\omnilauncher.exe" (
    start "" "%~dp0src-tauri\target\debug\omnilauncher.exe"
) else (
    npm.cmd run tauri dev
)
