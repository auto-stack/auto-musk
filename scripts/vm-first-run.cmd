@echo off
REM PLAN-047 T2: 一键委托形式（cmd batch 相对跳转在本机不可靠,045-T9 实测;
REM 本体逻辑在 vm-first-run.mjs）。用法:
REM   scripts\vm-first-run.cmd [--keep] [--observe-ms N]
node "%~dp0vm-first-run.mjs" %*
