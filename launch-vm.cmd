@echo off
rem PLAN-080 UAT: VM desktop version one-click launch (backend + frontend + AI daemon reuse)
rem §10-9 note: self-launched process (not going through the agent shell) — lifecycle is user-owned
cd /d D:\autostack\.wt\musk-080\auto-musk

rem ① Start backend (if 17201 is not occupied)
curl -s -m 2 http://127.0.0.1:17201/api/health >nul 2>&1
if errorlevel 1 (
    echo Starting backend musk.exe serve @17201 ...
    start "musk-backend" /min cmd /c "set MUSK_SERVE_PORT=17201&& backend\target\release\musk.exe serve"
    timeout /t 3 >nul
) else (
    echo Backend already running @17201, reusing.
)

rem ② Start VM desktop frontend (auto-lang 7d6888076+ binary, reuse backend)
echo Starting VM desktop ...
set AUTO_REUSE_BACKEND=1
set AUTO_HTTP_PORT=17201
set AUTO_HTTP_BASE=http://127.0.0.1:17201
set RUST_MIN_STACK=16777216
D:\autostack\.wt\musk-080\auto-lang\target\release\auto.exe run --render vm
pause
