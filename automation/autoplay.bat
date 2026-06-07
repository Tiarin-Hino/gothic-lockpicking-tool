@echo off
REM Double-click to run the auto-player. It reads the key plan from your
REM clipboard (click "Copy key plan" in the web tool first), counts down,
REM then presses the keys. Optional: pass a countdown, e.g.  autoplay.bat 8
python "%~dp0autoplay.py" %*
echo.
pause
