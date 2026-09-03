@echo off
REM PLAN-059 S1: musk 开发栈一键编排（后端+VM+可选 Vue,MCP 换口 9277）。
REM 本体逻辑在 dev-stack.mjs。用法:
REM   scripts\dev-stack.cmd [--web] [--fresh-backend]
node "%~dp0dev-stack.mjs" %*
