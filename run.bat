@echo off
setlocal EnableExtensions DisableDelayedExpansion
REM Double-click to build and run Pale Blue Dot in release mode.
REM Optional arguments are passed to pbd-app, e.g. run.bat --tour.

pushd "%~dp0"
if errorlevel 1 (
    echo Could not open the Pale Blue Dot project directory.
    pause
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo Cargo was not found. Install Rust from https://rustup.rs,
    echo then reopen your terminal and run this launcher again.
    set "run_exit_code=1"
    goto finish
)

echo Building and running Pale Blue Dot in release mode.
echo The first build may take several minutes.
echo Opening the planet explorer. Use --headless 600 for a console smoke run.
echo.

cargo run --release --locked -p pbd-app -- %*
set "run_exit_code=%errorlevel%"
if not "%run_exit_code%"=="0" echo Build or application failed with exit code %run_exit_code%.

:finish
popd
pause
exit /b %run_exit_code%
