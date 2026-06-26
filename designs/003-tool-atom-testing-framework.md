# Design 003:AI 原子操作测试框架(Tool Atom Testing)

> **Status**: Design Draft — 待评审
> **动机**: auto-forge 的教训——整个 workflow 每一步都是 AI 决策,任何一步幻觉(调错工具/参数错/改错地方)就断裂。需要像测代码一样**自动测 AI 的原子操作**,给每个工具的各种情形建用例,跑完自动恢复状态。
> **范围**: 以 musk 的 9 个工具为首要对象,设计可扩展到 skill/agent-loop 的通用测试框架。

---

## 1. 问题诊断

### 1.1 auto-forge 为什么跑不通
- 工作流每一步 = AI 决策(LLM 选工具 + 生参数 + 解读结果)
- 任何一步幻觉 → 链条断裂,且**无护栏**:没有断言"这步该做什么",错了静默继续
- 人工排查极慢:不知道是哪步、哪个工具、参数还是结果解读错了

### 1.2 现状(musk tools.rs)
- 有 22 个 `#[tokio::test]`,但:
  - **无沙箱**:副作用工具(write_file/edit_file)直接写 `temp_dir` 硬编码路径,并行测试互相污染
  - **手动清理**:`let _ = remove_file(...)` 散落各处,漏了就残留
  - **只测 happy path**:大部分是"写进去读出来对不对",缺**边界/错误/幻觉情形**(空文件、不唯一匹配、超大文件、路径穿越…)
  - **用例结构散**:每个 test 自行造数据,没有统一的"fixture + 断言 + 回滚"骨架

### 1.3 目标
一套框架,让**每个工具的每种调用情形**都有:
1. **隔离的临时环境**(跑完自动消失,不污染)
2. **声明式用例**(给定 fixture → 调工具 → 断言结果)
3. **覆盖正常/边界/错误三类情形**
4. **可测 AI 调用**(不只是 `execute()` 直调,还能"模拟 LLM 传错参数,工具该拒绝")

---

## 2. 核心设计

### 2.1 沙箱:per-test 临时工作目录(根治副作用)

每个测试跑在一个**唯一的临时目录**里,工具的所有文件操作都被限制在其中。跑完目录自动删除。

```rust
/// 一个隔离的临时工作区。工具在这里读写,跑完自动清理。
/// 是测试框架的基础——副作用工具(edit_file/write_file/run_command)
/// 必须在 Sandbox 内运行,否则会污染真实文件系统。
struct Sandbox {
    dir: TempDir,        // tempfile::TempDir,drop 时自动删除
    // 工具按相对路径操作时,CWD 就是这个 dir
}
```

**关键**:现在的工具用**相对路径**(如 `read_file("Cargo.toml")`)或绝对路径。要让它们在沙箱里跑,有两条路:

| 方案 | 做法 | 优缺点 |
|---|---|---|
| **A: chdir** | 测试开始 `env::set_current_dir(sandbox)`,工具的相对路径自然落到沙箱 | 简单,但 `set_current_dir` 在并行测试里不安全(全局状态) |
| **B: 显式 base 参数** | 给工具加一个 `base_dir`(或让工具从 thread-local 读 CWD) | 更干净,但要改工具签名 |
| **C(推荐): 串行 + chdir** | 用 `#[serial_test::serial]` 标记有副作用的测试,保证串行,然后 chdir 到沙箱 | 不改工具代码,代价是这些测试不能并行(可接受:它们慢且有 IO) |

> **推荐 C**:副作用工具测试本来就该串行(磁盘 IO),用 `serial_test` crate 保证,加 chdir 到 TempDir。读类工具(read_file/glob/search)无副作用,可并行。这样工具代码零改动。

### 2.2 用例结构:given/when/then 声明式

