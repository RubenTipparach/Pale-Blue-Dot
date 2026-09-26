@echo off
setlocal EnableExtensions DisableDelayedExpansion
REM Double-click to build and run Pale Blue Dot.
REM Builds the "fast" profile by default: optimised, but without link-time
REM optimisation, so a small change relinks in seconds.
REM   run.bat --full    the real release build (slow link): use it for
REM                     performance numbers, recordings and regression checks.
REM Other arguments are passed to pbd-app, e.g. run.bat --tour.

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

set "profile=fast"
set "app_args="
:parse
if "%~1"=="" goto parsed
if /i "%~1"=="--full" (
    set "profile=release"
) else (
    set "app_args=%app_args% %1"
)
shift
goto parse
:parsed

echo Building and running Pale Blue Dot, %profile% profile.
if "%profile%"=="fast" echo Use run.bat --full for the full release build.
echo The first build may take several minutes.
echo Opening the planet explorer. Use --headless 600 for a console smoke run.
echo.

cargo run --profile %profile% --locked -p pbd-app --%app_args%
set "run_exit_code=%errorlevel%"
if not "%run_exit_code%"=="0" echo Build or application failed with exit code %run_exit_code%.

:finish
popd
pause
exit /b %run_exit_code%
