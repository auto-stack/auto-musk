//! Tool atom tests — each tool's `execute()` across a full matrix of
//! normal/boundary/error cases, in auto-cleaning sandboxes.
//!
//! (Design 003 — Tool Atom Testing Framework.)
//!
//! Side-effect cases use `#[serial]` (the `serial_test` crate) because they
//! `chdir` into a sandbox — a global state that can't be shared across
//! parallel tests. Read-only tools (read_file/glob/search/list_dir) don't
//! touch the CWD so they could run in parallel, but for simplicity every
//! case runs serially here.

use musk::tool_test::{run_cases, run_case, CaseCategory, Expect, Fixture, ToolCase};
use musk::tools::*;
use auto_ai_agent::Tool; // for direct .execute() calls in run_command tests

/// Produce a fresh tool instance by name. Each case gets its own (tools are
/// cheap zero-field unit structs).
fn make_tool(name: &str) -> Option<Box<dyn auto_ai_agent::Tool>> {
    match name {
        "read_file" => Some(Box::new(ReadFile::new())),
        "write_file" => Some(Box::new(WriteFile::new())),
        "edit_file" => Some(Box::new(EditFile::new())),
        "search" => Some(Box::new(Search::new())),
        "list_dir" => Some(Box::new(ListDir::new())),
        "list_symbols" => Some(Box::new(ListSymbols::new())),
        "glob" => Some(Box::new(Glob::new())),
        "run_command" => Some(Box::new(RunCommand::new())),
        _ => None,
    }
}

use serde_json::json;

// ════════════════════════════════════════════════════════════════════════════
// edit_file — the canonical complete matrix (normal / boundary / error)
// ════════════════════════════════════════════════════════════════════════════

