@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\wordweave5.exe" goto missing
start "" "target\release\wordweave5.exe"
exit /b 0
:missing
echo Run build.cmd first. See README.md for setup.
pause
exit /b 1

