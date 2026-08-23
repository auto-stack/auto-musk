//! highlight-rs — PLAN-038 T15：syntect（two-face 全量语法集，与 auto-lang
//! code_editor 同内核同版本）对 fixtures 代码块输出 classed HTML（class = scope 串），
//! 供 node 侧 highlight-compare.mjs 与 prismjs/lowlight 做 per-char token 一致性矩阵。
//!
//! 用法：highlight-rs <fixtures_dir> <out_json>
//! fixtures_dir 内每个 `<lang>.txt` 为该语言代码样本；lang→扩展名映射见 EXT_MAP。

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// 语言键 → 文件扩展名（two-face SyntaxSet 按扩展名解析；bash=shell 脚本）。
const EXT_MAP: &[(&str, &str)] = &[
    ("rust", "rs"),
    ("typescript", "ts"),
    ("javascript", "js"),
    ("json", "json"),
    ("bash", "sh"),
    ("python", "py"),
    ("markdown", "md"),
    ("yaml", "yaml"),
    ("toml", "toml"),
    ("sql", "sql"),
    ("java", "java"),
    ("c", "c"),
    ("cpp", "cpp"),
    ("go", "go"),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: highlight-rs <fixtures_dir> <out_json>");
        std::process::exit(2);
    }
    let dir = PathBuf::from(&args[1]);
    let out_path = PathBuf::from(&args[2]);

    let ss: SyntaxSet = two_face::syntax::extra_newlines();
    let mut result: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut missing: Vec<String> = Vec::new();

    for (lang, ext) in EXT_MAP {
        let file = dir.join(format!("{lang}.txt"));
        let text = match fs::read_to_string(&file) {
            Ok(t) => t,
            Err(_) => {
                missing.push(lang.to_string());
                continue;
            }
        };
        // two-face 语法查找：按扩展名（构造伪文件名）
        let syntax = ss
            .find_syntax_for_file(format!("probe.{ext}"))
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                eprintln!("[highlight-rs] {lang}: 无 {ext} 语法,回退 Plain Text");
                &ss.find_syntax_plain_text()
            });
        let mut generator = ClassedHTMLGenerator::new_with_class_style(&syntax, &ss, ClassStyle::Spaced);
        let mut had_error = false;
        for line in LinesWithEndings::from(&text) {
            if generator.parse_html_for_line_which_includes_newline(line).is_err() {
                had_error = true;
                break;
            }
        }
        if had_error {
            eprintln!("[highlight-rs] {lang}: 解析中断");
        }
        let html = generator.finalize();
        result.insert(
            lang.to_string(),
            serde_json::json!({
                "syntax": syntax.name,
                "html": html,
            }),
        );
    }

    let payload = serde_json::json!({
        "engine": "syntect/two-face (auto-lang aligned: syntect 5 + two-face 0.4)",
        "languages": result,
        "missing_fixtures": missing,
    });
    fs::write(&out_path, serde_json::to_string_pretty(&payload).unwrap())
        .expect("write out_json");
    eprintln!("[highlight-rs] ok -> {}", out_path.display());
}
