@echo off
setlocal
REM Double-click to PLAY Pale Blue Dot straight away: launches the last fast
REM build (target\fast\pbd-app.exe) without running cargo, so it starts in
REM seconds. Use build-play.bat instead to rebuild after pulling or changing code.
REM Arguments are passed to the game, e.g. play.bat --tour.

pushd "%~dp0"
if not exist "target\fast\pbd-app.exe" (
    echo No fast build yet. Building it once with build-play.bat...
    call build-play.bat %*
    popd
    exit /b
)
"target\fast\pbd-app.exe" %*
set "code=%errorlevel%"
popd
exit /b %code%
