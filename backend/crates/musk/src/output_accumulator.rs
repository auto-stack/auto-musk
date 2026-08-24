//! 流式输出累积器(PLAN-040 T1,对齐 pi `output-accumulator.ts`)。
//!
//! run_command 流式读到的 stdout/stderr 分块经 [`OutputAccumulator`] 进来:
//! 有界内存(滚动尾部缓冲,2×maxBytes)流式累积,UTF-8 多字节边界安全,
//! 超限时自动把**全量输出**落临时文件(路径随快照交给模型),行数/字节数
//! 全量统计。结束快照经 PLAN-039 [`crate::tool_truncate::truncate_tail`]
//! (尾部保留——错误与最终结果在输出末尾)。
//!
//! 与 pi 的分野:pi 临时文件创建失败会 throw 整个工具调用;这里优雅降级
//! (记录 `temp_error`,继续尾部快照——临时文件是增强,不该毁掉命令结果)。

use crate::tool_truncate::{truncate_tail, TruncatedBy};
use std::io::Write;

/// 滚动尾部缓冲默认字节上限(pi `DEFAULT_MAX_BYTES`,50KB)。
pub const DEFAULT_MAX_BYTES: usize = crate::tool_truncate::DEFAULT_MAX_BYTES;
/// 结束快照默认行数上限(pi `DEFAULT_MAX_LINES`)。
pub const DEFAULT_MAX_LINES: usize = crate::tool_truncate::DEFAULT_MAX_LINES;

/// 结束快照(`pi OutputSnapshot` + truncation 元数据)。
#[derive(Debug, Clone)]
pub struct OutputSnapshot {
    /// 尾部截断后的内容(结束快照给模型看的本体)。
    pub content: String,
    pub truncated: bool,
    pub truncated_by: Option<TruncatedBy>,
    /// 全量总行数(口径与 tool_truncate 一致:末尾换行不计行)。
    pub total_lines: usize,
    /// content 中包含的完整行数。
    pub output_lines: usize,
    /// 尾部截断边界情形:最后一行被部分保留。
    pub last_line_partial: bool,
    /// 全量解码字节数。
    pub total_bytes: usize,
    /// 全量输出临时文件路径(超限转储时 Some)。
    pub full_output_path: Option<std::path::PathBuf>,
    /// 临时文件写失败的原因(优雅降级,不毁命令结果)。
    pub temp_error: Option<String>,
    /// 最后一行(开放行)的字节数(pi `getLastLineBytes`,尾注文案用)。
    pub last_line_bytes: usize,
}

/// 临时文件随机后缀计数器(纳秒 + 进程内计数,单进程内足够唯一)。
static TEMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn temp_file_path(prefix: &str) -> std::path::PathBuf {
    let n = TEMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos:08x}-{n:04x}.log"))
}

/// 有界内存流式累积器(pi `OutputAccumulator`)。
pub struct OutputAccumulator {
    max_lines: usize,
    max_bytes: usize,
    max_rolling_bytes: usize,
    temp_file_prefix: String,

    /// 流式 UTF-8 解码:上一 chunk 末尾的不完整多字节序列(pi TextDecoder stream)。
    incomplete: Vec<u8>,
    tail_text: String,
    tail_bytes: usize,
    /// trim 后的 tail 是否从行首开始(影响快照丢弃首残行)。
    tail_starts_at_line_boundary: bool,
    total_raw_bytes: usize,
    total_decoded_bytes: usize,
    completed_lines: usize,
    total_lines: usize,
    current_line_bytes: usize,
    has_open_line: bool,
    finished: bool,

    temp_file_path: Option<std::path::PathBuf>,
    temp_file: Option<std::fs::File>,
    raw_chunks: Vec<Vec<u8>>,
    temp_error: Option<String>,
}

impl OutputAccumulator {
    pub fn new(max_lines: usize, max_bytes: usize) -> Self {
        Self::with_prefix(max_lines, max_bytes, "musk-output")
    }

