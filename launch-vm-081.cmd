@echo off
rem PLAN-081 UAT: VM desktop from 081 worktree (r6: tool card/think block/block gap/param alignment)
rem Double-click to launch. Control experiment for §10-9: if this user-launched
rem instance survives past ~3min while agent-launched ones die at ~90s, the killer
rem is the agent shell lineage; if it dies the same way, §10-9 is confirmed
rem independent of launch method (log: .wt\musk-081\auto-musk\tmp\p081-vm-user.log)
cd /d D:\autostack\.wt\musk-081\auto-musk

rem Reuse backend @17201 if alive
curl -s -m 2 http://127.0.0.1:17201/api/health >nul 2>&1
if errorlevel 1 (
    echo Starting backend musk.exe serve @17201 ...
    start "musk-backend" /min cmd /c "set MUSK_SERVE_PORT=17201&& backend\target\release\musk.exe serve"
    timeout /t 3 >nul
) else (
    echo Backend already running @17201, reusing.
)

echo Starting VM desktop (081 worktree r6) ...
set AUTO_REUSE_BACKEND=1
set AUTO_HTTP_PORT=17201
set AUTO_HTTP_BASE=http://127.0.0.1:17201
set RUST_MIN_STACK=16777216
set AUTOUI_MCP_PORT=9251
set AUTOUI_ACCEPTANCE=1
D:\autostack\auto-lang\target\release\auto.exe run --render vm > D:\autostack\.wt\musk-081\auto-musk\tmp\p081-vm-user.log 2>&1
pause
