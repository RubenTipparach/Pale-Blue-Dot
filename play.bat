@echo off
setlocal
REM Double-click to PLAY Pale Blue Dot straight away: launches the last fast
REM build (target\fast\pbd-app.exe) without running cargo, so it starts in
REM seconds. Use build-play.bat instead to rebuild after pulling or changing code.
REM Arguments are passed to the game, e.g. play.bat --tour.
REM
REM Every run's output (and a panic's message and backtrace, if it crashes) is
REM kept in logs\play-last.log; the run before it in logs\play-previous.log.

pushd "%~dp0"
if not exist "target\fast\pbd-app.exe" (
    echo No fast build yet. Building it once with build-play.bat...
    call build-play.bat %*
    popd
    exit /b
)
if not exist logs mkdir logs
if exist logs\play-last.log move /y logs\play-last.log logs\play-previous.log >nul
set RUST_BACKTRACE=1
echo Playing. Output goes to logs\play-last.log
"target\fast\pbd-app.exe" %* > logs\play-last.log 2>&1
set "code=%errorlevel%"
if not "%code%"=="0" (
    echo.
    echo The game exited with code %code%. The end of logs\play-last.log:
    powershell -NoProfile -Command "Get-Content logs\play-last.log -Tail 30"
    pause
)
popd
exit /b %code%