fn edit_file_cases() -> Vec<ToolCase> {
    
    vec![
        // ── Normal ────────────────────────────────────────────────────────────
        ToolCase {
            name: "replace_unique_match",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "hello world")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"hello","new_string":"hi"})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "hi world" },
        },
        ToolCase {
            name: "replace_multiline",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "line one\nline two\nline three")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"line two","new_string":"LINE TWO"})),
            expect: Expect::OkFileEquals {
                path: "a.txt",
                content: "line one\nLINE TWO\nline three",
            },
        },
        ToolCase {
            name: "replace_unicode",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "你好 世界")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"你好","new_string":"您好"})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "您好 世界" },
        },
        // ── Boundary ──────────────────────────────────────────────────────────
        ToolCase {
            name: "empty_file_no_match",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::file("a.txt", "")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"x","new_string":"y"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "replace_entire_content",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::file("a.txt", "xxx")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"xxx","new_string":"yyy"})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "yyy" },
        },
        // ── Error ─────────────────────────────────────────────────────────────
        ToolCase {
            name: "multiple_matches_rejected",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "x x x")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"x","new_string":"y"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "no_match",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "abc")],
            call: ("edit_file", json!({"path":"a.txt","old_string":"zzz","new_string":"y"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "path_not_found",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("edit_file", json!({"path":"nope.txt","old_string":"a","new_string":"b"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "missing_path_arg",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("edit_file", json!({"old_string":"a","new_string":"b"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "missing_old_string_arg",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "abc")],
            call: ("edit_file", json!({"path":"a.txt","new_string":"b"})),
            expect: Expect::Err,
        },
    ]
}

#[tokio::test]
#[serial_test::serial]
async fn edit_file_matrix() {
    run_cases(&edit_file_cases(), make_tool).await;
}

// Run each case also as an individual named test (better failure attribution).
mod edit_file_individual {
    use super::*;

    #[tokio::test]
    #[serial_test::serial]
    async fn replace_unique_match() {
        run_case(&edit_file_cases()[0], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn replace_multiline() {
        run_case(&edit_file_cases()[1], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn replace_unicode() {
        run_case(&edit_file_cases()[2], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn empty_file_no_match() {
        run_case(&edit_file_cases()[3], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn replace_entire_content() {
        run_case(&edit_file_cases()[4], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn multiple_matches_rejected() {
        run_case(&edit_file_cases()[5], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn no_match() {
        run_case(&edit_file_cases()[6], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn path_not_found() {
        run_case(&edit_file_cases()[7], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn missing_path_arg() {
        run_case(&edit_file_cases()[8], make_tool).await;
    }
    #[tokio::test]
    #[serial_test::serial]
    async fn missing_old_string_arg() {
        run_case(&edit_file_cases()[9], make_tool).await;
    }
}

// ════════════════════════════════════════════════════════════════════════════
// write_file — normal/boundary/error
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn write_file_matrix() {
    
    let cases = vec![
        ToolCase {
            name: "write_new_file",
            category: CaseCategory::Normal,
            fixtures: vec![],
            call: ("write_file", json!({"path":"out.txt","content":"hello"})),
            expect: Expect::OkFileEquals { path: "out.txt", content: "hello" },
        },
        ToolCase {
            name: "overwrite_existing",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"out.txt",content:"old"}],
            call: ("write_file", json!({"path":"out.txt","content":"new"})),
            expect: Expect::OkFileEquals { path: "out.txt", content: "new" },
        },
        ToolCase {
            name: "creates_parent_dirs",
            category: CaseCategory::Normal,
            fixtures: vec![],
            call: ("write_file", json!({"path":"sub/dir/out.txt","content":"nested"})),
            expect: Expect::OkFileEquals { path: "sub/dir/out.txt", content: "nested" },
        },
        ToolCase {
            name: "empty_content",
            category: CaseCategory::Boundary,
            fixtures: vec![],
            call: ("write_file", json!({"path":"empty.txt","content":""})),
            expect: Expect::OkFileEquals { path: "empty.txt", content: "" },
        },
        ToolCase {
            name: "missing_content_arg",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("write_file", json!({"path":"x.txt"})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// read_file — normal/boundary/error (no side effects but run serially for CWD)
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn read_file_matrix() {
    
    let cases = vec![
        ToolCase {
            name: "read_existing",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"a.txt",content:"hello musk"}],
            call: ("read_file", json!({"path":"a.txt"})),
            expect: Expect::OkExact("hello musk"),
        },
        ToolCase {
            name: "read_multiline",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"a.txt",content:"one\ntwo\nthree"}],
            call: ("read_file", json!({"path":"a.txt"})),
            expect: Expect::OkContains("two"),
        },
        ToolCase {
            name: "read_empty",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::File{path:"empty.txt",content:""}],
            call: ("read_file", json!({"path":"empty.txt"})),
            expect: Expect::OkExact(""),
        },
        ToolCase {
            name: "read_missing_file",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("read_file", json!({"path":"nope.txt"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "read_missing_arg",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("read_file", json!({})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// list_dir + glob + search — navigation tools
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn list_dir_matrix() {
    
    let cases = vec![
        ToolCase {
            name: "list_with_files",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"a.txt",content:"x"}, Fixture::File{path:"b.rs",content:"y"}],
            call: ("list_dir", json!({"path":"."})),
            expect: Expect::OkContains("a.txt"),
        },
        ToolCase {
            name: "list_empty_dir",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::Dir{path:"empty"}],
            call: ("list_dir", json!({"path":"empty"})),
            expect: Expect::OkContains("(empty directory)"),
        },
        ToolCase {
            name: "list_missing_dir",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("list_dir", json!({"path":"nope"})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

#[tokio::test]
#[serial_test::serial]
async fn glob_matrix() {
    
    let cases = vec![
        ToolCase {
            name: "glob_match",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"a.txt",content:"x"}, Fixture::File{path:"b.rs",content:"y"}],
            call: ("glob", json!({"pattern":"*.txt"})),
            expect: Expect::OkContains("a.txt"),
        },
        ToolCase {
            name: "glob_no_match",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::File{path:"a.txt",content:"x"}],
            call: ("glob", json!({"pattern":"*.md"})),
            expect: Expect::OkContains("no match"),
        },
        ToolCase {
            name: "glob_recursive",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"sub/c.txt",content:"x"}],
            call: ("glob", json!({"pattern":"**/*.txt"})),
            expect: Expect::OkContains("c.txt"),
        },
    ];
    run_cases(&cases, make_tool).await;
}

#[tokio::test]
#[serial_test::serial]
async fn search_matrix() {
    
    let cases = vec![
        ToolCase {
            name: "search_finds_pattern",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::File{path:"a.txt",content:"fn main() {}"}],
            call: ("search", json!({"pattern":"fn main"})),
            expect: Expect::OkContains("fn main"),
        },
        ToolCase {
            name: "search_no_match",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::File{path:"a.txt",content:"hello"}],
            call: ("search", json!({"pattern":"zzz"})),
            expect: Expect::OkContains("no match"),
        },
        ToolCase {
            name: "search_missing_arg",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("search", json!({})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// run_command — only safe commands (echo/type), per Design 003 §2.5
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn run_command_matrix() {
    let cases = vec![
        ToolCase {
            name: "echo_succeeds",
            category: CaseCategory::Normal,
            fixtures: vec![],
            // echo is universally safe (writes only to stdout)
            call: ("run_command", json!({"cmd":"echo hello_musk"})),
            expect: Expect::OkContains("hello_musk"),
        },
        ToolCase {
            name: "missing_command_arg",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("run_command", json!({})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// batch_replace 已删除(PLAN-039 T7):多编辑语义由 edit_file 的 edits[]
// 承担,原子性由前置校验(未命中/歧义/重叠在写盘前拒绝)全覆盖。
// ════════════════════════════════════════════════════════════════════════════

// ════════════════════════════════════════════════════════════════════════════
// edit_file pi 行为对拍表(PLAN-039 T11)——与 src/edit_diff.rs 单测同源的
// 8 类边界 fixtures,经完整工具链(垫片/BOM 拆分/行尾检测/行级保留/恢复)
// 用二进制精确断言(OkFileEquals 读原文比对)。
// ════════════════════════════════════════════════════════════════════════════

fn edit_file_parity_cases() -> Vec<ToolCase> {
    vec![
        // 1. CRLF 文件 + LF old_string(模型常态)→ 命中且写回保持 CRLF。
        ToolCase {
            name: "parity_crlf_file_lf_oldstring",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "alpha\r\nbeta\r\ngamma\r\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"beta","new_string":"BETA"}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "alpha\r\nBETA\r\ngamma\r\n" },
        },
        // 2. BOM:匹配前剥、写回恢复。
        ToolCase {
            name: "parity_bom_preserved",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "\u{FEFF}hello world")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"world","new_string":"musk"}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "\u{FEFF}hello musk" },
        },
        // 3. 智能引号:ASCII old_string 模糊命中;未触达行尾空白字节不变。
        ToolCase {
            name: "parity_smart_quotes_untouched_bytes",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "keep \n“hello”\ntail  \n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"\"hello\"","new_string":"[hi]"}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "keep \n[hi]\ntail  \n" },
        },
        // 4. NBSP:特殊空格规范化后命中。
        ToolCase {
            name: "parity_nbsp_normalized_to_space",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "a\u{00A0}b\nc")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"a b","new_string":"X"}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "X\nc" },
        },
        // 5. 行尾空白差异:模糊命中,触达行从规范化空间重写。
        ToolCase {
            name: "parity_trailing_whitespace_fuzzy",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "x  \ny")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"x\ny","new_string":"X\nY"}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "X\nY" },
        },
        // 6. 重复出现:规范化空间计数 >1 → 拒绝(pi 歧义报错)。
        ToolCase {
            name: "parity_duplicate_rejected",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "dup\ndup\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"dup","new_string":"x"}]})),
            expect: Expect::ErrContains("occurrences"),
        },
        // 7. 重叠编辑:拒绝且不落盘(原子)。
        ToolCase {
            name: "parity_overlap_rejected_atomically",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "abcdef\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[
                {"old_string":"bcd","new_string":"X"},
                {"old_string":"def","new_string":"Y"}
            ]})),
            expect: Expect::ErrContains("overlap"),
        },
        // 8. 模糊命中但原文混排 CRLF:未触达行保留 CRLF 字节,触达行恢复 CRLF。
        ToolCase {
            name: "parity_fuzzy_hit_in_crlf_file",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::file("a.txt", "one\r\n“q”\r\ntwo")],
            call: ("edit_file", json!({"path":"a.txt","edits":[{"old_string":"\"q\"","new_string":"\"Q\""}]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "one\r\n\"Q\"\r\ntwo" },
        },
        // 附:多重编辑(替代 batch_replace 的原子多编辑语义)。
        ToolCase {
            name: "parity_multi_edits_replaces_batch",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "foo bar baz\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[
                {"old_string":"foo","new_string":"FOO"},
                {"old_string":"baz","new_string":"BAZ"}
            ]})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "FOO bar BAZ\n" },
        },
        // 附:多重编辑中一条未命中 → 整体拒绝(原子性)。
        ToolCase {
            name: "parity_multi_edits_atomic_on_missing",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "foo\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":[
                {"old_string":"foo","new_string":"x"},
                {"old_string":"zzz","new_string":"y"}
            ]})),
            expect: Expect::ErrContains("Could not find edits[1]"),
        },
        // 附:入口垫片——模型把 edits 发成 JSON 字符串。
        ToolCase {
            name: "parity_edits_json_string_shim",
            category: CaseCategory::Boundary,
            fixtures: vec![Fixture::file("a.txt", "aaa bbb\n")],
            call: ("edit_file", json!({"path":"a.txt","edits":"[{\"old_string\":\"bbb\",\"new_string\":\"BBB\"}]"})),
            expect: Expect::OkFileEquals { path: "a.txt", content: "aaa BBB\n" },
        },
    ]
}

#[tokio::test]
#[serial_test::serial]
async fn edit_file_parity_matrix() {
    run_cases(&edit_file_parity_cases(), make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// read_file 分页(pi read.ts 语义;大文件截断用例在 src/tools.rs 单测覆盖)
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn read_file_paging_matrix() {
    let lines10 = "l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\nl9\nl10\n";
    let cases = vec![
        ToolCase {
            name: "offset_limit_window_with_note",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", lines10)],
            call: ("read_file", json!({"path":"a.txt","offset":3,"limit":4})),
            // 总行数含末尾换行的空行(pi split 口径 = 11)。
            expect: Expect::OkExact("l3\nl4\nl5\nl6\n\n[5 more lines in file. Use offset=7 to continue.]"),
        },
        ToolCase {
            name: "offset_out_of_bounds",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", lines10)],
            call: ("read_file", json!({"path":"a.txt","offset":99})),
            expect: Expect::ErrContains("beyond end of file"),
        },
        ToolCase {
            name: "read_whole_small_file_unchanged",
            category: CaseCategory::Normal,
            fixtures: vec![Fixture::file("a.txt", "hello\nmusk\n")],
            call: ("read_file", json!({"path":"a.txt"})),
            expect: Expect::OkExact("hello\nmusk\n"),
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// Security: path confinement (Design 004) — out-of-bounds paths rejected
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn path_confinement_rejects_traversal() {
    let cases = vec![
        ToolCase {
            name: "read_traversal_rejected",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "safe")],
            call: ("read_file", json!({"path":"../../../etc/passwd"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "write_traversal_rejected",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("write_file", json!({"path":"../escape.txt","content":"evil"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "edit_absolute_outside_rejected",
            category: CaseCategory::Error,
            fixtures: vec![Fixture::file("a.txt", "x")],
            call: ("edit_file", json!({"path":"/etc/hosts","old_string":"x","new_string":"y"})),
            expect: Expect::Err,
        },
        ToolCase {
            name: "glob_traversal_rejected",
            category: CaseCategory::Error,
            fixtures: vec![],
            call: ("glob", json!({"pattern":"*.txt","path":".."})),
            expect: Expect::Err,
        },
    ];
    run_cases(&cases, make_tool).await;
}

// ════════════════════════════════════════════════════════════════════════════
// Security: run_command classification (Design 004)
// ════════════════════════════════════════════════════════════════════════════

#[tokio::test]
#[serial_test::serial]
async fn run_command_whitelist_passes() {
    let cases = vec![
        ToolCase {
            name: "echo_whitelisted",
            category: CaseCategory::Normal,
            fixtures: vec![],
            call: ("run_command", json!({"cmd":"echo safe_cmd"})),
            expect: Expect::OkContains("safe_cmd"),
        },
    ];
    run_cases(&cases, make_tool).await;
}

#[tokio::test]
#[serial_test::serial]
async fn run_command_unknown_returns_paused() {
    let t = RunCommand::new();
    let result = t.execute(&json!({"cmd":"some-unknown-binary --flag"})).await.unwrap();
    let result = result.content;
    assert!(result.contains("PAUSED"), "unknown command should return PAUSED, got: {result}");
    assert!(result.contains("not on the whitelist"), "should explain why: {result}");
}

#[tokio::test]
#[serial_test::serial]
async fn run_command_dangerous_returns_paused() {
    let t = RunCommand::new();
    let result = t.execute(&json!({"cmd":"rm -rf /"})).await.unwrap();
    let result = result.content;
    assert!(result.contains("PAUSED"), "dangerous command should return PAUSED, got: {result}");
    assert!(result.contains("dangerous pattern"), "should warn about danger: {result}");
}

#[tokio::test]
#[serial_test::serial]
async fn run_command_force_overrides_pause() {
    let t = RunCommand::new();
    let result = t.execute(&json!({"cmd":"echo forced","force":true})).await.unwrap();
    let result = result.content;
    assert!(result.contains("forced"), "force should execute the command, got: {result}");
    assert!(!result.contains("PAUSED"), "force should not PAUSE: {result}");
}
