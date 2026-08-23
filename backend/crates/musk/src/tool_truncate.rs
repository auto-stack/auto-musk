//! 共享截断模块(PLAN-039,对齐 pi `truncate.ts`)。
//!
//! 工具输出的双限制截断:行数上限与字节上限,先到为准;切割点永远落在
//! UTF-8 字符边界上——这是本模块存在的直接动因:`String::truncate` 在
//! 多字节字符中间切割会 panic(中文内容 8KB 边界几乎必然切在字符中间,
//! 曾让 search 工具有整进程崩溃风险,见 tools.rs 旧 `result.truncate`)。
//!
//! 字节数一律取 `str::len()`(UTF-8 字节数);行数统计与 pi 一致:按
//! `\n` 切分、末尾换行不计为一行。

/// read_file / 全文截断默认行数上限(pi `DEFAULT_MAX_LINES`)。
pub const DEFAULT_MAX_LINES: usize = 2000;
/// read_file / 全文截断默认字节上限(pi `DEFAULT_MAX_BYTES`,50KB)。
pub const DEFAULT_MAX_BYTES: usize = 50 * 1024;
/// search 单行匹配的字符数上限(pi `GREP_MAX_LINE_LENGTH`)。
pub const GREP_MAX_LINE_LENGTH: usize = 500;

/// 触发截断的限制类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TruncatedBy {
    Lines,
    Bytes,
}

/// 截断结果元数据。PLAN-027(content/details 分离)落地后应整体移入
/// `ToolOutput.details`;当前挂在返回字符串尾部(见 tools.rs 挂接点标注)。
#[derive(Clone, Debug)]
pub struct TruncationResult {
    /// 截断后的内容。
    pub content: String,
    /// 是否发生了截断。
    pub truncated: bool,
    /// 哪个限制触发(`truncated == false` 时为 None)。
    pub truncated_by: Option<TruncatedBy>,
    /// 原始内容总行数(按 pi 口径:末尾换行不计行)。
    pub total_lines: usize,
    /// 输出中包含的完整行数。
    pub output_lines: usize,
    /// head 截断时首行单独超出字节上限(内容为空,调用方应给逃生提示)。
    pub first_line_exceeds_limit: bool,
    /// tail 截断的边界情形:最后一行被部分保留(自末端按字节切)。
    pub last_line_partial: bool,
}

impl TruncationResult {
    fn not_truncated(content: &str, total_lines: usize) -> Self {
        Self {
            content: content.to_string(),
            truncated: false,
            truncated_by: None,
            total_lines,
            output_lines: total_lines,
            first_line_exceeds_limit: false,
            last_line_partial: false,
        }
    }
}

/// 行数统计(pi `splitLinesForCounting`):空串为 0 行;末尾换行不计行。
fn count_lines(content: &str) -> usize {
    if content.is_empty() {
        return 0;
    }
    let mut n = content.split('\n').count();
    if content.ends_with('\n') {
        n -= 1;
    }
    n
}

