//! PLAN-088 T-03: 锚点索引 —— vtree Atom 解析 + 命中测试 + 实质回溯。
//!
//! 数据源 = AutoUI MCP `autoui_vtree`（include_source 默认 true）的 Atom 文本。
//! 序列化契约（auto-lang `ui/vtree_atom.rs` + `auto-val Node Display`，钉位
//! 8fecfcf69 核实）：节点头 `<kind> vnode_<n> {`；props 与 children 同层、
//! 以 `; ` 分隔；值形态 = `"str"`（裸引号、无转义）/ `{ k: v; ... }` 对象 /
//! 数 / 布尔 / `[a, b]` 数组。元数据全部是 props（bbox/style/class/events/
//! source/for_iter），children 严格 1:1。
//!
//! 命中语义（§10-1 定案）：deepest-first（含点最小面积，auto-lang
//! ui/debug/hit_test.rs 同款）→ 命中节点无 source 且无 events 时沿 parent
//! 链上溯至最近"实质节点"。返回恒含 ancestor_chain（root→…→命中）。

use std::collections::HashMap;

use serde_json::{json, Value};

/// border-box（窗口逻辑像素；与帧 PNG 1:1 为 T-02 实测契约）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone)]
pub struct ForCtx {
    pub var_name: String,
    pub index: Option<usize>,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub vnode: u64,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
    pub kind: String,
    /// 展示标签：content → label → placeholder → value 首个非空。
    pub label: Option<String>,
    pub bbox: Option<Rect>,
    /// 事件名集合（handler 名不进索引；实质判定只看有无）。
    pub events: Vec<String>,
    /// 源码 span（.at 文件字节偏移；T-02B 填充臂；行号换算见 resolve_source）。
    pub span: Option<(usize, usize)>,
    /// for 循环上下文（实例区分，AC-02）。
    pub for_ctx: Option<ForCtx>,
    pub depth: usize,
}

impl NodeInfo {
    /// §10-1 实质节点：有源码 span 或有事件。
    fn substantive(&self) -> bool {
        self.span.is_some() || !self.events.is_empty()
    }
}

/// 一帧的锚点索引（文档序节点表 + id 反查）。
#[derive(Debug, Clone, Default)]
pub struct AnchorIndex {
    pub nodes: Vec<NodeInfo>,
    by_id: HashMap<u64, usize>,
    /// 构建时的帧序号（overlay/picked 失效判定用）。
    pub seq: u64,
}

impl AnchorIndex {
    pub fn get(&self, vnode: u64) -> Option<&NodeInfo> {
        self.by_id.get(&vnode).map(|&i| &self.nodes[i])
    }

    /// deepest-first 命中：含点节点中面积最小者（hit_test.rs 同款，含边界）。
    pub fn hit_test(&self, x: f32, y: f32) -> Option<u64> {
        let mut best: Option<(u64, f32)> = None;
        for n in &self.nodes {
            let Some(r) = n.bbox else { continue };
            // 含边界（iced 实测 border-box；hit_test.rs edge-exactly-on-boundary 同口径）。
            if x < r.x || y < r.y || x > r.x + r.w || y > r.y + r.h {
                continue;
            }
            let area = r.w * r.h;
            match best {
                Some((_, a)) if area >= a => {}
                _ => best = Some((n.vnode, area)),
            }
        }
        best.map(|(id, _)| id)
    }

    /// §10-1 实质回溯：从命中节点沿 parent 链上溯至最近实质节点（含自身）。
    /// 全链无实质 → 返回命中节点本身（层树仍可定位）。
    pub fn substantive_anchor(&self, vnode: u64) -> u64 {
        let mut cur = Some(vnode);
        while let Some(id) = cur {
            let Some(n) = self.get(id) else { break };
            if n.substantive() {
                return id;
            }
            cur = n.parent;
        }
        vnode
    }