```rust
/// 一个工具测试用例。
struct ToolCase {
    name: &'static str,
    /// 类别:normal / boundary / error(决定覆盖率统计)
    category: CaseCategory,
    /// 在沙箱里准备的初始文件(fixture)
    fixtures: Vec<Fixture>,
    /// 调用哪个工具 + 什么参数
    call: (&'static str, serde_json::Value),
    /// 期望:返回 Ok(匹配内容) 还是 Err(匹配错误类型)
    expect: Expect,
}

enum Fixture {
    File { path: &'static str, content: &'static str },
    Dir { path: &'static str },
}

enum Expect {
    Ok { contains: &'static str },        // 结果包含某串
    OkExact(&'static str),                 // 结果精确等于
    OkMatches(&'static str),               // 正则匹配
    Err { kind: ErrorKind },               // 该报某类错
    FileUnchanged { path: &'static str },  // 副作用检查:文件内容没变
}
```

好处:用例是**数据**,不是散落的代码。一眼看清"这个工具测了哪些情形",覆盖率统计也容易。

### 2.3 覆盖矩阵:每个工具的三类情形

以 `edit_file`(替换文件中唯一字符串)为例,设计**完整**的用例矩阵:

| 情形 | 类别 | fixture | 调用 | 期望 |
|---|---|---|---|---|
| 替换单次匹配 | normal | `a.txt`="hello world" | edit_file(a.txt, hello→hi) | Ok,"hi world",文件已改 |
| 多次匹配应拒绝 | error | `a.txt`="x x x" | edit_file(a.txt, x→y) | Err(NotUnique) |
| 无匹配 | error | `a.txt`="abc" | edit_file(a.txt, zzz→y) | Err(NotFound) |
| 空文件 | boundary | `a.txt`="" | edit_file(a.txt, a→b) | Err(NotFound) |
| 多行替换 | normal | 多行文件 | edit_file(跨行串→新串) | Ok,验证多行内容 |
| Unicode | boundary | `a.txt`="你好 世界" | edit_file(你好→您好) | Ok,"您好 世界" |
| 路径不存在 | error | (无) | edit_file(no.txt, a→b) | Err(Io) |
| 缺参数 | error | (无) | edit_file({}) | Err(Args) |
| old==new | boundary | `a.txt`="x" | edit_file(x→x) | Ok 或 Err(明确语义) |

**每个工具都照此建表**。这就是"用例驱动"——先穷举情形,再写代码实现。

### 2.4 完整工具清单与分类

| 工具 | 副作用 | 需沙箱? | 测试要点 |
|---|---|---|---|
| read_file | 否 | 否(可并行) | 读已有/不存在/缺参数/二进制 |
| write_file | 是(创建/覆盖) | 是 | 写新文件/覆盖/建父目录/空内容 |
| edit_file | 是(改) | 是 | 唯一匹配/多匹配报错/无匹配/空文件/Unicode |
| batch_replace | 是(改多处) | 是 | 多条替换/部分失败/空列表 |
| list_dir | 否 | 否(可并行) | 列空目录/嵌套/文件大小 |
| glob | 否 | 否 | 通配符/无匹配/递归 |
| search | 否 | 否 | 正则/无匹配/多文件 |
| list_symbols | 否 | 否 | 各语言符号/空文件 |
| run_command | 是(执行任意命令) | **是,且需限制** | echo/失败命令/超时;**安全:禁止危险命令** |

### 2.5 run_command 的特殊处理(安全)

`run_command` 能执行任意 shell,测试它有**安全风险**。框架要:
- 沙箱内**只允许无害命令**(`echo`/`dir`/`type`/`cat`)
- 用一个**命令白名单**或**沙箱内 shell 限制**(不联网、不写沙箱外)
- 测试断言:stdout 合并 stderr、退出码、超时

---

## 3. 框架骨架(代码草图)

```rust
// crates/musk/tests/tool_atoms.rs (或 src/tools/tests.rs)

use musk_tool_test::{Sandbox, Fixture, run_cases};

mod edit_file_cases {
    use super::*;
    fn cases() -> Vec<ToolCase> {
        vec![
            ToolCase {
                name: "replace_unique_match",
                category: CaseCategory::Normal,
                fixtures: vec![Fixture::file("a.txt", "hello world")],
                call: ("edit_file", json!({"path":"a.txt","old_string":"hello","new_string":"hi"})),
                expect: Expect::Ok { contains: "hi world" },
            },
            ToolCase {
                name: "multiple_matches_rejected",
                category: CaseCategory::Error,
                fixtures: vec![Fixture::file("a.txt", "x x x")],
                call: ("edit_file", json!({"path":"a.txt","old_string":"x","new_string":"y"})),
                expect: Expect::Err { kind: ErrorKind::NotUnique },
            },
            // ... 边界用例
        ]
    }

    #[tokio::test]
    #[serial]  // 副作用 → 串行
    async fn run_all() {
        run_cases(cases()).await;  // 每个用例:建沙箱→造fixture→调工具→断言→drop清理
    }
}
```

