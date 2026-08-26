@echo off
REM thin delegator - real logic in vm-link-probe.mjs (PLAN-045)
node "%~dp0vm-link-probe.mjs"
exit /b %ERRORLEVEL%
