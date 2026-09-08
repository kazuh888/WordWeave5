@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 goto missing
echo Running Rust tests. First build downloads dependencies and can take time.
cargo test --all-targets --locked > build.log 2>&1
if errorlevel 1 goto failed
echo Building release executable...
cargo build --release --locked --bin wordweave5 >> build.log 2>&1
if errorlevel 1 goto failed
echo.
echo Build succeeded: target\release\wordweave5.exe
echo Keep Cargo.lock for reproducible subsequent builds.
echo Use run.cmd to start the application.
pause
exit /b 0
:missing
echo Cargo was not found. Read README.md to install Rust and MSVC C++ Build Tools.
pause
exit /b 1
:failed
type build.log
echo.
echo Tests or build failed. Full output is in build.log.
echo No release executable is claimed to be ready.
pause
exit /b 1