### 3.1 run_case 的内部流程
```
1. Sandbox::new()              → 唯一 TempDir,chdir 进去
2. 应用 fixtures               → 写入初始文件/目录
3. 取工具实例(从全局 registry)
4. tool.execute(args).await    → 拿结果
5. 按 Expect 断言              → Ok 含串?Err 类型对?
6. (副作用工具)验证文件终态     → 文件改对了没?没误伤别的?
7. drop Sandbox                → TempDir 自动删除,状态恢复
```

**状态恢复 = TempDir 的 drop**。这就是"跑完自动恢复"——零手动清理。

---

## 4. 扩展:从工具原子到 AI 操作原子

框架的可扩展性是重点。测完工具的 `execute()`,还要测**更高层的 AI 原子**:

| 层级 | 原子 | 怎么测 |
|---|---|---|
| **L0 工具直调** | `tool.execute(args)` | 本设计(确定性,断言精确结果) |
| **L1 工具调用规约** | "LLM 该传什么参数" | 给场景描述,断言参数 schema 校验通过/失败(假 LLM) |
| **L2 单轮 agent** | 一次 `agent.run(task)` | Mock LLM(返回固定 tool_call),断言工具被正确调用+结果喂回(已部分存在于 agent.rs 测试) |
| **L3 多轮 workflow** | 一个 workflow step | Mock LLM + 断言 step 完成/跳过/重试(workflow.rs 已有雏形) |

本设计先做 **L0**(最基础、确定性、可完全自动化)。L1-L3 用 MockClient(已有)+ 扩展断言。

---

## 5. 与现有测试的关系

- **保留** agent.rs / workflow.rs 的 MockClient 测试(L2/L3)
- **迁移** tools.rs 现有 22 个散乱 test → 框架的用例矩阵(更结构化、加沙箱、补边界)
- **新增** 每个工具的错误/边界用例(现在缺)
- 测试目录:`crates/musk/tests/tool_atoms.rs`(集成测试)或 `src/tools/tests.rs`(单元)

---

## 6. 依赖与风险

| 项 | 说明 |
|---|---|
| `tempfile` | 已有(dev-dep),提供 TempDir 自动清理 |
| `serial_test` | **新增**,~20 行 crate,保证副作用测试串行(chdir 全局状态) |
| **chdir 全局状态** | 串行规避;若未来工具支持 base_dir 参数可去掉 |
| **Windows 路径** | 工具已有 `std::path`,测试用正斜杠(Windows 兼容) |
| **run_command 安全** | 测试用例只用 `echo`/`type` 等无害命令,不测真实危险操作 |

---

## 7. 实施步骤(确认设计后)

1. 加 `serial_test` dev-dep;写 `Sandbox` + `Fixture` + `run_cases` 框架骨架
2. 迁移 + 扩充 `edit_file` 用例(作为第一个完整样例,含错误/边界)
3. 照矩阵补齐其余 8 个工具的用例
4. 特殊处理 `run_command`(白名单 + 超时)
5. (可选)覆盖率统计脚本:列出每个工具测了哪些 category

---

## 8. 待确认的设计点

1. **chdir vs base_dir 参数**:不改工具(用 serial+chdir)还是改工具签名(加 base_dir)?推荐前者(零侵入),但 base_dir 更"正确"。
2. **用例放哪**:`tests/tool_atoms.rs`(集成)还是 `src/tools/tests.rs`(单元)?推荐集成测试(独立二进制,不拖慢 lib 编译)。
3. **run_command 测多深**:只测"echo 成功/失败命令报错",还是也测超时/管道?推荐先只测基本。
4. **是否引入假 LLM 测 L1**:本设计聚焦 L0,L1 留后续。确认?
