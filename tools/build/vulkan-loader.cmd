@echo off
setlocal
set TAG=vulkan-sdk-1.4.363.0
set ROOT=%~dp0..\..
set SRC=%ROOT%\work\vulkan-loader\src
set OUT=%ROOT%\work\vulkan-loader\bin
for /f "usebackq delims=" %%i in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set VS_PATH=%%i
if not defined VS_PATH (echo no Visual Studio C++ tools found by vswhere & exit /b 1)
call "%VS_PATH%\VC\Auxiliary\Build\vcvars64.bat" >nul || exit /b 1
if not exist "%SRC%\.git" git clone --depth 1 --branch %TAG% https://github.com/KhronosGroup/Vulkan-Loader.git "%SRC%" || exit /b 1
cmake -S "%SRC%" -B "%SRC%\build" -G Ninja -D CMAKE_BUILD_TYPE=Release -D UPDATE_DEPS=ON -D BUILD_TESTS=OFF || exit /b 1
cmake --build "%SRC%\build" --config Release || exit /b 1
if not exist "%OUT%" mkdir "%OUT%"
copy /y "%SRC%\build\loader\vulkan-1.dll" "%OUT%\vulkan-1.dll" >nul || exit /b 1
copy /y "%SRC%\LICENSE.txt" "%OUT%\LICENSE-vulkan-loader.txt" >nul || exit /b 1
echo vulkan-1.dll from %TAG% in %OUT%
