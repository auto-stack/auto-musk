//! `edit_file` 的匹配/应用核心(PLAN-039 T5)。
//!
//! 移植 pi `edit-diff.ts`(main @ a1f955e9f)的四个关键机制:
//!
//! 1. **CRLF/BOM 往返**:匹配前拆 BOM、规范为 LF;写回前恢复(模型输出
//!   的 old_string 几乎永远不带 BOM、用 LF)。
//! 2. **精确优先 + 模糊回退**:先精确 `find`;失败后在规范化空间
//!   (NFKC、去行尾空白、智能引号/破折号/特殊空格 → ASCII)重试。
//! 3. **行级保留回写**:模糊命中时,只有被触达的行从规范化空间写回,
//!   未触达行保留原始字节——防止规范化污染全文件(核心正确性机制)。
//! 4. **多重编辑**:所有 edit 对同一份原始内容匹配(非增量)、歧义计数、
//!   重叠检测、一趟应用。

use unicode_normalization::UnicodeNormalization;

/// 换行风格(检测文件首个出现的换行)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineEnding {
    Crlf,
    Lf,
}

/// pi `detectLineEnding`:无 `\n` 或无 `\r\n` 都算 LF;否则先出现者为准。
pub fn detect_line_ending(content: &str) -> LineEnding {
    match (content.find("\r\n"), content.find('\n')) {
        (Some(crlf), Some(lf)) if crlf < lf => LineEnding::Crlf,
        _ => LineEnding::Lf,
    }
}

/// pi `normalizeToLF`:`\r\n` 与孤立 `\r` 都规范为 `\n`。
pub fn normalize_to_lf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// pi `restoreLineEndings`。
pub fn restore_line_endings(text: &str, ending: LineEnding) -> String {
    match ending {
        LineEnding::Crlf => text.replace('\n', "\r\n"),
        LineEnding::Lf => text.to_string(),
    }
}

/// pi `splitBom`:拆出前导 UTF-8 BOM,返回 (bom, text)。
/// 模型不会在 old_string 里带不可见 BOM,匹配必须先剥掉。
pub fn split_bom(content: &str) -> (&str, &str) {
    match content.strip_prefix('\u{FEFF}') {
        Some(rest) => ("\u{FEFF}", rest),
        None => ("", content),
    }
}

/// pi `normalizeForFuzzyMatch`:NFKC → 逐行去行尾空白 → 智能引号→ASCII、
/// Unicode 破折号→`-`、特殊空格→空格(码点表逐字照抄)。
pub fn normalize_for_fuzzy_match(text: &str) -> String {
    let nfkc: String = text.nfkc().collect();
    let trimmed: String = nfkc.split('\n').map(str::trim_end).collect::<Vec<_>>().join("\n");
    trimmed
        .chars()
        .map(|c| match c {
            // 智能单引号 → '
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            // 智能双引号 → "
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            // U+2010 连字符、U+2011 不换行连字符、U+2012 图形短线、
            // U+2013 en-dash、U+2014 em-dash、U+2015 水平线、U+2212 减号 → -
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => '-',
            // U+00A0 NBSP、U+2002-U+200A 各类空格、U+202F 窄 NBSP、
            // U+205F 中数学空格、U+3000 全角空格 → ' '
            '\u{00A0}' | '\u{2002}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => ' ',
            other => other,
        })
        .collect()
}

/// 一次目标替换(pi `fuzzyFindText` 的匹配部分)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuzzyMatchResult {
    pub found: bool,
    /// 匹配起点(字节偏移,落在 `used_fuzzy_match` 对应的内容空间里)。
    pub index: usize,
    pub match_length: usize,
    /// false = 精确命中(偏移基于原文);true = 模糊命中(偏移基于规范化空间)。
    pub used_fuzzy_match: bool,
}

