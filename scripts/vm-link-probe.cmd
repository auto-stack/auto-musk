@echo off
REM vm-link-probe.cmd — PLAN-045: auto-musk VM 链接门禁一键探针
REM
REM 对 musk 全量前端(src/front/app.at)做 VM 目标的 parse+codegen+link
REM headless 验证(auto-lang plan442_musk_probe_tests,#[ignore] 手动门)。
REM
REM 用法(在 auto-musk 任意位置): scripts\vm-link-probe.cmd
REM 前置: sibling 检出 ..\..\auto-lang 存在且可 cargo 构建。
REM
REM 勘误(PLAN-045): 探针模块门控是 feature "ui-iced"(lib.rs plan442_musk_probe_tests
REM 的 cfg),auto-lang 442 计划文档头注写的 "--features ui-interpreter" 已过时——
REM 该 feature 集编译失败(unresolved iced_adapter 等)。
REM 环境注: 直接运行测试 exe 需 RUST_MIN_STACK=16777216(经 cargo 运行时由
REM auto-lang 仓 .cargo/config.toml [env] 自动提供)。

setlocal
set "PROBE_ROOT=%~dp0.."
pushd "%PROBE_ROOT%\..\auto-lang" || (echo [vm-link-probe] ..\auto-lang not found & exit /b 2)

cargo test -p auto-lang --lib --features ui-iced musk_probe -- --ignored --nocapture
set "RC=%ERRORLEVEL%"
popd
if "%RC%"=="0" (
  echo [vm-link-probe] PASS — musk frontend links on VM target
) else (
  echo [vm-link-probe] FAIL — see [HANDLER-CODEGEN] / [CODEGEN] lines above
)
exit /b %RC%