    /// root→vnode 祖先链（供前端层树定位与语义再选择）。
    pub fn ancestor_chain(&self, vnode: u64) -> Vec<u64> {
        let mut chain = Vec::new();
        let mut cur = Some(vnode);
        while let Some(id) = cur {
            chain.push(id);
            cur = self.get(id).and_then(|n| n.parent);
        }
        chain.reverse();
        chain
    }

    /// for 实例标注（AC-02"第 N 项"）：节点自身 for_ctx.index，缺则沿链
    /// 找最近祖先的（循环体容器可能整体携带 for_context）。
    pub fn for_context_of(&self, vnode: u64) -> Option<&ForCtx> {
        let mut cur = Some(vnode);
        while let Some(id) = cur {
            let Some(n) = self.get(id) else { break };
            if n.for_ctx.is_some() {
                return n.for_ctx.as_ref();
            }
            cur = n.parent;
        }
        None
    }

    /// 层树载荷（AC-05）：扁平文档序表 [{id, depth, kind, label, source,
    /// source_path, source_line, source_confidence, for_index}]——.at 无递归
    /// 渲染面，前端按 depth 缩进平铺（全量；折叠由前端展示层做）。id 用
    /// `"vnode_N"` 字符串（哈希 id > JS 2^53 精度，数字形态会静默截断）。
    /// source 三件 = span 经 `resolver` 换算（display "path:line" + 拆分
    /// 字段 + Q-04 置信度；.at 侧无 split 能力）。未解析则缺省。
    pub fn tree_flat_json(
        &self,
        resolver: &dyn Fn(usize, usize) -> Option<(String, usize, &'static str)>,
    ) -> Value {
        let items: Vec<Value> = self
            .nodes
            .iter()
            .map(|n| {
                let mut o = json!({
                    "id": format!("vnode_{}", n.vnode),
                    "depth": n.depth,
                    "kind": n.kind,
                });
                if let Some(l) = &n.label {
                    o["label"] = json!(l);
                }
                if let Some((off, len)) = n.span {
                    if let Some((p, line, conf)) = resolver(off, len) {
                        o["source"] = json!(format!("{p}:{line}"));
                        o["source_path"] = json!(p);
                        o["source_line"] = json!(line);
                        o["source_confidence"] = json!(conf);
                    }
                }
                if let Some(f) = &n.for_ctx {
                    if let Some(i) = f.index {
                        o["loop_index"] = json!(i);
                    }
                    o["loop_value"] = json!(f.value);
                }
                o
            })
            .collect();
        Value::Array(items)
    }

    /// pick 锚点载荷（AC-01/03 同构面）。`scale` = 帧像素/逻辑像素（T-02
    /// 契约：bbox 为逻辑坐标）；`frame_w/h` = 帧 PNG 像素尺寸（bbox_px 与
    /// bbox_pct 的换算基准——pct = 像素/帧尺寸×100，前端百分比定位直吃）。
    /// `resolver` 把 span 换算 (path, line, confidence)（尽力；未解析缺省，
    /// 非门）。vnode_id 字符串形态（>2^53 哈希，JSON number 静默截断）。
    pub fn pick_json(
        &self,
        vnode: u64,
        scale: f32,
        frame_w: u32,
        frame_h: u32,
        resolver: &dyn Fn(usize, usize) -> Option<(String, usize, &'static str)>,
    ) -> Option<Value> {
        let n = self.get(vnode)?;
        let chain: Vec<String> = self
            .ancestor_chain(vnode)
            .into_iter()
            .map(|v| format!("vnode_{v}"))
            .collect();
        let for_ctx = self.for_context_of(vnode);
        let mut o = json!({
            "vnode_id": format!("vnode_{}", vnode),
            "kind": n.kind,
            "ancestor_chain": chain,
        });
        if let Some(l) = &n.label {
            o["label"] = json!(l);
        }
        if let Some(r) = n.bbox {
            let (fw, fh) = (frame_w.max(1) as f32, frame_h.max(1) as f32);
            let (px, py, pw, ph) = (r.x * scale, r.y * scale, r.w * scale, r.h * scale);
            o["bbox"] = json!({ "x": r.x, "y": r.y, "w": r.w, "h": r.h });
            o["bbox_px"] = json!({ "x": px, "y": py, "w": pw, "h": ph });
            o["bbox_pct"] = json!({
                "x": px / fw * 100.0, "y": py / fh * 100.0,
                "w": pw / fw * 100.0, "h": ph / fh * 100.0,
            });
        }
        if let Some((off, len)) = n.span {
            if let Some((p, line, conf)) = resolver(off, len) {
                o["source"] = json!(format!("{p}:{line}"));
                o["source_path"] = json!(p);
                o["source_line"] = json!(line);
                // PLAN-093 Q-04：多文件启发式来源必须可辨认（前端显示
                // "来源待确认"）；单文件 exact，不假装已准确锚定。
                o["source_confidence"] = json!(conf);
            }
        }
                if let Some(f) = for_ctx {
                    // var_name = "var" 的关键字安全副本（.at 字段名撞硬关键字
                    // ——fc.var 不可读；消费方统一走 var_name）。
                    o["forctx"] = json!({ "var": f.var_name, "var_name": f.var_name, "index": f.index, "value": f.value });
                }
        Some(o)
    }

    /// span → `(相对路径, 行号, 置信度)`（尽力）。文件判定：候选 = app 下
    /// src 前后台 .at（有界集）；偏移落在文件长度内者中，择切片含 kind
    /// 关键词的，缺判据取唯一命中。M2 主路径 = 单文件 app（多文件启发，
    /// 登记契约记录）。
    /// PLAN-093 Q-04（§5.6）：置信度两档——`exact` = 单候选文件（唯一
    /// 命中，硬验收面：单文件有效来源行定位准确）；`uncertain` = 多候选
    /// 文件的启发式命中（无论 kind 关键词命中与否——多文件归定位不准，
    /// 前端必须显示"来源待确认"，不得假装已准确锚定）。
    pub fn resolve_source(
        app_dir: &std::path::Path,
        span: (usize, usize),
        kind: &str,
    ) -> Option<(String, usize, &'static str)> {
        let (off, len) = span;
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        for sub in ["src/front", "src/back", "src", ""] {
            let dir = if sub.is_empty() { app_dir.to_path_buf() } else { app_dir.join(sub) };
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|e| e.to_str()) == Some("at") {
                    // Q-04：pac.at 是包清单（根目录），永远不是 widget span
                    // 的来源——计入候选会让所有真实 app（根清单+src 源码）
                    // 恒为多候选 uncertain，单文件硬验收面失真。
                    if p.file_name().and_then(|n| n.to_str()) == Some("pac.at") {
                        continue;
                    }
                    candidates.push(p);
                }
            }
        }
        candidates.sort();
        let confidence: &'static str = if candidates.len() <= 1 { "exact" } else { "uncertain" };
        let mut fallback: Option<(String, usize)> = None;
        for p in candidates {
            let Ok(bytes) = std::fs::read(&p) else { continue };
            if off + len > bytes.len() {
                continue;
            }
            let rel = p.strip_prefix(app_dir).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            let line = 1 + bytes[..off].iter().filter(|&&b| b == b'\n').count();
            let slice = String::from_utf8_lossy(&bytes[off..(off + len).min(bytes.len())]);
            if slice.to_lowercase().contains(&kind.to_lowercase()) {
                return Some((rel, line, confidence));
            }
            if fallback.is_none() {
                fallback = Some((rel, line));
            }
        }
        fallback.map(|(p, l)| (p, l, confidence))
    }