/// pi `fuzzyFindText`:精确优先,失败后在双方规范化空间重试。
pub fn fuzzy_find_text(content: &str, old_text: &str) -> FuzzyMatchResult {
    // Try exact match first.
    if let Some(i) = content.find(old_text) {
        return FuzzyMatchResult {
            found: true,
            index: i,
            match_length: old_text.len(),
            used_fuzzy_match: false,
        };
    }
    // Fuzzy: work entirely in normalized space.
    let fuzzy_content = normalize_for_fuzzy_match(content);
    let fuzzy_old = normalize_for_fuzzy_match(old_text);
    match fuzzy_content.find(&fuzzy_old) {
        Some(i) => FuzzyMatchResult {
            found: true,
            index: i,
            match_length: fuzzy_old.len(),
            used_fuzzy_match: true,
        },
        None => FuzzyMatchResult { found: false, index: usize::MAX, match_length: 0, used_fuzzy_match: false },
    }
}

/// 一次编辑(new_string 保持调用方给的原样,不做规范化)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub old_string: String,
    pub new_string: String,
}

impl Edit {
    pub fn new(old: &str, new: &str) -> Self {
        Self { old_string: old.into(), new_string: new.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MatchedEdit {
    edit_index: usize,
    match_index: usize,
    match_length: usize,
}

/// pi `applyEditsToNormalizedContent` 的产物。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEdits {
    pub base_content: String,
    pub new_content: String,
    /// 替换组的行区间(LF 规范化空间,行 = split_inclusive('\n') 口径,0 基
    /// [start, end);相邻/重叠的替换合并为一组)。PLAN-042 details 的数据源。
    pub replaced_groups: Vec<ReplacedGroup>,
}

/// 一个替换组在新旧内容中的行区间(PLAN-042)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacedGroup {
    pub base_start: usize,
    pub base_end: usize,
    pub new_start: usize,
    pub new_end: usize,
}

// ── 五类自愈报错(pi 错误构造函数,文案语义照抄)─────────────────────

fn not_found_error(path: &str, edit_index: usize, total: usize) -> String {
    if total == 1 {
        format!(
            "Could not find the exact text in {path}. The old_string must match \
             exactly including all whitespace and newlines."
        )
    } else {
        format!(
            "Could not find edits[{edit_index}] in {path}. The old_string must \
             match exactly including all whitespace and newlines."
        )
    }
}

fn duplicate_error(path: &str, edit_index: usize, total: usize, occurrences: usize) -> String {
    if total == 1 {
        format!(
            "Found {occurrences} occurrences of the text in {path}. The text \
             must be unique. Please provide more context to make it unique."
        )
    } else {
        format!(
            "Found {occurrences} occurrences of edits[{edit_index}] in {path}. \
             Each old_string must be unique. Please provide more context to \
             make it unique."
        )
    }
}

fn empty_old_text_error(path: &str, edit_index: usize, total: usize) -> String {
    if total == 1 {
        format!("old_string must not be empty in {path}.")
    } else {
        format!("edits[{edit_index}].old_string must not be empty in {path}.")
    }
}

fn no_change_error(path: &str, total: usize) -> String {
    if total == 1 {
        format!(
            "No changes made to {path}. The replacement produced identical \
             content. This might indicate an issue with special characters \
             or the text not existing as expected."
        )
    } else {
        format!("No changes made to {path}. The replacements produced identical content.")
    }
}

fn overlap_error(path: &str, prev: usize, cur: usize) -> String {
    format!(
        "edits[{prev}] and edits[{cur}] overlap in {path}. Merge them into \
         one edit or target disjoint regions."
    )
}

/// 规范化空间中的出现次数(pi `countOccurrences`)。
fn count_occurrences(content: &str, old_text: &str) -> usize {
    let fuzzy_content = normalize_for_fuzzy_match(content);
    let fuzzy_old = normalize_for_fuzzy_match(old_text);
    fuzzy_content.matches(&fuzzy_old).count()
}

// ── 行级保留应用(pi `applyReplacementsPreservingUnchangedLines`)──────

/// 一行的字节区间 [start, end)(含换行符)。
struct LineSpan {
    start: usize,
    end: usize,
}

/// pi `splitLinesWithEndings` + `getLineSpans`:`[^\n]*\n|[^\n]+` 语义 =
/// Rust `split_inclusive('\n')`(末行无换行也成一段;空串为 0 段)。
fn line_spans(content: &str) -> Vec<LineSpan> {
    let mut spans = Vec::new();
    let mut off = 0usize;
    for line in content.split_inclusive('\n') {
        spans.push(LineSpan { start: off, end: off + line.len() });
        off += line.len();
    }
    spans
}

/// 替换触达的行区间(返回 [start_line, end_line) 半开区间)。
fn replacement_line_range(
    spans: &[LineSpan],
    match_index: usize,
    match_length: usize,
) -> Result<(usize, usize), String> {
    let rs = match_index;
    let re = match_index + match_length;
    let start_line = spans
        .iter()
        .position(|l| rs >= l.start && rs < l.end)
        .ok_or("Replacement range is outside the base content.")?;
    let mut end_line = start_line;
    while end_line < spans.len() && spans[end_line].end < re {
        end_line += 1;
    }
    if end_line >= spans.len() {
        return Err("Replacement range is outside the base content.".into());
    }
    Ok((start_line, end_line + 1))
}

/// 升序非重叠替换的一趟拼接(等价于 pi 的倒序改写)。
fn apply_replacements(content: &str, replacements: &[MatchedEdit], new_texts: &[String], offset: usize) -> String {
    let mut result = String::new();
    let mut last = 0usize;
    for r in replacements {
        let start = r.match_index - offset;
        let end = start + r.match_length;
        result.push_str(&content[last..start]);
        result.push_str(&new_texts[r.edit_index]);
        last = end;
    }
    result.push_str(&content[last..]);
    result
}

/// pi `applyEditsToNormalizedContent`:对 LF 规范化内容应用一批编辑。
///
/// 全部 edit 对同一份原始内容匹配(非增量);任一 edit 未命中/歧义/重叠、
/// old_string 为空、或应用后内容无变化,都返回 Err(String)(五类自愈报错,
/// 文案对齐 pi 错误构造函数的语义)。
pub fn apply_edits_to_normalized_content(
    normalized_content: &str,
    edits: &[Edit],
    path: &str,
) -> Result<AppliedEdits, String> {
    // 1. LF 规范化每个 edit(old/new 都规范)。
    let normalized_edits: Vec<Edit> = edits
        .iter()
        .map(|e| Edit {
            old_string: normalize_to_lf(&e.old_string),
            new_string: normalize_to_lf(&e.new_string),
        })
        .collect();
    let total = normalized_edits.len();

    // 2. 空 old_string 前置拒绝。
    for (i, e) in normalized_edits.iter().enumerate() {
        if e.old_string.is_empty() {
            return Err(empty_old_text_error(path, i, total));
        }
    }

    // 3. 第一趟匹配探测:任一 edit 需要模糊匹配 → 整体切到规范化空间。
    let used_fuzzy = normalized_edits
        .iter()
        .any(|e| fuzzy_find_text(normalized_content, &e.old_string).used_fuzzy_match);
    let replacement_base = if used_fuzzy {
        normalize_for_fuzzy_match(normalized_content)
    } else {
        normalized_content.to_string()
    };

    // 4. 逐 edit 匹配 + 歧义计数(规范化空间)。
    let mut matched: Vec<MatchedEdit> = Vec::with_capacity(total);
    for (i, e) in normalized_edits.iter().enumerate() {
        let m = fuzzy_find_text(&replacement_base, &e.old_string);
        if !m.found {
            return Err(not_found_error(path, i, total));
        }
        let occurrences = count_occurrences(&replacement_base, &e.old_string);
        if occurrences > 1 {
            return Err(duplicate_error(path, i, total, occurrences));
        }
        matched.push(MatchedEdit { edit_index: i, match_index: m.index, match_length: m.match_length });
    }

    // 5. 重叠检测(排序后相邻比较)。
    matched.sort_by_key(|m| m.match_index);
    for w in matched.windows(2) {
        let (prev, cur) = (w[0], w[1]);
        if prev.match_index + prev.match_length > cur.match_index {
            return Err(overlap_error(path, prev.edit_index, cur.edit_index));
        }
    }

    // 6. 替换组行区间（相邻/重叠合并）——模糊路径的行组重写与 PLAN-042 的
    //    diff 渲染共用同一分组。
    let new_texts: Vec<String> = normalized_edits.iter().map(|e| e.new_string.clone()).collect();
    let spans = line_spans(&replacement_base);
    let groups = group_replacements(&spans, &matched)?;

    // 每组新文本行数：组内替换应用于组切片（组边界 = 匹配区间并集，两条
    // 应用路径的组产物与此一致），据此换算新内容中的对应行区间。
    let mut replaced_groups: Vec<ReplacedGroup> = Vec::with_capacity(groups.len());
    let mut delta: isize = 0;
    for (s, e, reps) in &groups {
        let group_start = spans[*s].start;
        let group_end = spans[e - 1].end;
        let group_new = apply_replacements(
            &replacement_base[group_start..group_end],
            reps,
            &new_texts,
            group_start,
        );
        let new_len = if group_new.is_empty() { 0 } else { group_new.split_inclusive('\n').count() };
        let base_len = e - s;
        let new_start = (*s as isize + delta) as usize;
        replaced_groups.push(ReplacedGroup {
            base_start: *s,
            base_end: *e,
            new_start,
            new_end: new_start + new_len,
        });
        delta += new_len as isize - base_len as isize;
    }

    // 7. 应用:模糊 → 行级保留;纯精确 → 直接一趟拼接。
    let new_content = if used_fuzzy {
        apply_replacements_preserving_unchanged_lines(
            normalized_content,
            &replacement_base,
            &groups,
            &new_texts,
        )?
    } else {
        apply_replacements(&replacement_base, &matched, &new_texts, 0)
    };

    // 8. 无变化拒绝。
    if normalized_content == new_content {
        return Err(no_change_error(path, total));
    }

    Ok(AppliedEdits {
        base_content: normalized_content.to_string(),
        new_content,
        replaced_groups,
    })
}

/// 按触达行分组合并（相邻/重叠的替换进同一组），返回 (start, end, 组内替换)。
fn group_replacements(
    spans: &[LineSpan],
    replacements: &[MatchedEdit],
) -> Result<Vec<(usize, usize, Vec<MatchedEdit>)>, String> {
    let mut sorted: Vec<MatchedEdit> = replacements.to_vec();
    sorted.sort_by_key(|r| r.match_index);
    let mut groups: Vec<(usize, usize, Vec<MatchedEdit>)> = Vec::new();
    for r in sorted {
        let range = replacement_line_range(spans, r.match_index, r.match_length)?;
        if let Some(cur) = groups.last_mut() {
            if range.0 < cur.1 {
                cur.1 = cur.1.max(range.1);
                cur.2.push(r);
                continue;
            }
        }
        groups.push((range.0, range.1, vec![r]));
    }
    Ok(groups)
}

/// pi `applyReplacementsPreservingUnchangedLines`:以行组为单位应用替换,
/// 未触达行从 original 原样复制,触达行组从 base(规范化空间)重写。
fn apply_replacements_preserving_unchanged_lines(
    original_content: &str,
    base_content: &str,
    groups: &[(usize, usize, Vec<MatchedEdit>)],
    new_texts: &[String],
) -> Result<String, String> {
    let original_lines: Vec<&str> = original_content.split_inclusive('\n').collect();
    let base_spans = line_spans(base_content);
    if original_lines.len() != base_spans.len() {
        return Err(
            "Cannot preserve unchanged lines because the base content has a \
             different line count."
                .into(),
        );
    }

    let mut result = String::new();
    let mut orig_idx = 0usize;
    for (start_line, end_line, reps) in groups.iter() {
        // 未触达行:原始字节原样复制。
        result.push_str(&original_lines[orig_idx..*start_line].concat());
        // 触达行组:从规范化 base 切片重写(替换偏移相对组起点)。
        let group_start = base_spans[*start_line].start;
        let group_end = base_spans[end_line - 1].end;
        result.push_str(&apply_replacements(
            &base_content[group_start..group_end],
            reps,
            new_texts,
            group_start,
        ));
        orig_idx = *end_line;
    }
    result.push_str(&original_lines[orig_idx..].concat());
    Ok(result)
}

// ── PLAN-042:edit details（diff 展示 + unified patch）──────────────────

/// edit_file 的 details 载荷（语义对齐 pi `EditToolDetails`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditDetails {
    /// 行号标注的展示 diff（±context 行上下文；格式 `+N line`/`-N line`/
    /// ` N line`，省略行以 ` N ...` 标记——pi `generateDiffString` 同款）。
    pub diff: String,
    /// 标准 unified diff（`--- a/path` / `+++ b/path` + `@@` hunk）。
    pub patch: String,
    /// 新文件中首个改动行的行号（1 基，编辑器跳转用；无替换组时 None）。
    pub first_changed_line: Option<usize>,
}

