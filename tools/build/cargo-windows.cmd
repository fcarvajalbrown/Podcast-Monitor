@echo off
setlocal
call "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
if not defined CMAKE_GENERATOR set CMAKE_GENERATOR=Ninja
if not defined CARGO_TARGET_DIR set CARGO_TARGET_DIR=%USERPROFILE%\.pmt
if not defined VULKAN_SDK set VULKAN_SDK=C:\VulkanSDK\1.4.363.0
cd /d "%~dp0..\.."
cargo %*
