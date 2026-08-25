@echo off
REM PLAN-041 T12: gen (Auto/vue) dev server - production track
REM Rollback: set MUSK_WEB_DIST=web/dist before "musk serve" for hw track
cd /d "%~dp0gen\front\vue"
echo Starting gen (Auto/vue) dev server on :3334...
call pnpm dev