/// 由 [`AppliedEdits::replaced_groups`] 生成展示 diff 与 unified patch。
/// base/new 均为 LF 规范化空间（与 apply 相同），path 仅入 patch 头。
pub fn generate_edit_diff(
    path: &str,
    base: &str,
    new: &str,
    groups: &[ReplacedGroup],
    context: usize,
) -> EditDetails {
    let mut base_lines: Vec<&str> = base.split('\n').collect();
    if base_lines.last() == Some(&"") {
        base_lines.pop();
    }
    let mut new_lines: Vec<&str> = new.split('\n').collect();
    if new_lines.last() == Some(&"") {
        new_lines.pop();
    }
    let width = base_lines.len().max(new_lines.len()).to_string().len();
    let pad = |n: usize| format!("{n:>width$}");

    // ── 展示 diff（pi generateDiffString 的行号标注格式）─────────────
    let mut out: Vec<String> = Vec::new();
    let mut shown_until = 0usize; // base 行 [0, shown_until) 已输出
    for (i, g) in groups.iter().enumerate() {
        let next_start = groups.get(i + 1).map(|n| n.base_start).unwrap_or(base_lines.len());
        let ctx_start = g.base_start.saturating_sub(context).max(shown_until);
        if ctx_start > shown_until && !out.is_empty() {
            out.push(format!("{} ...", " ".repeat(width + 1)));
        }
        for ln in ctx_start..g.base_start {
            out.push(format!(" {} {}", pad(ln + 1), base_lines[ln]));
        }
        for ln in g.base_start..g.base_end {
            out.push(format!("-{} {}", pad(ln + 1), base_lines[ln]));
        }
        for ln in g.new_start..g.new_end {
            out.push(format!("+{} {}", pad(ln + 1), new_lines[ln]));
        }
        let ctx_end = (g.base_end + context).min(next_start);
        for ln in g.base_end..ctx_end {
            out.push(format!(" {} {}", pad(ln + 1), base_lines[ln]));
        }
        shown_until = ctx_end;
    }

    // ── unified patch（逐组一个 hunk；组间上下文经 next_start 裁剪不重叠）──
    fn trim(s: &str) -> &str {
        s.trim_end_matches('\n')
    }
    let mut patch = format!("--- a/{path}\n+++ b/{path}\n");
    for (i, g) in groups.iter().enumerate() {
        let next_start = groups.get(i + 1).map(|n| n.base_start).unwrap_or(base_lines.len());
        let ctx_start = g.base_start.saturating_sub(context);
        let ctx_end = (g.base_end + context).min(next_start);
        let old_count = ctx_end - ctx_start;
        let new_count =
            old_count - (g.base_end - g.base_start) + (g.new_end - g.new_start);
        // 空区间(计数 0)的标准写法是"位于其后"，即 1 基起点为 ctx_start 本身。
        let old_start = if old_count == 0 { ctx_start } else { ctx_start + 1 };
        let new_start = if new_count == 0 { g.new_start } else { g.new_start - (g.base_start - ctx_start) + 1 };
        patch.push_str(&format!("@@ -{old_start},{old_count} +{new_start},{new_count} @@\n"));
        for ln in ctx_start..g.base_start {
            patch.push_str(&format!(" {}\n", trim(base_lines[ln])));
        }
        for ln in g.base_start..g.base_end {
            patch.push_str(&format!("-{}\n", trim(base_lines[ln])));
        }
        for ln in g.new_start..g.new_end {
            patch.push_str(&format!("+{}\n", trim(new_lines[ln])));
        }
        for ln in g.base_end..ctx_end {
            patch.push_str(&format!(" {}\n", trim(base_lines[ln])));
        }
    }

    EditDetails {
        diff: out.join("\n"),
        patch,
        first_changed_line: groups.first().map(|g| g.new_start + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 行尾/BOM 往返 ───────────────────────────────────────────────

    #[test]
    fn detect_line_ending_first_occurrence_wins() {
        assert_eq!(detect_line_ending("a\r\nb"), LineEnding::Crlf);
        assert_eq!(detect_line_ending("a\nb\r\n"), LineEnding::Lf);
        assert_eq!(detect_line_ending("no newline"), LineEnding::Lf);
        assert_eq!(detect_line_ending(""), LineEnding::Lf);
    }

    #[test]
    fn normalize_and_restore_roundtrip() {
        let raw = "a\r\nb\rc\n";
        let lf = normalize_to_lf(raw);
        assert_eq!(lf, "a\nb\nc\n");
        assert_eq!(restore_line_endings(&lf, LineEnding::Crlf), "a\r\nb\r\nc\r\n");
        assert_eq!(restore_line_endings(&lf, LineEnding::Lf), lf);
    }

    #[test]
    fn split_bom_strips_leading_feff() {
        assert_eq!(split_bom("\u{FEFF}abc"), ("\u{FEFF}", "abc"));
        assert_eq!(split_bom("abc"), ("", "abc"));
        // BOM 只在开头算
        assert_eq!(split_bom("a\u{FEFF}b"), ("", "a\u{FEFF}b"));
    }

    // ── 模糊规范化表 ────────────────────────────────────────────────

    #[test]
    fn fuzzy_norm_smart_quotes() {
        assert_eq!(normalize_for_fuzzy_match("‘a’ ‚b‛"), "'a' 'b'");
        assert_eq!(normalize_for_fuzzy_match("“x” „y‟"), "\"x\" \"y\"");
    }

    #[test]
    fn fuzzy_norm_dashes() {
        // U+2010/2011/2012/2013/2014/2015/2212 → '-'
        assert_eq!(normalize_for_fuzzy_match("a‐b‑c‒d–e—f−g"), "a-b-c-d-e-f-g");
    }

    #[test]
    fn fuzzy_norm_special_spaces() {
        // NBSP + U+2002..200A +窄/中/全角空格 → ' '
        assert_eq!(normalize_for_fuzzy_match("a\u{00A0}b\u{2003}c\u{3000}d"), "a b c d");
    }

    #[test]
    fn fuzzy_norm_trailing_whitespace_stripped_per_line() {
        // pi:split("\n") 逐段 trimEnd 再 join——末尾换行产生的空段保留,
        // 所以结果是 "x\ny\n"(与 JS join 语义一致)。
        assert_eq!(normalize_for_fuzzy_match("x  \ny\t\n"), "x\ny\n");
        // 行中空白不动
        assert_eq!(normalize_for_fuzzy_match("a b"), "a b");
    }

    #[test]
    fn fuzzy_norm_nfkc_fullwidth() {
        // NFKC:全角字母 → 半角(码点表之外的 NFKC 通例)
        assert_eq!(normalize_for_fuzzy_match("ａｂｃ"), "abc");
    }

    // ── fuzzy_find_text ─────────────────────────────────────────────

    #[test]
    fn fuzzy_find_exact_first() {
        let r = fuzzy_find_text("hello world", "world");
        assert!(r.found && !r.used_fuzzy_match);
        assert_eq!(&"hello world"[r.index..r.index + r.match_length], "world");
    }

    #[test]
    fn fuzzy_find_falls_back_on_smart_quotes() {
        let r = fuzzy_find_text("he said “hi”", "he said \"hi\"");
        assert!(r.found && r.used_fuzzy_match);
    }

    #[test]
    fn fuzzy_find_not_found() {
        let r = fuzzy_find_text("abc", "zzz");
        assert!(!r.found);
    }

    // ── apply_edits:五类报错 ────────────────────────────────────────

    #[test]
    fn apply_edits_not_found_error_single_and_indexed() {
        let e1 = apply_edits_to_normalized_content("abc", &[Edit::new("zzz", "x")], "f.txt").unwrap_err();
        assert!(e1.contains("Could not find the exact text in f.txt"), "{e1}");
        let e2 = apply_edits_to_normalized_content(
            "abc",
            &[Edit::new("abc", "x"), Edit::new("zzz", "y")],
            "f.txt",
        )
        .unwrap_err();
        assert!(e2.contains("edits[1]"), "{e2}");
    }

    #[test]
    fn apply_edits_duplicate_error_mentions_occurrences() {
        let e = apply_edits_to_normalized_content("dup\ndup\n", &[Edit::new("dup", "x")], "f.txt").unwrap_err();
        assert!(e.contains("Found 2 occurrences"), "{e}");
        assert!(e.contains("must be unique"), "{e}");
    }

    #[test]
    fn apply_edits_empty_old_error() {
        let e = apply_edits_to_normalized_content("abc", &[Edit::new("", "x")], "f.txt").unwrap_err();
        assert!(e.contains("must not be empty"), "{e}");
    }

    #[test]
    fn apply_edits_no_change_error() {
        let e = apply_edits_to_normalized_content("abc", &[Edit::new("abc", "abc")], "f.txt").unwrap_err();
        assert!(e.contains("No changes made"), "{e}");
        assert!(e.contains("identical content"), "{e}");
    }

    #[test]
    fn apply_edits_overlap_error() {
        // "abcdef":edit0 命中 [1..4],edit1 命中 [3..5] → 重叠。
        let e = apply_edits_to_normalized_content(
            "abcdef",
            &[Edit::new("bcd", "X"), Edit::new("def", "Y")],
            "f.txt",
        )
        .unwrap_err();
        assert!(e.contains("overlap"), "{e}");
        assert!(e.contains("edits[0] and edits[1]"), "{e}");
    }

    // ── apply_edits:正常路径 ────────────────────────────────────────

    #[test]
    fn apply_edits_exact_multi_disjoint() {
        let r = apply_edits_to_normalized_content(
            "aaa\nbbb\nccc\n",
            &[Edit::new("aaa", "AAA"), Edit::new("ccc", "CCC")],
            "f.txt",
        )
        .unwrap();
        assert_eq!(r.base_content, "aaa\nbbb\nccc\n");
        assert_eq!(r.new_content, "AAA\nbbb\nCCC\n");
    }

    /// 模糊命中 + 行级保留:未触达行保留原始字节(含行尾空白),
    /// 触达行从规范化空间重写。
    #[test]
    fn apply_edits_fuzzy_preserves_untouched_line_bytes() {
        // 原文行 2 带行尾空格 + 智能引号;old 只在规范化空间命中行 2。
        let original = "keep \n“q”  \nkeep2";
        let r = apply_edits_to_normalized_content(original, &[Edit::new("\"q\"", "\"Q\"")], "f.txt").unwrap();
        // 未触达行原样:行 1 的行尾空格、行 3 的字节。
        assert_eq!(r.new_content, "keep \n\"Q\"\nkeep2");
        // base 即 LF 规范化原文(未再动)。
        assert_eq!(r.base_content, original);
    }

    /// 同一行被两个编辑触达 → 合并为一个行组,一趟应用。
    #[test]
    fn apply_edits_fuzzy_two_edits_same_line_group() {
        let original = "a “x” b ‘y’ c";
        let r = apply_edits_to_normalized_content(
            original,
            &[Edit::new("\"x\"", "[x]"), Edit::new("'y'", "[y]")],
            "f.txt",
        )
        .unwrap();
        assert_eq!(r.new_content, "a [x] b [y] c");
    }

    /// 全模糊内容 + 多行编辑:仅触达行组重写,别行保留原始行尾空白。
    #[test]
    fn apply_edits_fuzzy_multiline_edit() {
        let original = "one  \ntwo “two”\nthree\t\n";
        let r = apply_edits_to_normalized_content(
            original,
            &[Edit::new("two \"two\"\nthree", "TWO\nTHREE")],
            "f.txt",
        )
        .unwrap();
        // 行 1 未触达:原始 "one  \n";行 2-3 触达:规范化空间重写。
        assert_eq!(r.new_content, "one  \nTWO\nTHREE\n");
    }

    /// 歧义计数在规范化空间进行:原文两次、规范化后仍两次 → 拒绝。
    #[test]
    fn apply_edits_duplicate_counted_in_normalized_space() {
        let e = apply_edits_to_normalized_content(
            "“a”\n“a”\n",
            &[Edit::new("\"a\"", "x")],
            "f.txt",
        )
        .unwrap_err();
        assert!(e.contains("2 occurrences"), "{e}");
    }

    // ── PLAN-042:替换组区间 + diff/patch 生成 ────────────────────────

    #[test]
    fn replaced_groups_single_edit_ranges() {
        let r = apply_edits_to_normalized_content(
            "one\ntwo\nthree\n",
            &[Edit::new("two", "TWO")],
            "f.txt",
        )
        .unwrap();
        assert_eq!(
            r.replaced_groups,
            vec![ReplacedGroup { base_start: 1, base_end: 2, new_start: 1, new_end: 2 }]
        );
        let d = generate_edit_diff("f.txt", &r.base_content, &r.new_content, &r.replaced_groups, 4);
        assert_eq!(d.diff, " 1 one\n-2 two\n+2 TWO\n 3 three");
        assert_eq!(d.first_changed_line, Some(2));
        assert!(d.patch.contains("--- a/f.txt"), "{}", d.patch);
        assert!(d.patch.contains("@@ -1,3 +1,3 @@"), "{}", d.patch);
        assert!(d.patch.contains("-two\n+TWO"), "{}", d.patch);
    }

    #[test]
    fn replaced_groups_multiline_insert_shifts_later_groups() {
        // 前组扩一行、后组行号随之 +1;纯精确路径(无模糊)。
        let base = "a\nb\nc\nd\ne\nf\ng\n";
        let r = apply_edits_to_normalized_content(
            base,
            &[Edit::new("b", "B1\nB2"), Edit::new("f", "F")],
            "f.txt",
        )
        .unwrap();
        assert_eq!(
            r.replaced_groups,
            vec![
                ReplacedGroup { base_start: 1, base_end: 2, new_start: 1, new_end: 3 },
                ReplacedGroup { base_start: 5, base_end: 6, new_start: 6, new_end: 7 },
            ]
        );
        let d = generate_edit_diff("f.txt", &r.base_content, &r.new_content, &r.replaced_groups, 1);
        // 上下文 1:两组间有省略号;后组 +行号 = 新内容行号(7)。
        assert!(
            d.diff.lines().any(|l| l.starts_with('+') && l.contains('F')),
            "{}",
            d.diff
        );
        assert_eq!(d.first_changed_line, Some(2));
    }

    #[test]
    fn generate_diff_distant_edits_show_ellipsis() {
        let base: String = (1..=20).map(|i| format!("line{i}\n")).collect();
        let r = apply_edits_to_normalized_content(
            &base,
            &[Edit::new("line2
", "LINE2
"), Edit::new("line18
", "LINE18
")],
            "f.txt",
        )
        .unwrap();
        let d = generate_edit_diff("f.txt", &r.base_content, &r.new_content, &r.replaced_groups, 4);
        assert!(d.diff.contains(" ..."), "ellipsis between distant groups: {}", d.diff);
        assert!(d.diff.starts_with("  1 line1"), "{}", d.diff);
        assert!(d.diff.ends_with(" 20 line20"), "{}", d.diff);
        // patch 两 hunk,头尾行号正确。
        assert_eq!(d.patch.matches("@@ -").count(), 2, "{}", d.patch);
        assert!(d.patch.contains("@@ -1,6 +1,6 @@"), "{}", d.patch);
        assert!(d.patch.contains("@@ -14,7 +14,7 @@"), "{}", d.patch);
    }

    #[test]
    fn generate_diff_deletion_group() {
        let r = apply_edits_to_normalized_content(
            "keep\ndrop\nkeep2\n",
            &[Edit::new("drop\n", "")],
            "f.txt",
        )
        .unwrap();
        let d = generate_edit_diff("f.txt", &r.base_content, &r.new_content, &r.replaced_groups, 4);
        assert!(d.diff.contains("-2 drop"), "{}", d.diff);
        assert_eq!(d.first_changed_line, Some(2));
        assert!(d.patch.contains("@@ -1,3 +1,2 @@"), "{}", d.patch);
    }

    #[test]
    fn generate_diff_no_trailing_newline() {
        let r = apply_edits_to_normalized_content(
            "x\ny",
            &[Edit::new("y", "z")],
            "f.txt",
        )
        .unwrap();
        let d = generate_edit_diff("f.txt", &r.base_content, &r.new_content, &r.replaced_groups, 4);
        assert_eq!(d.diff, " 1 x\n-2 y\n+2 z");
    }
}