/// 按行切分并统计口径对齐 pi(末尾换行后的空段丢弃)。
/// 每行保留原有内容(含行尾 `\r`),不自带换行符。
fn lines_for_counting(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = content.split('\n').collect();
    if content.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// 头部截断(保留前 N 行/字节),read_file 用。
///
/// 永不返回半行;首行单独超字节上限时返回空内容并置
/// `first_line_exceeds_limit`(调用方据此给出 run_command 逃生提示)。
pub fn truncate_head(content: &str, max_lines: usize, max_bytes: usize) -> TruncationResult {
    let total_bytes = content.len();
    let lines = lines_for_counting(content);
    let total_lines = lines.len();

    if total_lines <= max_lines && total_bytes <= max_bytes {
        return TruncationResult::not_truncated(content, total_lines);
    }

    // 首行单独超限:无任何完整行可给,交调用方走逃生通道。
    if lines[0].len() > max_bytes {
        return TruncationResult {
            content: String::new(),
            truncated: true,
            truncated_by: Some(TruncatedBy::Bytes),
            total_lines,
            output_lines: 0,
            first_line_exceeds_limit: true,
            last_line_partial: false,
        };
    }

    // 逐行累积完整行;第 i 行的代价 = 行字节 + (i>0 时 1 字节换行)。
    let mut out: Vec<&str> = Vec::with_capacity(max_lines.min(total_lines));
    let mut out_bytes = 0usize;
    let mut truncated_by = TruncatedBy::Lines;
    for (i, line) in lines.iter().enumerate().take(max_lines) {
        let line_bytes = line.len() + usize::from(i > 0);
        if out_bytes + line_bytes > max_bytes {
            truncated_by = TruncatedBy::Bytes;
            break;
        }
        out.push(line);
        out_bytes += line_bytes;
    }
    if out.len() >= max_lines && out_bytes <= max_bytes {
        truncated_by = TruncatedBy::Lines;
    }

    TruncationResult {
        content: out.join("\n"),
        truncated: true,
        truncated_by: Some(truncated_by),
        total_lines,
        output_lines: out.len(),
        first_line_exceeds_limit: false,
        last_line_partial: false,
    }
}

/// 尾部截断(保留后 N 行/字节),run_command/search 用。
///
/// 边界情形:最后一行单独超字节上限时,从末端按字节保留(字符边界安全)。
pub fn truncate_tail(content: &str, max_lines: usize, max_bytes: usize) -> TruncationResult {
    let total_bytes = content.len();
    let lines = lines_for_counting(content);
    let total_lines = lines.len();

    if total_lines <= max_lines && total_bytes <= max_bytes {
        return TruncationResult::not_truncated(content, total_lines);
    }

    // 自末端向前收集完整行。行字节统计口径与 head 相同:换行只算在
    // 非首行输出行上(与 pi 一致:join("\n") 后的字节数)。
    let mut out: Vec<&str> = Vec::new();
    let mut out_bytes = 0usize;
    let mut truncated_by = TruncatedBy::Lines;
    let mut last_line_partial = false;

    for line in lines.iter().rev() {
        if out.len() >= max_lines {
            break;
        }
        let line_bytes = line.len() + usize::from(!out.is_empty());
        if out_bytes + line_bytes > max_bytes {
            truncated_by = TruncatedBy::Bytes;
            // 边界情形:一行都没有且末行单独超限 → 从末端按字节保留,
            // 切割点回退到字符边界(这里与裸 String::truncate 的分野所在)。
            if out.is_empty() {
                let start = floor_char_boundary_from_end(line, max_bytes);
                out.push(&line[start..]);
                out_bytes = line.len() - start;
                last_line_partial = true;
            }
            break;
        }
        out.push(line);
        out_bytes += line_bytes;
    }
    if out.len() >= max_lines && out_bytes <= max_bytes {
        truncated_by = TruncatedBy::Lines;
    }
    out.reverse();

    TruncationResult {
        content: out.join("\n"),
        truncated: true,
        truncated_by: Some(truncated_by),
        total_lines,
        output_lines: out.len(),
        first_line_exceeds_limit: false,
        last_line_partial,
    }
}

/// 单行截断(search 匹配行,500 字符),超限时追加 `... [truncated]`。
pub fn truncate_line(line: &str, max_chars: usize) -> String {
    if line.chars().count() <= max_chars {
        return line.to_string();
    }
    let head: String = line.chars().take(max_chars).collect();
    format!("{head}... [truncated]")
}

/// 自末端保留至多 `max_bytes` 字节,起点回退到 UTF-8 字符边界。
fn floor_char_boundary_from_end(s: &str, max_bytes: usize) -> usize {
    if s.len() <= max_bytes {
        return 0;
    }
    let mut start = s.len() - max_bytes;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    start
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── truncate_head ──────────────────────────────────────────────────

    #[test]
    fn head_no_truncation_when_within_limits() {
        let r = truncate_head("one\ntwo\nthree", 2000, 50 * 1024);
        assert!(!r.truncated);
        assert_eq!(r.content, "one\ntwo\nthree");
        assert_eq!(r.total_lines, 3);
        assert_eq!(r.output_lines, 3);
        assert!(r.truncated_by.is_none());
    }

    #[test]
    fn head_line_limit_keeps_first_n_lines() {
        // 5 行,限 2 行 → 保留前 2 行,触发 Lines。
        let content = "l1\nl2\nl3\nl4\nl5";
        let r = truncate_head(content, 2, 50 * 1024);
        assert!(r.truncated);
        assert_eq!(r.truncated_by, Some(TruncatedBy::Lines));
        assert_eq!(r.content, "l1\nl2");
        assert_eq!(r.total_lines, 5);
        assert_eq!(r.output_lines, 2);
    }

    #[test]
    fn head_byte_limit_never_cuts_mid_line() {
        // 字节上限 7:"l1\nl2\nl3" → 第 2 行加换行共 6 字节,第 3 行会到 9 > 7。
        let r = truncate_head("l1\nl2\nl3", 2000, 7);
        assert!(r.truncated);
        assert_eq!(r.truncated_by, Some(TruncatedBy::Bytes));
        assert_eq!(r.content, "l1\nl2");
        assert_eq!(r.output_lines, 2);
    }

    /// 多字节中文:字节边界落在字符中间时必须退到字符边界,不 panic。
    /// 6 个三字节中文行(每行 3 字节 + 换行),字节上限 10:
    /// 行 1 = 3B,行 2 = 3+1+3 = 7 ≤ 10,行 3 = +4 = 11 > 10 → 保留 2 行。
    #[test]
    fn head_multibyte_never_splits_char_boundary() {
        let content = "你\n好\n世\n界\n吗\n呢";
        let r = truncate_head(content, 2000, 10);
        assert!(r.truncated);
        assert_eq!(r.truncated_by, Some(TruncatedBy::Bytes));
        assert_eq!(r.content, "你\n好");
        assert_eq!(r.output_lines, 2);
    }

    /// 恰好整行:累计正好等于上限时不截断该行。
    #[test]
    fn head_exactly_at_limit_includes_the_line() {
        // "l1\nl2" = 5 字节,上限 5 → 两行都在。
        let r = truncate_head("l1\nl2", 2000, 5);
        assert!(!r.truncated);
        assert_eq!(r.content, "l1\nl2");
    }

    /// 首行超限:内容为空 + first_line_exceeds_limit 信号。
    #[test]
    fn head_first_line_exceeds_limit_flags_escape_hatch() {
        let r = truncate_head("aaaa\nbbbb", 2000, 3);
        assert!(r.truncated);
        assert!(r.first_line_exceeds_limit);
        assert_eq!(r.content, "");
        assert_eq!(r.output_lines, 0);
    }

    #[test]
    fn head_empty_content() {
        let r = truncate_head("", 2000, 50 * 1024);
        assert!(!r.truncated);
        assert_eq!(r.total_lines, 0);
        assert_eq!(r.content, "");
    }

    /// 行尾换行不计行(pi 口径):"a\nb\n" 是 2 行,不是 3 行。
    #[test]
    fn head_trailing_newline_not_counted_as_line() {
        let r = truncate_head("a\nb\n", 2, 50 * 1024);
        assert!(!r.truncated, "2 行 + 末尾换行在限内");
        assert_eq!(r.total_lines, 2);
    }

    // ── truncate_tail ──────────────────────────────────────────────────

    #[test]
    fn tail_no_truncation_when_within_limits() {
        let r = truncate_tail("one\ntwo", 2000, 50 * 1024);
        assert!(!r.truncated);
        assert_eq!(r.content, "one\ntwo");
    }

    #[test]
    fn tail_line_limit_keeps_last_n_lines() {
        let r = truncate_tail("l1\nl2\nl3\nl4\nl5", 2, 50 * 1024);
        assert!(r.truncated);
        assert_eq!(r.truncated_by, Some(TruncatedBy::Lines));
        assert_eq!(r.content, "l4\nl5");
        assert_eq!(r.output_lines, 2);
        assert_eq!(r.total_lines, 5);
    }

    #[test]
    fn tail_byte_limit_keeps_whole_lines_from_end() {
        // 上限 7:自末端 l3(2B)+ 换行 + l2 = 5 + l1 = 8 > 7 → 保留 l2\nl3。
        let r = truncate_tail("l1\nl2\nl3", 2000, 7);
        assert!(r.truncated);
        assert_eq!(r.truncated_by, Some(TruncatedBy::Bytes));
        assert_eq!(r.content, "l2\nl3");
    }

    /// search panic 修复的回归:多字节内容自末端按字节截断,退到字符边界。
    /// 8 个三字节中文字符共 24 字节,上限 7 字节 → 从末端取 ≤7B 的整字符。
    #[test]
    fn tail_multibyte_partial_line_is_char_boundary_safe() {
        let line = "一二三四五六七八"; // 8 × 3B = 24B,单行
        let r = truncate_tail(line, 2000, 7);
        assert!(r.truncated);
        assert!(r.last_line_partial);
        // 自末端最多 7 字节且不切半字符 → 2 个字符(6B)。
        assert_eq!(r.content, "七八");
    }

    #[test]
    fn tail_exact_bytes_keeps_all() {
        let r = truncate_tail("ab\ncd", 2000, 5);
        assert!(!r.truncated);
    }

    // ── truncate_line ──────────────────────────────────────────────────

    #[test]
    fn line_within_limit_unchanged() {
        assert_eq!(truncate_line("short line", 500), "short line");
    }

    #[test]
    fn line_over_limit_gets_suffix() {
        let long = "x".repeat(600);
        let out = truncate_line(&long, 500);
        assert_eq!(out.chars().count(), 500 + "... [truncated]".chars().count());
        assert!(out.ends_with("... [truncated]"));
    }

    /// 按字符数(而非字节数)截断:中文 10 字 = 30 字节但只有 10 chars。
    #[test]
    fn line_counts_chars_not_bytes() {
        let cn = "字".repeat(10); // 10 chars, 30 bytes
        assert_eq!(truncate_line(&cn, 500), cn);
        let out = truncate_line(&cn, 5);
        assert!(out.starts_with("字字字字字"));
        assert!(out.ends_with("... [truncated]"));
    }
}
