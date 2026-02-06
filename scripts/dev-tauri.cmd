@echo off
setlocal EnableDelayedExpansion

set "PF86=%ProgramFiles(x86)%"
if not defined PF86 set "PF86=%ProgramFiles%"

set "VCVARS=!PF86!\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if not exist "!VCVARS!" (
  echo [dev-tauri] vcvars64.bat not found: !VCVARS!
  echo [dev-tauri] Install Visual Studio Build Tools with C++ workload.
  exit /b 1
)

for /f "delims=" %%d in ('dir /b /ad "!PF86!\Windows Kits\10\Lib" ^| sort /r') do (
  set "SDKVER=%%d"
  goto foundsdk
)

:foundsdk
if not defined SDKVER (
  echo [dev-tauri] Windows SDK not found under !PF86!\Windows Kits\10\Lib
  exit /b 1
)

call "!VCVARS!"
set "WindowsSdkDir=!PF86!\Windows Kits\10\"
set "WindowsSDKVersion=!SDKVER!\"
set "LIB=!LIB!;!PF86!\Windows Kits\10\Lib\!SDKVER!\um\x64"
set "INCLUDE=!INCLUDE!;!PF86!\Windows Kits\10\Include\!SDKVER!\um;!PF86!\Windows Kits\10\Include\!SDKVER!\shared;!PF86!\Windows Kits\10\Include\!SDKVER!\ucrt"
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

cd /d "%~dp0.."
npm run tauri dev