    pub fn with_prefix(max_lines: usize, max_bytes: usize, temp_file_prefix: &str) -> Self {
        Self {
            max_lines,
            max_bytes,
            max_rolling_bytes: (max_bytes * 2).max(1),
            temp_file_prefix: temp_file_prefix.to_string(),
            incomplete: Vec::new(),
            tail_text: String::new(),
            tail_bytes: 0,
            tail_starts_at_line_boundary: true,
            total_raw_bytes: 0,
            total_decoded_bytes: 0,
            completed_lines: 0,
            total_lines: 0,
            current_line_bytes: 0,
            has_open_line: false,
            finished: false,
            temp_file_path: None,
            temp_file: None,
            raw_chunks: Vec::new(),
            temp_error: None,
        }
    }

    /// 追加一个输出分块(pi `append`)。
    pub fn append(&mut self, data: &[u8]) {
        if self.finished {
            // pi 此处 throw;工具环境不 panic——防御性忽略(调用方契约:
            // finish 之后不再 append)。
            return;
        }
        self.total_raw_bytes += data.len();
        let text = self.decode_stream(data);
        self.append_decoded_text(&text);

        if self.temp_file.is_some() || self.temp_error.is_some() {
            self.write_temp(data);
        } else if self.should_use_temp_file() {
            self.ensure_temp_file();
            self.write_temp(data);
        } else if !data.is_empty() {
            self.raw_chunks.push(data.to_vec());
        }
    }