    /// 重建索引（帧循环每拍调用；解析失败返回 Err，调用方保留旧索引）。
    pub fn parse(atom: &str, seq: u64) -> Result<AnchorIndex, String> {
        let mut p = Parser { b: atom.as_bytes(), pos: 0 };
        let nodes = p.parse_forest()?;
        let mut idx = AnchorIndex { by_id: HashMap::with_capacity(nodes.len()), nodes, seq };
        for (i, n) in idx.nodes.iter().enumerate() {
            if idx.by_id.insert(n.vnode, i).is_some() {
                return Err(format!("anchor: duplicate vnode_{} in vtree", n.vnode));
            }
        }
        Ok(idx)
    }
}

// ── Atom 解析器 ─────────────────────────────────────────────────────────────

struct Parser<'a> {
    b: &'a [u8],
    pos: usize,
}

/// 解析中的节点（parent/children 链后处理）。
struct RawNode {
    vnode: u64,
    kind: String,
    parent: Option<u64>,
    children: Vec<u64>,
    label: Option<String>,
    bbox: Option<Rect>,
    events: Vec<String>,
    span: Option<(usize, usize)>,
    for_ctx: Option<ForCtx>,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.pos).copied()
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\n') | Some(b'\r') | Some(b'\t')) {
            self.pos += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// 主入口：解析到输入耗尽。栈顶闭括号即收栈（Display 括号配对）。
    fn parse_forest(&mut self) -> Result<Vec<NodeInfo>, String> {
        let mut raw: Vec<RawNode> = Vec::new();
        let mut stack: Vec<usize> = Vec::new(); // raw 下标栈
        loop {
            self.skip_ws();
            let Some(c) = self.peek() else { break };
            if c == b'}' {
                self.pos += 1;
                stack.pop();
                continue;
            }
            if let Some((vnode, kind)) = self.try_node_header() {
                let parent: Option<u64> = stack.last().map(|&pi| raw[pi].vnode);
                if let Some(pi) = stack.last().copied() {
                    raw[pi].children.push(vnode);
                }
                let depth = stack.last().map(|&pi| raw[pi].depth + 1).unwrap_or(0);
                raw.push(RawNode {
                    vnode,
                    kind,
                    parent,
                    children: Vec::new(),
                    label: None,
                    bbox: None,
                    events: Vec::new(),
                    span: None,
                    for_ctx: None,
                    depth,
                });
                stack.push(raw.len() - 1);
                self.skip_ws();
                if !self.eat(b'{') {
                    return Err(format!("anchor: node vnode_{vnode} missing '{{' at byte {}", self.pos));
                }
                continue;
            }
            // prop：当前栈顶节点的 `key: value`。
            let Some(&top) = stack.last() else {
                return Err(format!("anchor: prop outside node at byte {}", self.pos));
            };
            self.parse_prop_into(&mut raw[top])?;
        }
        if !stack.is_empty() {
            return Err("anchor: unbalanced braces in vtree atom".to_string());
        }
        Ok(raw
            .into_iter()
            .map(|r| NodeInfo {
                vnode: r.vnode,
                parent: r.parent,
                children: r.children,
                kind: r.kind,
                label: r.label,
                bbox: r.bbox,
                events: r.events,
                span: r.span,
                for_ctx: r.for_ctx,
                depth: r.depth,
            })
            .collect())
    }

    /// 节点头探测：`<kind> vnode_<n>`（kind = [_a-z]+，与 prop 的区别在于
    /// 后随 ` vnode_N`）。命中则消费并返回；否则不动 pos 返回 None。
    fn try_node_header(&mut self) -> Option<(u64, String)> {
        let start = self.pos;
        let mut i = self.pos;
        while i < self.b.len() && (self.b[i].is_ascii_lowercase() || self.b[i] == b'_') {
            i += 1;
        }
        if i == start {
            return None;
        }
        let kind = std::str::from_utf8(&self.b[start..i]).ok()?.to_string();
        // 后随 ` vnode_<digits>`
        let mut j = i;
        if j >= self.b.len() || self.b[j] != b' ' {
            return None;
        }
        j += 1;
        if !self.b[j..].starts_with(b"vnode_") {
            return None;
        }
        j += "vnode_".len();
        let ds = j;
        while j < self.b.len() && self.b[j].is_ascii_digit() {
            j += 1;
        }
        if j == ds {
            return None;
        }
        // vnode_N 后必须是空格+'{'（Display 恒有体）或行尾——防 content 字符串
        // 内碰巧含 "xxx vnode_3" 文本被误认（后随非 '{' 即拒）。
        let vnode: u64 = std::str::from_utf8(&self.b[ds..j]).ok()?.parse().ok()?;
        let after = self.b.get(j).copied();
        if !matches!(after, Some(b' ') | Some(b'{')) {
            return None;
        }
        self.pos = j;
        Some((vnode, kind))
    }

    /// `key: value` 解析并入槽位。值 = 字符串/对象/标量/数组（引号感知的
    /// 平衡扫描；字符串内嵌引号/花括号的病态串按降级接受——Display 无转义）。
    fn parse_prop_into(&mut self, node: &mut RawNode) -> Result<(), String> {
        let ks = self.pos;
        while let Some(c) = self.peek() {
            if c == b':' || c == b'}' || c == b';' || c == b',' {
                break;
            }
            self.pos += 1;
        }
        let key = std::str::from_utf8(&self.b[ks..self.pos]).unwrap_or("").trim().to_string();
        if !self.eat(b':') {
            // 无法识别的散置 token（防御）：吞到最近的 ; 或 }。
            while let Some(c) = self.peek() {
                if c == b';' {
                    self.pos += 1;
                    break;
                }
                if c == b'}' {
                    break;
                }
                self.pos += 1;
            }
            return Ok(());
        }
        self.skip_ws();
        let val = self.parse_value();
        match (key.as_str(), val) {
            ("label", V::Str(s)) => node.label = Some(s),
            ("content", V::Str(s)) if node.label.is_none() => node.label = Some(s),
            ("placeholder", V::Str(s)) if node.label.is_none() => node.label = Some(s),
            ("bbox", V::Obj(o)) => node.bbox = rect_of(&o),
            ("events", V::Obj(o)) => {
                node.events = o.iter().map(|(k, _)| k.clone()).collect();
            }
            ("span", V::Obj(o)) => {
                node.span = Some((num_of(&o, "offset").unwrap_or(0.0) as usize, num_of(&o, "len").unwrap_or(0.0) as usize));
            }
            ("for_iter", V::Obj(o)) => {
                node.for_ctx = Some(ForCtx {
                    var_name: str_of(&o, "var").unwrap_or_default(),
                    index: num_of(&o, "index").map(|f| f as usize),
                    value: str_of(&o, "value").unwrap_or_default(),
                });
            }
            // box/style/class/value/checked/…：M2 索引不消费，安全略过。
            _ => {}
        }
        self.skip_ws();
        self.eat(b';'); // 项间分隔符（末项可无）
        Ok(())
    }

    fn parse_value(&mut self) -> V {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => {
                self.pos += 1;
                let s = self.pos;
                while let Some(c) = self.peek() {
                    self.pos += 1;
                    if c == b'"' {
                        break;
                    }
                }
                let end = (self.pos - 1).min(self.b.len());
                V::Str(String::from_utf8_lossy(&self.b[s..end]).into_owned())
            }
            Some(b'{') => {
                self.pos += 1;
                let mut obj: Vec<(String, V)> = Vec::new();
                loop {
                    self.skip_ws();
                    match self.peek() {
                        Some(b'}') => {
                            self.pos += 1;
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                    // obj 内 `key: value`
                    let ks = self.pos;
                    while let Some(c) = self.peek() {
                        if c == b':' || c == b'}' || c == b',' || c == b';' {
                            break;
                        }
                        self.pos += 1;
                    }
                    let key = std::str::from_utf8(&self.b[ks..self.pos]).unwrap_or("").trim().to_string();
                    if !self.eat(b':') {
                        // 防御：obj 内出现裸节点（不该发生）——吞 token。
                        self.pos += 1;
                        continue;
                    }
                    let v = self.parse_value();
                    obj.push((key, v));
                    self.skip_ws();
                    // 对象值内分隔：print_object 出 `, `（Node body 顶层才是 `; `）。
                    if !self.eat(b',') {
                        self.eat(b';');
                    }
                }
                V::Obj(obj)
            }
            Some(b'[') => {
                // 数组值（options/col_widths 等）：平衡扫描到配对 ]。
                self.pos += 1;
                let mut depth = 1usize;
                while let Some(c) = self.peek() {
                    self.pos += 1;
                    match c {
                        b'[' => depth += 1,
                        b']' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                V::Other
            }
            _ => {
                // 标量：读到分隔符（Float Display 为 "0"/"40.5" 形态；
                // obj 内分隔 `,` 与 Node body `;`、闭 `}` 同为界）。
                let s = self.pos;
                while let Some(c) = self.peek() {
                    if c == b';' || c == b'}' || c == b',' {
                        break;
                    }
                    self.pos += 1;
                }
                let t = std::str::from_utf8(&self.b[s..self.pos]).unwrap_or("").trim();
                if t == "true" {
                    V::Bool(true)
                } else if t == "false" {
                    V::Bool(false)
                } else {
                    match t.parse::<f32>() {
                        Ok(n) => V::Num(n),
                        Err(_) => V::Other,
                    }
                }
            }
        }
    }
}

enum V {
    Str(String),
    Obj(Vec<(String, V)>),
    Bool(bool),
    Num(f32),
    Other,
}

fn rect_of(o: &[(String, V)]) -> Option<Rect> {
    let f = |k: &str| {
        o.iter().find(|(kk, _)| kk == k).and_then(|(_, v)| match v {
            V::Num(n) => Some(*n),
            _ => None,
        })
    };
    Some(Rect { x: f("x")?, y: f("y")?, w: f("w")?, h: f("h")? })
}

fn str_of(o: &[(String, V)], k: &str) -> Option<String> {
    o.iter().find(|(kk, _)| kk == k).and_then(|(_, v)| match v {
        V::Str(s) => Some(s.clone()),
        _ => None,
    })
}

fn num_of(o: &[(String, V)], k: &str) -> Option<f64> {
    o.iter().find(|(kk, _)| kk == k).and_then(|(_, v)| match v {
        V::Num(n) => Some(*n as f64),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 形状锚样例（vtree_atom 序列化实测形态：`; ` 分隔 props、对象值内
    /// `,` 分隔、哈希 vnode id、span/for_iter 填充臂）。
    const SAMPLE: &str = r##"col vnode_1201 { spacing: 8; padding: 4; bbox: {x: 0, y: 0, w: 100, h: 50}; box: {bbox: {x: 0, y: 0, w: 100, h: 50}, content: {x: 0, y: 0, w: 100, h: 50}, padding: {t: 0, r: 0, b: 0, l: 0}, border: {t: 0, r: 0, b: 0, l: 0}, margin: {t: 0, r: 0, b: 0, l: 0}}; class: "btn-root"; span: {offset: 130, len: 240} text vnode_1302 { content: "Hello"; bbox: {x: 0, y: 4, w: 30, h: 12}; span: {offset: 210, len: 40} } button vnode_1403 { label: "OK"; bbox: {x: 40, y: 10, w: 60, h: 30}; events: {press: ".Ok"}; style: {pad: "24"}; span: {offset: 300, len: 55} } }"##;

    const LOOP_SAMPLE: &str = r##"col vnode_900 { bbox: {x: 0, y: 0, w: 200, h: 120}; span: {offset: 200, len: 355} col vnode_901 { bbox: {x: 0, y: 0, w: 200, h: 120}; for_iter: {var: "item", index: 0, value: "Alpha"}; span: {offset: 230, len: 120} row vnode_902 { bbox: {x: 0, y: 0, w: 200, h: 28}; span: {offset: 250, len: 100} text vnode_903 { content: "Alpha task"; bbox: {x: 0, y: 0, w: 160, h: 28}; span: {offset: 270, len: 60} } button vnode_904 { label: "Pick"; bbox: {x: 160, y: 0, w: 40, h: 28}; events: {press: ".Select"}; span: {offset: 290, len: 50} } } } col vnode_905 { bbox: {x: 0, y: 30, w: 200, h: 120}; for_iter: {var: "item", index: 1, value: "Beta"}; span: {offset: 230, len: 120} row vnode_906 { bbox: {x: 0, y: 30, w: 200, h: 28}; span: {offset: 250, len: 100} text vnode_907 { content: "Beta task"; bbox: {x: 0, y: 30, w: 160, h: 28}; span: {offset: 270, len: 60} } button vnode_908 { label: "Pick"; bbox: {x: 160, y: 30, w: 40, h: 28}; events: {press: ".Select"}; span: {offset: 290, len: 50} } } } }"##;

    fn resolver(off: usize, _len: usize) -> Option<(String, usize, &'static str)> {
        let line = 1 + off / 100;
        Some(("src/front/app.at".to_string(), line, "exact"))
    }

    #[test]
    fn parses_sample_tree_with_meta() {
        let idx = AnchorIndex::parse(SAMPLE, 7).unwrap();
        assert_eq!(idx.nodes.len(), 3);
        let b = idx.get(1403).unwrap();
        assert_eq!(b.kind, "button");
        assert_eq!(b.label.as_deref(), Some("OK"));
        assert_eq!(b.bbox, Some(Rect { x: 40.0, y: 10.0, w: 60.0, h: 30.0 }));
        assert_eq!(b.events, vec!["press".to_string()]);
        assert_eq!(b.span, Some((300, 55)));
        assert_eq!(b.parent, Some(1201));
        assert_eq!(b.depth, 1);
        // box prop 的嵌套 bbox 不得污染顶层 bbox。
        let root = idx.get(1201).unwrap();
        assert_eq!(root.bbox, Some(Rect { x: 0.0, y: 0.0, w: 100.0, h: 50.0 }));
    }

    #[test]
    fn hit_test_prefers_smallest_area() {
        let idx = AnchorIndex::parse(SAMPLE, 1).unwrap();
        // 点在 button 内 → button；点在根空白 → root col；点外 → None。
        assert_eq!(idx.hit_test(70.0, 25.0), Some(1403));
        assert_eq!(idx.hit_test(10.0, 45.0), Some(1201));
        assert_eq!(idx.hit_test(500.0, 500.0), None);
        // 边界含端点。
        assert_eq!(idx.hit_test(40.0, 10.0), Some(1403));
    }

    #[test]
    fn substantive_backtrack_skips_bare_text() {
        // 无 span 无 events 的纯 text：回溯到最近实质祖先（有 events 的 button）。
        let atom2 = r##"col vnode_0 { bbox: {x: 0, y: 0, w: 100, h: 100} button vnode_1 { label: "B"; bbox: {x: 0, y: 0, w: 100, h: 50}; events: {press: ".B"} text vnode_2 { content: "inner"; bbox: {x: 0, y: 0, w: 20, h: 10} } } }"##;
        let idx2 = AnchorIndex::parse(atom2, 1).unwrap();
        assert_eq!(idx2.substantive_anchor(2), 1);
        // 全链无实质 → 返回命中本身。
        let atom = r##"col vnode_0 { bbox: {x: 0, y: 0, w: 100, h: 100} text vnode_2 { content: "bare"; bbox: {x: 0, y: 0, w: 20, h: 10} } }"##;
        let idx = AnchorIndex::parse(atom, 1).unwrap();
        assert_eq!(idx.substantive_anchor(2), 2);
    }

    #[test]
    fn loop_instances_distinguishable() {
        let idx = AnchorIndex::parse(LOOP_SAMPLE, 1).unwrap();
        // 第 1 项与第 2 项的 Pick 按钮不同 vnode、同 source 模板行（span 同）。
        let b1 = idx.get(904).unwrap();
        let b2 = idx.get(908).unwrap();
        assert_ne!(b1.vnode, b2.vnode);
        assert_eq!(b1.span, b2.span);
        // for_context 沿链取到容器实例的 index。
        assert_eq!(idx.for_context_of(904).and_then(|f| f.index), Some(0));
        assert_eq!(idx.for_context_of(908).and_then(|f| f.index), Some(1));
        // 祖先链 root→…→按钮。
        assert_eq!(idx.ancestor_chain(904), vec![900, 901, 902, 904]);
    }

    #[test]
    fn pick_json_shape_and_scale() {
        let idx = AnchorIndex::parse(SAMPLE, 1).unwrap();
        // 帧 200×100 逻辑 100×50 → scale 2.0；pct = 像素/帧尺寸×100。
        let p = idx.pick_json(1403, 2.0, 200, 100, &resolver).unwrap();
        assert_eq!(p["kind"], "button");
        // id 字符串形态（哈希 vnode > JS 2^53，数字会被静默截断）。
        assert_eq!(p["vnode_id"], "vnode_1403");
        assert_eq!(p["bbox"]["w"], 60.0);
        assert_eq!(p["bbox_px"]["w"], 120.0); // 逻辑 × scale
        // pct 经 f32 除法有尾差，近似断言。
        let pct_w = p["bbox_pct"]["w"].as_f64().unwrap_or(0.0);
        assert!((pct_w - 60.0).abs() < 0.01, "pct w = {pct_w}");
        assert_eq!(p["source"], "src/front/app.at:4"); // offset 300 → line 4
        assert_eq!(p["source_confidence"], "exact"); // 测试 resolver 固定章
        assert_eq!(p["ancestor_chain"], json!(["vnode_1201", "vnode_1403"]));
    }

    #[test]
    fn tree_flat_shape() {
        let idx = AnchorIndex::parse(SAMPLE, 1).unwrap();
        let t = idx.tree_flat_json(&resolver);
        let items = t.as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["kind"], "col");
        assert_eq!(items[0]["depth"], 0);
        assert_eq!(items[1]["depth"], 1);
        assert_eq!(items[2]["kind"], "button");
        assert_eq!(items[2]["id"], "vnode_1403");
        assert!(items[2]["source"].as_str().unwrap().starts_with("src/front/app.at:"));
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(AnchorIndex::parse("not a vtree", 1).is_err());
        assert!(AnchorIndex::parse("col vnode_1 { broken", 1).is_err());
    }

    #[test]
    fn duplicate_vnode_rejected() {
        assert!(AnchorIndex::parse(r##"col vnode_1 { text vnode_1 { content: "x" } }"##, 1).is_err());
    }

    /// PLAN-093 Q-04：来源置信度两档——单候选文件 = exact（硬验收面），
    /// 多候选 = uncertain（前端必须显示"来源待确认"）。pac.at 不计入
    /// 候选（包清单非源码）。
    #[test]
    fn resolve_source_confidence_single_vs_multi() {
        let tmp = std::env::temp_dir().join(format!("anchor-conf-{}", std::process::id()));
        // 真实 app 形态：根 pac.at（清单，排除）+ 单一 src 源文件 → exact。
        let single = tmp.join("single");
        std::fs::create_dir_all(single.join("src")).unwrap();
        std::fs::write(single.join("pac.at"), "app: \"demo\"\n").unwrap();
        std::fs::write(single.join("src/app.at"), "widget App {\n  view {\n    col {\n      text \"hi\"\n    }\n  }\n}\n").unwrap();
        let r = AnchorIndex::resolve_source(&single, (4, 30), "text").unwrap();
        assert_eq!(r.0, "src/app.at");
        assert_eq!(r.2, "exact", "pac.at 排除后单候选 → exact");

        // 双源文件：同 span 两文件都容纳 → uncertain（启发式，不得假装
        // 准确锚定）。
        let multi = tmp.join("multi/src");
        std::fs::create_dir_all(&multi).unwrap();
        std::fs::write(multi.join("app.at"), "widget App {\n  view {\n    col {\n      text \"hi\"\n    }\n  }\n}\n").unwrap();
        std::fs::write(multi.join("views.at"), "widget Views {\n  view {\n    col {\n      text \"there\"\n    }\n  }\n}\n").unwrap();
        let r2 = AnchorIndex::resolve_source(&tmp.join("multi"), (4, 30), "text").unwrap();
        assert_eq!(r2.2, "uncertain", "多候选 → uncertain");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
