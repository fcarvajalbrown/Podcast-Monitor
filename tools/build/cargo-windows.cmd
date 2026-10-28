@echo off
setlocal
for /f "usebackq delims=" %%i in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set VS_PATH=%%i
if not defined VS_PATH (echo no Visual Studio C++ tools found by vswhere & exit /b 1)
set "PATH=%PATH%;%ProgramFiles(x86)%\Microsoft Visual Studio\Installer"
call "%VS_PATH%\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
if not defined CMAKE_GENERATOR set CMAKE_GENERATOR=Ninja
if not defined CARGO_TARGET_DIR set CARGO_TARGET_DIR=%USERPROFILE%\.pmt
if not defined VULKAN_SDK set VULKAN_SDK=C:\VulkanSDK\1.4.363.0
cd /d "%~dp0..\.."
cargo %*