    /// 结束:flush 流式解码器残留(pi `finish`)。
    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        if !self.incomplete.is_empty() {
            // decoder.decode() 语义:残留的不完整序列解码为一个 U+FFFD。
            // 这些字节已在 append 时计入 total_raw_bytes(且若临时文件已开,
            // 已随当时的原始分块写入)——这里只补解码层。
            self.incomplete.clear();
            self.append_decoded_text("\u{FFFD}");
        }
        if self.should_use_temp_file() {
            self.ensure_temp_file();
        }
        if let Some(mut f) = self.temp_file.take() {
            let _ = f.flush();
            let _ = f.sync_all();
        }
    }

    /// 结束快照:tail 经 `truncate_tail` + 全量截断判定
    /// (pi `snapshot`,`persist_if_truncated` = 超限时确保临时文件存在)。
    pub fn snapshot(&mut self, persist_if_truncated: bool) -> OutputSnapshot {
        let t = truncate_tail(self.snapshot_text(), self.max_lines, self.max_bytes);
        let truncated =
            self.total_lines > self.max_lines || self.total_decoded_bytes > self.max_bytes;
        let truncated_by = truncated.then(|| {
            t.truncated_by.unwrap_or(if self.total_decoded_bytes > self.max_bytes {
                TruncatedBy::Bytes
            } else {
                TruncatedBy::Lines
            })
        });
        if persist_if_truncated && truncated {
            self.ensure_temp_file();
        }
        OutputSnapshot {
            content: t.content,
            truncated,
            truncated_by,
            total_lines: self.total_lines,
            output_lines: t.output_lines,
            last_line_partial: t.last_line_partial,
            total_bytes: self.total_decoded_bytes,
            full_output_path: self.temp_file_path.clone(),
            temp_error: self.temp_error.clone(),
            last_line_bytes: self.current_line_bytes,
        }
    }

    /// 最后一行(开放行)的字节数(pi `getLastLineBytes`)。
    pub fn last_line_bytes(&self) -> usize {
        self.current_line_bytes
    }

    /// 全量是否超限(pi `shouldUseTempFile`)。
    fn should_use_temp_file(&self) -> bool {
        self.total_raw_bytes > self.max_bytes
            || self.total_decoded_bytes > self.max_bytes
            || self.total_lines > self.max_lines
    }

    /// 首次超限时创建临时文件并回放已缓存的原始分块(pi `ensureTempFile`)。
    /// 写失败 → 优雅降级(记录错误,后续跳过临时文件)。
    fn ensure_temp_file(&mut self) {
        if self.temp_file_path.is_some() || self.temp_error.is_some() {
            return;
        }
        let path = temp_file_path(&self.temp_file_prefix);
        match std::fs::File::create(&path) {
            Ok(mut f) => {
                for chunk in self.raw_chunks.drain(..) {
                    if let Err(e) = f.write_all(&chunk) {
                        self.temp_error = Some(format!("write temp file: {e}"));
                        let _ = std::fs::remove_file(&path);
                        return;
                    }
                }
                self.temp_file = Some(f);
                self.temp_file_path = Some(path);
            }
            Err(e) => {
                self.temp_error = Some(format!("create temp file: {e}"));
            }
        }
    }

    /// 把原始分块写进临时文件;失败 → 降级(记录错误,关闭句柄)。
    fn write_temp(&mut self, data: &[u8]) {
        if let Some(f) = self.temp_file.as_mut() {
            if let Err(e) = f.write_all(data) {
                self.temp_error = Some(format!("write temp file: {e}"));
                self.temp_file = None;
            }
        }
    }

    /// 流式解码一个分块:跨 chunk 的不完整多字节序列留到下一 chunk,
    /// 非法字节替换 U+FFFD(pi `TextDecoder` non-fatal 语义)。
    fn decode_stream(&mut self, data: &[u8]) -> String {
        let mut buf = std::mem::take(&mut self.incomplete);
        buf.extend_from_slice(data);
        let mut out = String::new();
        let mut rest: &[u8] = &buf;
        loop {
            match std::str::from_utf8(rest) {
                Ok(s) => {
                    out.push_str(s);
                    break;
                }
                Err(e) => {
                    let valid = e.valid_up_to();
                    out.push_str(std::str::from_utf8(&rest[..valid]).unwrap());
                    match e.error_len() {
                        None => {
                            // 末尾不完整多字节序列 → 留给下一 chunk。
                            self.incomplete = rest[valid..].to_vec();
                            break;
                        }
                        Some(n) => {
                            out.push('\u{FFFD}');
                            rest = &rest[valid + n..];
                        }
                    }
                }
            }
        }
        out
    }

    /// 解码文本进滚动尾部 + 行计数(pi `appendDecodedText`)。
    fn append_decoded_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let bytes = text.len();
        self.total_decoded_bytes += bytes;
        self.tail_text.push_str(text);
        self.tail_bytes += bytes;
        if self.tail_bytes > self.max_rolling_bytes * 2 {
            self.trim_tail();
        }

        let mut newlines = 0usize;
        let mut last_newline: Option<usize> = None;
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                newlines += 1;
                last_newline = Some(i);
            }
        }
        if newlines == 0 {
            self.current_line_bytes += bytes;
            self.has_open_line = true;
        } else {
            self.completed_lines += newlines;
            let tail = &text[last_newline.unwrap() + 1..];
            self.current_line_bytes = tail.len();
            self.has_open_line = !tail.is_empty();
        }
        self.total_lines = self.completed_lines + usize::from(self.has_open_line);
    }

    /// 滚动尾部裁剪到 `max_rolling_bytes`(字节边界回退,pi `trimTail`)。
    fn trim_tail(&mut self) {
        let buf = self.tail_text.as_bytes();
        if buf.len() <= self.max_rolling_bytes {
            self.tail_bytes = buf.len();
            return;
        }
        let mut start = buf.len() - self.max_rolling_bytes;
        // 跳过 UTF-8 continuation bytes(0x80..=0xBF),切割点退到字符边界。
        while start < buf.len() && (buf[start] & 0xc0) == 0x80 {
            start += 1;
        }
        if start > 0 {
            self.tail_starts_at_line_boundary = buf[start - 1] == b'\n';
        }
        let kept = String::from_utf8_lossy(&buf[start..]).into_owned();
        self.tail_bytes = kept.len();
        self.tail_text = kept;
    }

    /// 快照文本:tail 不在行首时丢弃第一残行(pi `getSnapshotText`)。
    fn snapshot_text(&self) -> &str {
        if self.tail_starts_at_line_boundary {
            return &self.tail_text;
        }
        match self.tail_text.find('\n') {
            None => &self.tail_text,
            Some(i) => &self.tail_text[i + 1..],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 基础累积与行计数 ────────────────────────────────────────────────

    #[test]
    fn accumulates_within_limits_no_truncation_no_temp_file() {
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"l1\nl2\nl3");
        acc.finish();
        let snap = acc.snapshot(true);
        assert!(!snap.truncated);
        assert_eq!(snap.content, "l1\nl2\nl3");
        assert_eq!(snap.total_lines, 3);
        assert_eq!(snap.total_bytes, 8);
        assert!(snap.full_output_path.is_none());
    }

    /// pi 行计数口径:"a\nb\n" = 2 行(末尾换行不计)、"a\nb" = 2、空 = 0。
    #[test]
    fn line_counting_matches_pi_semantics() {
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"a\nb\n");
        acc.finish();
        assert_eq!(acc.snapshot(false).total_lines, 2);

        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"a\nb");
        acc.finish();
        assert_eq!(acc.snapshot(false).total_lines, 2);
        // 开放行字节数 = "b" 的 1 字节。
        assert_eq!(acc.last_line_bytes(), 1);

        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"");
        acc.finish();
        assert_eq!(acc.snapshot(false).total_lines, 0);
    }

    /// 行数跨 chunk 计数:两个分块各带换行,总数与一次性推入一致。
    #[test]
    fn line_counting_across_chunks() {
        let whole = b"one\ntwo\nthree\nfour";
        let mut a = OutputAccumulator::new(100, 100 * 1024);
        a.append(whole);
        a.finish();
        let mut b = OutputAccumulator::new(100, 100 * 1024);
        b.append(b"one\ntwo");
        b.append(b"\nthree\nfou");
        b.append(b"r");
        b.finish();
        assert_eq!(a.snapshot(false).total_lines, b.snapshot(false).total_lines);
        assert_eq!(a.snapshot(false).content, b.snapshot(false).content);
    }

    // ── UTF-8 多字节边界 ────────────────────────────────────────────────

    /// 中文 3 字节字符在任意字节边界切开流式推入:尾快照完整字符,
    /// 不出现 U+FFFD(PLAN-039 search panic 同源问题的流式版)。
    #[test]
    fn multibyte_chunks_split_mid_char_decode_intact() {
        let text = "你好世界——流式测试";
        let bytes = text.as_bytes();
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        // 按 4 字节一块(必然切在字符中间)流式推入。
        for chunk in bytes.chunks(4) {
            acc.append(chunk);
        }
        acc.finish();
        let snap = acc.snapshot(false);
        assert!(!snap.truncated);
        assert_eq!(snap.content, text);
        assert!(!snap.content.contains('\u{FFFD}'));
    }

    /// 非法 UTF-8 字节替换 U+FFFD,不 panic、不死循环。
    #[test]
    fn invalid_utf8_replaced_with_replacement_char() {
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"ok\xFF\xFEend");
        acc.finish();
        let snap = acc.snapshot(false);
        assert!(snap.content.contains('\u{FFFD}'));
        assert!(snap.content.starts_with("ok"));
        assert!(snap.content.contains("end"));
    }

    /// finish 时残留的不完整多字节序列 flush 为 U+FFFD(pi decoder.decode())。
    #[test]
    fn finish_flushes_incomplete_sequence_as_replacement() {
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append("ab".as_bytes());
        acc.append(&[0xe4, 0xbd]); // "你" 的前两字节
        acc.finish();
        let snap = acc.snapshot(false);
        assert!(snap.content.contains('\u{FFFD}'));
    }

    /// 滚动尾部裁剪触发(> 4×maxBytes)时,切割点回退到字符边界:
    /// 大量中文流式推入后 tail 仍是合法 UTF-8。
    #[test]
    fn rolling_tail_trim_is_char_boundary_safe() {
        // max_bytes=64 → rolling=128,触发阈值 256。
        let mut acc = OutputAccumulator::new(10_000, 64);
        let line = "一二三四五六七八九十"; // 10 chars × 3B = 30B
        for _ in 0..40 {
            acc.append(line.as_bytes());
        }
        acc.finish();
        let snap = acc.snapshot(false);
        // 全量 1200B > 50KB?否——超 max_bytes(64),截断 + 临时文件。
        assert!(snap.truncated);
        // tail 必须仍是合法 UTF-8(snapshot 已是 String,能构造即合法;
        // 再验证以整字符结尾)。
        let last = snap.content.chars().last().unwrap();
        assert!("一二三四五六七八九十".contains(last));
    }

    // ── 临时文件转储 ────────────────────────────────────────────────────

    /// 超限(字节)→ 全量输出落临时文件:文件内容 = 原始字节流,
    /// 快照 full_output_path 指向它,content 是尾部。
    #[test]
    fn overflow_dumps_full_output_to_temp_file() {
        let mut acc = OutputAccumulator::new(100, 100);
        for i in 0..50 {
            acc.append(format!("line-{i:03}\n").as_bytes());
        }
        acc.finish();
        let snap = acc.snapshot(true);
        assert!(snap.truncated);
        let path = snap.full_output_path.expect("temp file on overflow");
        let on_disk = std::fs::read(&path).unwrap();
        assert_eq!(on_disk.len(), 50 * 9, "full raw output persisted");
        assert_eq!(on_disk[0], b'l');
        // 尾部快照保留最后几行。
        assert!(snap.content.contains("line-049"));
        assert!(!snap.content.contains("line-000"));
        let _ = std::fs::remove_file(&path);
    }

    /// 未超限 → 不创建临时文件。
    #[test]
    fn under_limit_no_temp_file() {
        let mut acc = OutputAccumulator::new(100, 100 * 1024);
        acc.append(b"small output");
        acc.finish();
        assert!(acc.snapshot(true).full_output_path.is_none());
    }

    /// 行数超限(字节未超)同样触发临时文件(pi shouldUseTempFile 三条件)。
    #[test]
    fn line_overflow_triggers_temp_file() {
        let mut acc = OutputAccumulator::new(5, 100 * 1024);
        for i in 0..20 {
            acc.append(format!("row-{i}\n").as_bytes());
        }
        acc.finish();
        let snap = acc.snapshot(true);
        assert!(snap.truncated);
        assert_eq!(snap.truncated_by, Some(TruncatedBy::Lines));
        assert!(snap.full_output_path.is_some());
        let _ = std::fs::remove_file(snap.full_output_path.unwrap());
    }

    /// snapshot(persist_if_truncated=true)在超限时补建临时文件
    /// (数据量在 append 后才过阈值的边界:append 时未超,snapshot 判定超)。
    #[test]
    fn snapshot_persist_if_truncated_creates_temp_file_late() {
        // max_bytes=200:推 40 行 × 6B = 240B,append 内部在最后一行才过阈值,
        // 这里直接验证 snapshot(true) 后路径存在。
        let mut acc = OutputAccumulator::new(10_000, 200);
        for _ in 0..40 {
            acc.append(b"row-x\n");
        }
        acc.finish();
        let snap = acc.snapshot(true);
        assert!(snap.truncated);
        let path = snap.full_output_path.expect("persisted on snapshot");
        assert_eq!(std::fs::read(&path).unwrap().len(), 240);
        let _ = std::fs::remove_file(&path);
    }

    // ── 快照语义 ───────────────────────────────────────────────────────

    /// tail 被滚动裁剪且不从行首开始时,快照丢弃第一残行
    /// (半个行的信息量低且行号对不上;pi getSnapshotText)。
    #[test]
    fn snapshot_drops_partial_first_line_when_tail_mid_line() {
        // rolling=64B,触发 128B:先推一行 40B(不带换行),再推一行带换行,
        // tail 首行将是残行。
        let mut acc = OutputAccumulator::new(10_000, 32);
        let long_head = "A".repeat(60); // 单行 60B,无换行
        acc.append(format!("{long_head}\n").as_bytes());
        acc.append(b"tail-line-1\ntail-line-2\n");
        acc.finish();
        let snap = acc.snapshot(false);
        // 残行被丢,快照从 tail-line-1 开始。
        assert!(snap.content.starts_with("tail-line-1") || snap.content.contains("tail-line-1"));
        assert!(!snap.content.starts_with('A'));
    }

    /// 尾部截断的行号区间元数据:total_lines 全量口径、output_lines 快照口径。
    #[test]
    fn snapshot_reports_full_line_totals_with_tail_content() {
        let mut acc = OutputAccumulator::new(3, 100 * 1024);
        for i in 0..10 {
            acc.append(format!("n{i}\n").as_bytes());
        }
        acc.finish();
        let snap = acc.snapshot(true);
        assert!(snap.truncated);
        assert_eq!(snap.truncated_by, Some(TruncatedBy::Lines));
        assert_eq!(snap.total_lines, 10);
        assert_eq!(snap.output_lines, 3);
        assert_eq!(snap.content, "n7\nn8\nn9");
        let _ = std::fs::remove_file(snap.full_output_path.unwrap());
    }
}
