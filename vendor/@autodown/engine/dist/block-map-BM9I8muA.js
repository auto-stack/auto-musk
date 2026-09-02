import { ref as y, getCurrentInstance as Jn, onBeforeUnmount as Ge, defineComponent as Z, computed as g, onMounted as ge, openBlock as b, createBlock as J, resolveDynamicComponent as pe, withModifiers as Pe, normalizeClass as $e, createElementBlock as R, unref as de, createCommentVNode as $, h as te, withCtx as st, Fragment as xe, renderList as Je, onUnmounted as St, normalizeStyle as _n, createElementVNode as _, withDirectives as kt, withKeys as tt, vModelText as Ot, toDisplayString as U, nextTick as ce, watch as De, inject as Zn, provide as eo, createTextVNode as ut, vShow as to, createVNode as yt, mergeProps as no } from "vue";
import { X as Be, Y as Se, Z as At, _ as Yt, m as F, B as ae, S as Fe, $ as $t, i as h, f as q, b as Y, a0 as oo, a1 as In, r as W, a2 as lo, g as He, V as se, n as je, a3 as Cn, w as Ue, l as Ke, j as X, k as Ye, y as ct, a4 as Xt, c as Ae, h as Me, M as E, a5 as Jt, a6 as nt, I as _t, a7 as io, a8 as ro, a9 as Dt, t as so, d as ao } from "./markdown-parser-0FkmfLuR.js";
import { i as Zt, s as ot } from "./parser-BSmv1gWa.js";
import { F as uo, p as xn, G as co, H as fo, I as Tt, J as en, K as dt, L as ho, a as tn, c as mo, t as vo, b as po, M as Rt, N as Bn, O as ft, P as Sn, Q as qt, R as Tn, S as ht, U as Ht, V as Rn, _ as bt, W as Mn, X as go, Y as ko, Z as bo, $ as wo, a0 as yo, a1 as _o, y as Nt, x as Io, l as Ze, a2 as Co, a3 as xo, w as wt, E as Ln, a4 as Bo, a5 as So, a6 as To, a7 as Ro, a8 as Mo, a9 as nn, aa as Lo, T as Eo } from "./render-node-DdquDFdQ.js";
import { Link as En, Code as On, Strikethrough as Oo, Underline as Ao, Italic as $o, Bold as Do, Check as qo, Text as Ho, Heading1 as No, Heading2 as Po, Heading3 as Fo, Heading4 as Ko, Heading5 as Uo, Heading6 as Wo, List as Vo, ListOrdered as Qo, CheckSquare as zo, Quote as Go, Minus as jo, Image as Yo, Table as Xo, AlertCircle as Jo, PanelTop as Zo, Sigma as el, Workflow as tl, Square as nl, CircleDot as ol, CheckCircle2 as ll, Clock as il, Timer as rl, ArrowUp as It, Search as sl } from "lucide-vue-next";
class al {
  constructor(t, e) {
    var o;
    this.undoStack = [], this.redoStack = [], this.listeners = [], this.tree = t, this.sel = e ?? Be(((o = t.children[0]) == null ? void 0 : o.id) ?? "", 0);
  }
  get doc() {
    return this.tree;
  }
  get selection() {
    return this.sel;
  }
  get canUndo() {
    return this.undoStack.length > 0;
  }
  get canRedo() {
    return this.redoStack.length > 0;
  }
  onChange(t) {
    this.listeners.push(t);
  }
  /** Apply one op through the 016 kernel, recording an undo entry.
   *  Adjacent InsertText typing coalesces into the previous entry. */
  apply(t, e = {}) {
    const o = this.undoStack[this.undoStack.length - 1], l = o != null && o.ops.length === 1 ? o.ops[0] : void 0;
    if (e.coalesce !== !1 && l != null && l._tag === "InsertText" && t._tag === "InsertText" && l.value.pos.blockId === t.value.pos.blockId && l.value.pos.offset + l.value.text.length === t.value.pos.offset && o) {
      const s = l.value, u = t.value;
      o.ops[0] = Se.InsertText(new At(s.pos, s.text + u.text));
    } else
      this.undoStack.push({ preTree: this.tree, preSel: this.sel, ops: [t] });
    this.redoStack = [];
    const r = Yt(this.tree, this.sel, t);
    return this.tree = r.tree, this.sel = r.selection, this.emit(!1), r;
  }
  /** Apply a composed op group as ONE undo step (input rules etc.). */
  applyGroup(t, e) {
    t.length === 0 && e == null || (this.undoStack.push({ preTree: this.tree, preSel: this.sel, ops: [...t], after: e }), this.redoStack = [], this.thread(t, e), this.emit(!1));
  }
  /** Apply a pure tree transform as ONE undo step (command layer:
   *  insertTemplate / table ops / moveBlock — Phase 3). */
  applyTree(t) {
    this.applyGroup([], t);
  }
  /** Set the selection without a document change (focus moves). */
  select(t) {
    this.sel = t, this.emit(!1);
  }
  thread(t, e) {
    let o = this.tree, l = this.sel;
    for (const i of t) {
      const r = Yt(o, l, i);
      o = r.tree, l = r.selection;
    }
    e && (o = e(o)), this.tree = o, this.sel = l;
  }
  undo() {
    const t = this.undoStack.pop();
    return t ? (this.redoStack.push(t), this.tree = t.preTree, this.sel = t.preSel, this.emit(!0), !0) : !1;
  }
  redo() {
    const t = this.redoStack.pop();
    if (!t) return !1;
    const e = this.tree, o = this.sel;
    return this.thread(t.ops, t.after), this.undoStack.push({ preTree: e, preSel: o, ops: t.ops, after: t.after }), this.emit(!0), !0;
  }
  /** Streaming append (plan 018 待澄清 1 — 追加分流裁定): AI/stream blocks
   *  land at the document tail without touching the focused block or the
   *  selection; not an undoable user edit. */
  appendBlocks(t) {
    t.length !== 0 && (this.tree = F(this.tree, [...this.tree.children, ...t]), this.emit(!1));
  }
  /** External document replacement (file load, full paste). Not undoable —
   *  callers that need undo wrap it in their own op. */
  replaceDoc(t, e) {
    var o;
    this.tree = t, this.sel = e ?? Be(((o = t.children[0]) == null ? void 0 : o.id) ?? "", 0), this.undoStack = [], this.redoStack = [], this.emit(!1);
  }
  emit(t) {
    for (const e of this.listeners) e({ tree: this.tree, selection: this.sel, history: t });
  }
}
class ul {
  constructor() {
    this.active = !1, this.baseline = "", this.blockId = "", this.baselineOffset = 0;
  }
  get composing() {
    return this.active;
  }
  /** compositionstart — record the pre-edit state of the focused block. */
  begin(t, e, o) {
    this.active = !0, this.blockId = t, this.baseline = e, this.baselineOffset = o;
  }
  /** compositionupdate — staged preedit; produces NO op by contract. */
  update(t) {
    return this.active, null;
  }
  /** compositionend — diff baseline → final text into one op. */
  commit(t) {
    if (!this.active) return null;
    if (this.active = !1, this.baseline.length === 0 && t.length > 0)
      return Se.InsertText(new At(new ae(this.blockId, this.baselineOffset), t));
    if (t === this.baseline) return null;
    const e = new Fe(
      new ae(this.blockId, this.baselineOffset),
      new ae(this.blockId, this.baselineOffset + this.baseline.length)
    );
    return Se.ReplaceRange(new $t(e, t));
  }
  /** composition cancelled — nothing happened, by contract. */
  cancel() {
    return this.active = !1, null;
  }
}
function An(n, t, e) {
  if (t === e) return null;
  let o = 0;
  for (; o < t.length && o < e.length && t[o] === e[o]; ) o++;
  let l = 0;
  for (; l < t.length - o && l < e.length - o && t[t.length - 1 - l] === e[e.length - 1 - l]; )
    l++;
  const i = t.slice(o, t.length - l), r = e.slice(o, e.length - l);
  if (i.length === 0)
    return Se.InsertText(new At(new ae(n, o), r));
  const s = new Fe(new ae(n, o), new ae(n, o + i.length));
  return Se.ReplaceRange(new $t(s, r));
}
const cl = [
  { marker: "# ", kind: h.Heading, level: 1 },
  { marker: "## ", kind: h.Heading, level: 2 },
  { marker: "### ", kind: h.Heading, level: 3 },
  { marker: "- ", kind: h.ListItem, wrap: h.ListBlock },
  { marker: "* ", kind: h.ListItem, wrap: h.ListBlock },
  { marker: "+ ", kind: h.ListItem, wrap: h.ListBlock },
  { marker: "> ", kind: h.Blockquote, wrap: h.Blockquote },
  { marker: "``` ", kind: h.Fence },
  { marker: "---", kind: h.ThematicBreak },
  { marker: "***", kind: h.ThematicBreak }
];
function dl(n) {
  for (const t of cl)
    if (n === t.marker) return t;
  return null;
}
function fl(n, t, e) {
  const o = q(n, t);
  if (!o) return null;
  const l = Y(o);
  if (l !== e.marker) return null;
  const i = new Fe(new ae(t, 0), new ae(t, l.length)), r = [Se.ReplaceRange(new $t(i, ""))];
  return e.wrap || r.push(Se.SetBlockType(new oo(t, e.kind))), { ops: r, rule: e };
}
function hl(n, t, e) {
  if (e.level == null) return n;
  const o = q(n, t);
  return o ? W(n, t, [lo({ ...o, attrs: He(o.attrs, "level", se.Int(e.level)) }, o.kind)]) : n;
}
function ml(n, t, e) {
  const o = [];
  for (; o.length < t; ) {
    const l = `${e}-${Math.random().toString(36).slice(2, 8)}`;
    !In(n, l) && !o.includes(l) && o.push(l);
  }
  return o;
}
function vl(n, t, e, o) {
  const l = q(n, t);
  if (!l || !e.wrap) return n;
  if (e.wrap === h.ListBlock) {
    const [i, r] = o, s = F(je(r ?? "li-x", h.ListItem), [l]);
    return W(n, t, [F(je(i ?? "lb-x", h.ListBlock), [s])]);
  }
  return W(n, t, [F(je(o[0] ?? "bq-x", h.Blockquote), [l])]);
}
function pl(n, t) {
  const e = q(n.doc, t);
  if (!e) return !1;
  const o = dl(Y(e));
  if (!o) return !1;
  const l = fl(n.doc, t, o);
  if (!l) return !1;
  const i = o.wrap ? ml(n.doc, o.wrap === h.ListBlock ? 2 : 1, "b") : [];
  return n.applyGroup(l.ops, (r) => hl(vl(r, t, o, i), t, o)), !0;
}
function mt(n, t) {
  for (; ; ) {
    const e = `${t}-${Math.random().toString(36).slice(2, 8)}`;
    if (!In(n, e)) return e;
  }
}
function ke(n, t) {
  const e = q(n, t);
  if (!e) return null;
  const o = X(n, t);
  if (!o || o.kind !== h.ListItem) return null;
  const l = X(n, o.id);
  return !l || l.kind !== h.ListBlock ? null : { para: e, item: o, list: l, itemIndex: Ye(l, o.id) };
}
function gl(n) {
  for (let t = n.children.length - 1; t >= 0; t--) {
    const e = n.children[t];
    if (e.kind === h.Paragraph) return { node: e, index: t };
  }
  return null;
}
function kl(n) {
  const t = [], e = ct(n.attrs, "ordered");
  e != null && t.push(Xt("ordered", e));
  const o = ct(n.attrs, "start");
  return o != null && t.push(Xt("start", o)), t;
}
function $n(n, t, e) {
  ke(n.doc, t) && (n.applyTree((o) => {
    const l = ke(o, t);
    if (!l) return o;
    const i = l.item.children.filter((u) => u.id !== t), r = [...l.list.children];
    i.length > 0 ? r[l.itemIndex] = F(l.item, i) : r.splice(l.itemIndex, 1);
    let s;
    if (r.length === 0) s = [l.para];
    else {
      const u = F(l.list, r);
      s = e === "after" ? [u, l.para] : [l.para, u];
    }
    return W(o, l.list.id, s);
  }), n.select(Be(t, 0)));
}
function bl(n, t, e) {
  const o = ke(n.doc, t);
  if (!o) return;
  if (Y(o.para) === "") {
    $n(n, t, "after");
    return;
  }
  const l = mt(n.doc, "b"), i = mt(n.doc, "li");
  n.applyTree((r) => {
    const s = ke(r, t);
    if (!s) return r;
    const u = Cn(s.para.inlines, e), d = F(
      s.item,
      s.item.children.map((v) => v.id === t ? Ue(s.para, u.before) : v)
    ), c = F(je(i, h.ListItem), [
      Ue(Ke(l, h.Paragraph, ""), u.after)
    ]), f = [...s.list.children];
    return f.splice(s.itemIndex, 1, d, c), W(r, s.list.id, [F(s.list, f)]);
  }), n.select(Be(l, 0));
}
function wl(n, t) {
  const e = ke(n.doc, t);
  if (!e) return;
  if (e.itemIndex === 0) {
    $n(n, t, "before");
    return;
  }
  let o = "", l = 0;
  n.applyTree((i) => {
    const r = ke(i, t);
    if (!r || r.itemIndex === 0) return i;
    const s = r.list.children[r.itemIndex - 1], u = r.item.children.filter((k) => k.id !== t), d = gl(s);
    let c;
    if (d) {
      o = d.node.id, l = Y(d.node).length;
      const k = Ue(d.node, [...d.node.inlines, ...r.para.inlines]);
      c = [...s.children.map((T, m) => m === d.index ? k : T), ...u];
    } else
      o = t, l = 0, c = [...s.children, r.para, ...u];
    const f = F(s, c), v = r.list.children.filter((k, T) => T !== r.itemIndex && T !== r.itemIndex - 1);
    return v.splice(r.itemIndex - 1, 0, f), W(i, r.list.id, [F(r.list, v)]);
  }), o && n.select(Be(o, l));
}
function yl(n, t) {
  const e = ke(n.doc, t);
  if (!e || e.itemIndex <= 0) return;
  const o = mt(n.doc, "lb");
  n.applyTree((l) => {
    const i = ke(l, t);
    if (!i || i.itemIndex <= 0) return l;
    const r = i.list.children[i.itemIndex - 1], u = [...r.children].reverse().find((v) => v.kind === h.ListBlock) ?? {
      ...je(o, h.ListBlock),
      attrs: kl(i.list)
    }, d = r.children.filter((v) => v.id !== u.id), c = F(r, [...d, F(u, [...u.children, i.item])]), f = i.list.children.filter((v, k) => k !== i.itemIndex);
    return f.splice(i.itemIndex - 1, 1, c), W(l, i.list.id, [F(i.list, f)]);
  });
}
function _l(n, t) {
  const e = ke(n.doc, t);
  if (!e) return;
  const o = X(n.doc, e.list.id);
  !o || o.kind !== h.ListItem || n.applyTree((l) => {
    const i = ke(l, t);
    if (!i) return l;
    const r = X(l, i.list.id);
    if (!r || r.kind !== h.ListItem) return l;
    const s = X(l, r.id);
    if (!s) return l;
    const u = i.list.children.filter((v) => v.id !== i.item.id), d = r.children.filter((v) => v.id !== i.list.id), c = Ye(s, r.id);
    let f;
    if (d.length > 0) {
      const v = u.length > 0 ? [...d, F(i.list, u)] : d, k = F(r, v);
      f = s.children.map((T, m) => m === c ? k : T), f.splice(c + 1, 0, i.item);
    } else
      f = s.children.filter((v, k) => k !== c), f.splice(c, 0, i.item);
    return W(l, s.id, [F(s, f)]);
  });
}
function Il(n, t, e) {
  const o = q(n.doc, t), l = o ? X(n.doc, t) : null;
  if (!o || !l || l.kind !== h.Blockquote) return;
  if (Y(o) === "") {
    Cl(n, t);
    return;
  }
  const i = mt(n.doc, "b");
  n.applyTree((r) => {
    const s = q(r, t), u = s ? X(r, t) : null;
    if (!s || !u || u.kind !== h.Blockquote) return r;
    const d = Cn(s.inlines, e), c = Ye(u, t), f = [...u.children];
    return f[c] = Ue(s, d.before), f.splice(c + 1, 0, Ue(Ke(i, h.Paragraph, ""), d.after)), W(r, u.id, [F(u, f)]);
  }), n.select(Be(i, 0));
}
function Cl(n, t) {
  const e = q(n.doc, t), o = e ? X(n.doc, t) : null;
  !e || !o || o.kind !== h.Blockquote || (n.applyTree((l) => {
    const i = q(l, t), r = i ? X(l, t) : null;
    if (!i || !r || r.kind !== h.Blockquote) return l;
    const s = r.children.filter((d) => d.id !== t), u = s.length === 0 ? [i] : [F(r, s), i];
    return W(l, r.id, u);
  }), n.select(Be(t, 0)));
}
function at(n) {
  return n.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function lt(n) {
  return at(n).replace(/'/g, "&#39;");
}
function Dn(n) {
  let t = "";
  for (const e of n) {
    let o = at(e.text);
    const l = Ae(e.attrs, "wikilink", "");
    if (l !== "") {
      const r = l.indexOf("#"), s = (r >= 0 ? l.slice(0, r) : l).trim();
      o = `<span class="autodown-wikilink-label" data-wikilink-title="${lt(s)}" contenteditable="false">${at(l)}</span>`;
    }
    const i = Ae(e.attrs, "math_inline", "");
    if (i !== "" && (o = `<span class="autodown-math-inline" data-math-src="${lt(i)}" contenteditable="false">${at(i)}</span>`), Me(e.marks, E.Code) && (o = `<code>${o}</code>`), Me(e.marks, E.Strong) && (o = `<strong>${o}</strong>`), Me(e.marks, E.Em) && (o = `<em>${o}</em>`), Me(e.marks, E.Underline) && (o = `<u>${o}</u>`), Me(e.marks, E.Del) && (o = `<del>${o}</del>`), Me(e.marks, E.Link)) {
      const r = lt(Ae(e.attrs, "href", "")), s = Ae(e.attrs, "title", ""), u = s ? ` title="${lt(s)}"` : "";
      o = `<a href="${r}"${u} contenteditable="false" data-autodown-link>${o}</a>`;
    }
    t += o;
  }
  return t.replace(/ +$/, (e) => "&nbsp;".repeat(e.length));
}
function xl(n) {
  const t = n.toUpperCase();
  return t === "STRONG" || t === "B" ? E.Strong : t === "EM" || t === "I" ? E.Em : t === "DEL" || t === "S" ? E.Del : t === "U" ? E.Underline : t === "CODE" ? E.Code : null;
}
function Mt(n) {
  let t = "";
  for (const e of n ?? [])
    e.text !== void 0 ? t += e.text.replace(/\u00A0/g, " ") : t += Mt(e.children);
  return t;
}
function Bl(n) {
  const t = [], e = (o, l, i) => {
    var c, f, v;
    if (o.text !== void 0) {
      const k = o.text.replace(/\u00A0/g, " ");
      k !== "" && t.push(new _t(k, l, i));
      return;
    }
    const r = ((c = o.attrs) == null ? void 0 : c.class) ?? "";
    if (r.includes("autodown-wikilink-label")) {
      const k = Mt(o.children);
      k !== "" && t.push(new _t(k, l, [new nt("wikilink", se.Str(k))]));
      return;
    }
    if (r.includes("autodown-math-inline")) {
      const k = ((f = o.attrs) == null ? void 0 : f["data-math-src"]) ?? Mt(o.children);
      k !== "" && t.push(new _t(k, l, [new nt("math_inline", se.Str(k))]));
      return;
    }
    let s = l, u = i;
    const d = xl(o.tag ?? "");
    d !== null && (s = Jt(s, d)), (o.tag ?? "").toUpperCase() === "A" && ((v = o.attrs) == null ? void 0 : v.href) !== void 0 && (s = Jt(s, E.Link), u = [new nt("href", se.Str(o.attrs.href))], o.attrs.title !== void 0 && u.push(new nt("title", se.Str(o.attrs.title))));
    for (const k of o.children ?? []) e(k, s, u);
  };
  return e(n, [], []), uo(t);
}
function Sl(n) {
  const t = (e) => {
    var c, f, v, k, T;
    if (e.nodeType === 3) return { text: e.textContent ?? "" };
    const o = e, l = {}, i = (c = o.getAttribute) == null ? void 0 : c.call(o, "href");
    i != null && (l.href = i);
    const r = (f = o.getAttribute) == null ? void 0 : f.call(o, "title");
    r != null && (l.title = r);
    const s = (v = o.getAttribute) == null ? void 0 : v.call(o, "class");
    s != null && (l.class = s);
    const u = (k = o.getAttribute) == null ? void 0 : k.call(o, "data-wikilink-title");
    u != null && (l["data-wikilink-title"] = u);
    const d = (T = o.getAttribute) == null ? void 0 : T.call(o, "data-math-src");
    return d != null && (l["data-math-src"] = d), { tag: o.tagName ?? "", children: Array.from(e.childNodes).map(t), attrs: l };
  };
  return Bl(t(n));
}
class Tl {
  constructor(t, e) {
    this.composition = new ul(), this.engine = t, this.blockId = e;
    const o = q(t.doc, e);
    this.knownText = o ? Y(o) : "";
  }
  get id() {
    return this.blockId;
  }
  get text() {
    return this.knownText;
  }
  /** The block's inline spans (rich host mount render — plan 024 P2T1). */
  get inlines() {
    const t = q(this.engine.doc, this.blockId);
    return t ? t.inlines : [];
  }
  /** The host was (re)rendered from the engine — re-sync the known text
   *  (history changes repaint the host). */
  syncFromModel() {
    const t = q(this.engine.doc, this.blockId);
    return this.knownText = t ? Y(t) : "", this.knownText;
  }
  /** `input` DOM event outside composition: old→new text becomes one op. */
  onInput(t) {
    if (this.composition.composing) return null;
    const e = An(this.blockId, this.knownText, t);
    return this.knownText = t, e ? (this.engine.apply(e), pl(this.engine, this.blockId), this.syncFromModel(), e) : null;
  }
  /** Enter key at caret offset → split the block. Nested paragraphs dispatch
   *  on the parent kind first (plan 025 P1T3): a ListItem parent splits the
   *  ITEM, a Blockquote parent continues the quote; only top-level leaves
   *  take the bare SplitBlock path. */
  onEnter(t, e) {
    const o = X(this.engine.doc, this.blockId);
    if ((o == null ? void 0 : o.kind) === h.ListItem) {
      bl(this.engine, this.blockId, t), this.syncFromModel();
      return;
    }
    if ((o == null ? void 0 : o.kind) === h.Blockquote) {
      Il(this.engine, this.blockId, t), this.syncFromModel();
      return;
    }
    this.engine.apply(Se.SplitBlock(new io(new ae(this.blockId, t), e))), this.knownText = "";
  }
  /** Backspace at offset 0 → merge with the previous sibling (if any). In a
   *  list item the structural command owns the semantics (merge into the
   *  previous ITEM / lift the first item out); elsewhere the merge target
   *  must be an editable leaf of the same container — a container sibling
   *  (nested list subtree) never merges. */
  onBackspaceAtStart(t) {
    const e = X(this.engine.doc, this.blockId);
    if ((e == null ? void 0 : e.kind) === h.ListItem)
      return wl(this.engine, this.blockId), this.syncFromModel(), !0;
    if (!t) return !1;
    const o = q(this.engine.doc, t);
    return !o || !Pt(o) ? !1 : (this.engine.apply(Se.MergeBlocks(new ro(t, this.blockId))), this.syncFromModel(), !0);
  }
  /** Tab / Shift+Tab inside a list item → indent / outdent (plan 025 P1T3).
   *  Returns false (browser default) when the block is not in a list. */
  onTab(t) {
    const e = X(this.engine.doc, this.blockId);
    return (e == null ? void 0 : e.kind) !== h.ListItem ? !1 : (t ? _l(this.engine, this.blockId) : yl(this.engine, this.blockId), this.syncFromModel(), !0);
  }
  // -- composition delegates ------------------------------------------------------
  compositionBegin(t, e) {
    this.composition.begin(this.blockId, t, e);
  }
  compositionUpdate(t) {
    this.composition.update(t);
  }
  compositionCommit(t) {
    const e = this.composition.commit(t);
    return e && (this.engine.apply(e, { coalesce: !1 }), this.syncFromModel()), e;
  }
  compositionCancel() {
    this.composition.cancel();
  }
  /** Markdown / multiline paste: parse to blocks and insert after this one
   *  (plan 018 目标 5 — paste is v1-mandatory; HTML paste degrades to
   *  text/plain per 待澄清 5). */
  onPasteMarkdown(t) {
    const e = Dt(t, !0), o = e.children.length > 0 ? e.children : [];
    if (o.length === 0) return;
    const l = this.engine.doc, i = l.children, r = i.findIndex((u) => u.id === this.blockId), s = [...i.slice(0, r + 1), ...o, ...i.slice(r + 1)];
    this.engine.applyTree(() => F(l, s)), this.syncFromModel();
  }
  // -- rich blur writeback (plan 024 P2T2) ----------------------------------------
  /** Focus-leave writeback of the rich host: DOM walk → spans → whole-block
   *  withInlines through applyTree — ONE undo step, CodeEditorBlock protocol.
   *  Returns true when a rewrite landed. */
  onRichBlur(t) {
    return this.commitRichSpans(Sl(t));
  }
  /** Headless core of onRichBlur (the walk itself is e2e-pinned). Blocks
   *  carrying Image marks are skipped: their marks are not rendered in the
   *  rich host, so a rewrite would silently drop them (v1 no-data-loss). */
  commitRichSpans(t) {
    const e = q(this.engine.doc, this.blockId);
    return !e || this.inlines.some((o) => Me(o.marks, E.Image)) || Zt(e.inlines) === Zt(t) ? !1 : (this.engine.applyTree((o) => {
      const l = q(o, this.blockId);
      return l ? W(o, this.blockId, [Ue(l, t)]) : o;
    }), this.syncFromModel(), !0);
  }
}
function Pt(n) {
  return !(n.children.length !== 0 || n.kind === h.ThematicBreak || n.kind === h.Details || n.kind === h.Callout || n.kind === h.QueryBlock || n.kind === h.BlockEmbed);
}
function Rl(n) {
  if (!n.ctrlKey && !n.metaKey) return null;
  const t = n.key.toLowerCase();
  return t === "z" ? n.shiftKey ? "redo" : "undo" : t === "y" ? "redo" : null;
}
function Ml(n, t, e) {
  const o = e === "undo" ? n.undo() : n.redo();
  if (o) for (const l of t) l.syncFromModel();
  return o;
}
function Ll(n, t) {
  const e = /* @__PURE__ */ new Set();
  if (!t) return e;
  let o = t;
  for (; ; ) {
    const l = X(n, o);
    if (!l || l === n) break;
    e.add(l.id), o = l.id;
  }
  return e;
}
function qn(n) {
  return xn(h[n.kind]) != null || Pt(n);
}
function Lt(n) {
  if (qn(n)) return n;
  for (const t of n.children) {
    const e = Lt(t);
    if (e) return e;
  }
  return null;
}
function Hn(n) {
  if (qn(n)) return n;
  for (let t = n.children.length - 1; t >= 0; t--) {
    const e = Hn(n.children[t]);
    if (e) return e;
  }
  return null;
}
function Ft(n) {
  return n.nodeType === 3 ? { raw: n, isText: !0, text: n.textContent ?? "", children: [] } : { raw: n, isText: !1, text: "", children: Array.from(n.childNodes).map(Ft) };
}
function Nn(n) {
  const t = [];
  let e = 0;
  const o = (l) => {
    l.isText ? (t.push({ node: l, start: e, len: l.text.length }), e += l.text.length) : l.children.forEach(o);
  };
  return o(n), { leaves: t, total: e };
}
function Kt(n, t) {
  if (n.raw === t) return n;
  for (const e of n.children) {
    const o = Kt(e, t);
    if (o) return o;
  }
  return null;
}
function on(n, t, e) {
  const o = Kt(n, t);
  if (!o) return -1;
  const { leaves: l } = Nn(n);
  if (o.isText) {
    const d = l.find((c) => c.node.raw === t);
    return d ? d.start + Math.max(0, Math.min(e, d.len)) : -1;
  }
  const i = Math.max(0, e), r = l.find((d) => {
    let c = d.node, f;
    for (; c && c !== o; )
      f = c, c = Pn(n, c);
    return !c || !f ? !1 : o.children.indexOf(f) >= i;
  });
  if (r) return r.start;
  const s = l.filter((d) => El(o, d.node.raw));
  if (s.length === 0) return -1;
  const u = s[s.length - 1];
  return u.start + u.len;
}
function Pn(n, t) {
  for (const e of n.children) {
    if (e === t) return n;
    const o = Pn(e, t);
    if (o) return o;
  }
}
function El(n, t) {
  return !!Kt(n, t);
}
function ln(n, t) {
  const { leaves: e, total: o } = Nn(n);
  if (e.length === 0) return null;
  const l = Math.max(0, Math.min(t, o));
  for (const r of e)
    if (l <= r.start + r.len) return { raw: r.node.raw, inner: l - r.start };
  const i = e[e.length - 1];
  return { raw: i.node.raw, inner: i.len };
}
function Ol(n, t) {
  const e = typeof window > "u" ? null : window.getSelection();
  if (!e || e.rangeCount === 0) return null;
  const o = e.getRangeAt(0);
  if (o.collapsed || !n.contains(o.startContainer) || !n.contains(o.endContainer)) return null;
  const l = Ft(n), i = on(l, o.startContainer, o.startOffset), r = on(l, o.endContainer, o.endOffset);
  return i < 0 || r < 0 ? null : { blockId: t, lo: Math.min(i, r), hi: Math.max(i, r) };
}
function Al(n, t, e) {
  const o = Ft(n), i = n.ownerDocument.createRange(), r = ln(o, Math.min(t, e)), s = ln(o, Math.max(t, e));
  return !r || !s ? (i.selectNodeContents(n), i) : (i.setStart(r.raw, r.inner), i.setEnd(s.raw, s.inner), i);
}
let Ne = null;
function Ut(n) {
  Ne = n;
}
function Fn() {
  return Ne;
}
const rn = {
  strong: ["strong", "b"],
  em: ["em", "i"],
  del: ["del", "s"],
  u: ["u"],
  code: ["code"]
}, Ct = {
  [E.Strong]: "strong",
  [E.Em]: "em",
  [E.Del]: "del",
  [E.Underline]: "u",
  [E.Code]: "code"
};
function it(n) {
  const t = typeof window > "u" ? null : window.getSelection();
  if (!t || t.rangeCount === 0) return null;
  const e = t.getRangeAt(0);
  return e.collapsed || !n.contains(e.startContainer) || !n.contains(e.endContainer) ? null : e;
}
function Ie(n, t, e) {
  let o = t;
  for (; o && o !== n; ) {
    if (o.nodeType === 1) {
      const l = o;
      if (e.includes(l.tagName.toLowerCase())) return l;
    }
    o = o.parentNode;
  }
  return null;
}
function sn(n, t, e) {
  const o = document.createRange();
  o.selectNodeContents(n);
  try {
    o.setEnd(t, e);
  } catch {
    return 0;
  }
  return o.toString().replace(/\u00A0/g, " ").length;
}
function an(n, t, e) {
  const o = Ie(n, t.startContainer, e);
  return o != null && o === Ie(n, t.endContainer, e) && o.contains(t.commonAncestorContainer);
}
function $l(n) {
  const t = n.parentNode;
  if (t) {
    for (; n.firstChild; ) t.insertBefore(n.firstChild, n);
    t.removeChild(n), t.normalize();
  }
}
function un(n, t, e) {
  try {
    n.surroundContents(t);
  } catch {
    const o = n.extractContents();
    t.appendChild(o), n.insertNode(t);
  }
}
const ue = {
  getSelection() {
    const n = Ne;
    if (!n) return null;
    const t = it(n);
    return t ? {
      blockId: n.dataset.blockId ?? "",
      start: sn(n, t.startContainer, t.startOffset),
      end: sn(n, t.endContainer, t.endOffset)
    } : null;
  },
  isActive(n) {
    const t = Ne;
    if (!t) return !1;
    const e = it(t);
    if (!e) return !1;
    if (n === E.Link) {
      const l = Ie(t, e.startContainer, ["a"]);
      return l != null && l === Ie(t, e.endContainer, ["a"]);
    }
    const o = Ct[n];
    return o == null ? !1 : an(t, e, rn[o]);
  },
  applyMark(n, t) {
    const e = Ne;
    if (!e) return !1;
    const o = it(e);
    if (!o) return !1;
    if (n === E.Link) {
      if (t == null || t === "") return ue.removeMark(E.Link);
      const r = Ie(e, o.startContainer, ["a"]);
      if (r && r === Ie(e, o.endContainer, ["a"]))
        return r.setAttribute("href", t), r.setAttribute("contenteditable", "false"), r.setAttribute("data-autodown-link", ""), !0;
      const s = e.ownerDocument.createElement("a");
      return s.setAttribute("href", t), s.setAttribute("contenteditable", "false"), s.setAttribute("data-autodown-link", ""), un(o, s), !0;
    }
    const l = Ct[n];
    if (l == null) return !1;
    const i = e.ownerDocument.createElement(l);
    return un(o, i), !0;
  },
  removeMark(n) {
    const t = Ne;
    if (!t) return !1;
    const e = it(t);
    if (!e) return !1;
    if (n === E.Link) {
      const i = Ie(t, e.startContainer, ["a"]);
      if (i && i === Ie(t, e.endContainer, ["a"])) {
        const r = i.parentNode;
        if (r) {
          for (; i.firstChild; ) r.insertBefore(i.firstChild, i);
          r.removeChild(i);
        }
        return !0;
      }
      return !1;
    }
    const o = Ct[n];
    if (o == null) return !1;
    const l = rn[o];
    return an(t, e, l) ? ($l(Ie(t, e.startContainer, l)), !0) : !1;
  }
};
function Ee(n, t) {
  return n.isActive(t) ? n.removeMark(t) : n.applyMark(t);
}
const Dl = {
  setParagraph: h.Paragraph,
  setMathBlock: h.MathBlock,
  setMermaidBlock: h.Mermaid,
  setHorizontalRule: h.ThematicBreak,
  toggleBulletList: h.ListItem,
  toggleOrderedList: h.ListItem,
  toggleBlockquote: h.Blockquote
}, ql = {
  bold: E.Strong,
  strong: E.Strong,
  italic: E.Em,
  em: E.Em,
  strike: E.Del,
  strikethrough: E.Del,
  underline: E.Underline,
  code: E.Code,
  link: E.Link
}, cn = {
  table: h.Table,
  codeBlock: h.Fence,
  fence: h.Fence,
  blockquote: h.Blockquote,
  bulletList: h.ListBlock,
  orderedList: h.ListBlock,
  listItem: h.ListItem,
  heading: h.Heading,
  details: h.Details,
  callout: h.Callout,
  mathBlock: h.MathBlock,
  mermaid: h.Mermaid,
  queryBlock: h.QueryBlock,
  blockEmbed: h.BlockEmbed
};
function Et(n, t, e) {
  if (n.id === t)
    return e.add(n.kind), !0;
  for (const o of n.children)
    if (Et(o, t, e))
      return e.add(n.kind), !0;
  return !1;
}
function dn(n) {
  const t = {};
  for (const e of n) {
    const o = e.value;
    t[e.key] = o != null && (o._tag === "Str" || o._tag === "Int" || o._tag === "Bool") ? o.value : null;
  }
  return t;
}
function Hl(n, t) {
  return n.anchor.blockId === t.anchor.blockId && n.anchor.offset === t.anchor.offset && n.head.blockId === t.head.blockId && n.head.offset === t.head.offset;
}
function Nl(n) {
  const t = y(0), e = /* @__PURE__ */ new Map();
  let o = n.selection;
  const l = (r) => {
    const s = e.get(r);
    if (s)
      for (const u of [...s]) u();
  };
  return n.onChange((r) => {
    t.value++, Hl(r.selection, o) || (o = r.selection, l("selectionUpdate"));
  }), {
    storage: { "slash-command": { query: "", range: null, handled: !1 } },
    isEditable: !0,
    on: (r, s) => {
      let u = e.get(r);
      u || (u = /* @__PURE__ */ new Set(), e.set(r, u)), u.add(s);
    },
    off: (r, s) => {
      var u;
      (u = e.get(r)) == null || u.delete(s);
    },
    isActive: (r) => {
      t.value;
      const s = ql[r];
      if (s != null) return Me(co(n, n.selection), s);
      const u = cn[r];
      if (u == null) return !1;
      const d = /* @__PURE__ */ new Set();
      return Et(n.doc, n.selection.anchor.blockId, d), d.has(u);
    },
    getAttributes: (r) => {
      t.value;
      const s = cn[r];
      if (s == null) return {};
      const u = q(n.doc, n.selection.anchor.blockId);
      if (u && u.kind === s) return dn(u.attrs);
      if (u) {
        const d = /* @__PURE__ */ new Set();
        if (Et(n.doc, u.id, d) && d.has(s)) {
          let c = u;
          for (; c; ) {
            if (c.kind === s) return dn(c.attrs);
            c = X(n.doc, c.id) ?? null;
          }
        }
      }
      return {};
    },
    view: {
      get dom() {
        return typeof document > "u" ? null : document.querySelector(".autodown-editor-content");
      },
      get state() {
        return {
          selection: {
            from: n.selection.anchor.offset,
            to: n.selection.head.offset
          }
        };
      },
      nodeDOM(r) {
        if (typeof document > "u") return null;
        const s = document.querySelector(".autodown-editor-content");
        if (!s) return null;
        const u = n.selection.anchor.blockId;
        for (const d of s.querySelectorAll("[data-block-id]"))
          if (d.dataset.blockId === u) return d;
        return null;
      },
      /** Caret viewport coords (plan 028 P3T1, 021-F5): the focused rich
       *  host's char offset → blockRangeToDomRange → first client rect
       *  (whole-host rect fallback). ProseMirror coordsAtPos shape — the
       *  generated floating menus (SlashMenu two-stage positioning) consume
       *  it to open at the caret instead of the default corner. */
      coordsAtPos(r) {
        if (typeof document > "u") return null;
        const s = n.selection.anchor.blockId, u = Fn() ?? document.querySelector(`.autodown-block-host[data-block-id="${s}"]`);
        if (!u) return null;
        const d = Al(u, r, r), c = d.getClientRects(), f = c.length > 0 ? c[0] : d.getBoundingClientRect();
        return { top: f.top, left: f.left, right: f.right, bottom: f.bottom };
      }
    },
    chain: () => Pl(n),
    __engine: n
  };
}
function Le(n) {
  const t = n.selection.anchor.blockId, e = q(n.doc, t);
  if (!e) return null;
  if (e.kind === h.Table) return { tableId: t, rowId: null, rowIdx: null, colIdx: null };
  if (e.kind !== h.TableCell) return null;
  const o = X(n.doc, t);
  if (!o || o.kind !== h.TableRow) return null;
  const l = X(n.doc, o.id);
  return !l || l.kind !== h.Table ? null : { tableId: l.id, rowId: o.id, rowIdx: Ye(l, o.id), colIdx: Ye(o, t) };
}
function Qe(n, t) {
  var e;
  return ((e = q(n.doc, t)) == null ? void 0 : e.children) ?? [];
}
function Pl(n) {
  const t = [], e = {
    focus: () => e,
    run: () => (n.applyTree((l) => t.reduce((i, r) => r(i), l)), !q(n.doc, n.selection.anchor.blockId) && n.doc.children[0] && n.select(Be(n.doc.children[0].id, 0)), !0),
    setHeading: (l) => (t.push((i) => ze(i, n, h.Heading, [{ key: "level", value: se.Int((l == null ? void 0 : l.level) ?? 1) }])), e),
    insertContent: (l) => {
      const i = String(l ?? "");
      return t.push((r) => {
        if (!i.includes(`
`)) return xt(r, n, i);
        const u = Dt(i, !0).children;
        if (u.length === 0) return r;
        const d = Oe(n), c = q(r, d);
        if (!c) return r;
        const f = r.children, v = f.findIndex((m) => m.id === d), k = Y(c) === "" ? [] : [c], T = [...f.slice(0, v), ...k, ...u, ...f.slice(v + 1)];
        return F(r, T);
      }), e;
    },
    deleteRange: (l) => (t.push((i) => {
      const r = Oe(n), s = q(i, r);
      if (!s) return i;
      const u = Y(s), d = Math.max(0, Math.min(l.from, u.length)), c = Math.max(d, Math.min(l.to, u.length));
      return W(i, r, [Ke(r, s.kind, u.slice(0, d) + u.slice(c))]);
    }), e),
    insertTable: (l) => (t.push((i) => xt(i, n, `| a | b |
| --- | --- |
|  |  |
|  |  |`)), e),
    setImage: (l) => {
      const i = String((l == null ? void 0 : l.src) ?? ""), r = String((l == null ? void 0 : l.alt) ?? "");
      return t.push((s) => xt(s, n, `![${r}](${i})`)), e;
    },
    // table verbs (plan 026 P0T3): forward to the commands.ts table transforms,
    // resolved against the focused cell (table-level focus takes the
    // table-ends defaults); tree-level so a chain stays ONE undo.
    addRowAfter: () => {
      var s;
      const l = Le(n);
      if (!l) return e;
      const i = Qe(n, l.tableId), r = l.rowId ?? ((s = i[i.length - 1]) == null ? void 0 : s.id) ?? null;
      return t.push((u) => en(u, l.tableId, r)), e;
    },
    addRowBefore: () => {
      const l = Le(n);
      if (!l) return e;
      const i = Qe(n, l.tableId), r = l.rowIdx != null && l.rowIdx > 0 ? i[l.rowIdx - 1].id : null;
      return t.push((s) => en(s, l.tableId, r)), e;
    },
    deleteRow: () => {
      const l = Le(n);
      if (!l) return e;
      const i = Qe(n, l.tableId);
      if (i.length <= 1) return e;
      const r = l.rowId ?? i[i.length - 1].id;
      return t.push((s) => W(s, r, [])), e;
    },
    addColumnBefore: () => {
      const l = Le(n);
      return l && t.push((i) => Tt(i, l.tableId, l.colIdx ?? 0)), e;
    },
    addColumnAfter: () => {
      var r;
      const l = Le(n);
      if (!l) return e;
      const i = ((r = Qe(n, l.tableId)[0]) == null ? void 0 : r.children.length) ?? 0;
      return t.push((s) => Tt(s, l.tableId, l.colIdx != null ? l.colIdx + 1 : i)), e;
    },
    deleteColumn: () => {
      var s;
      const l = Le(n);
      if (!l) return e;
      const i = Qe(n, l.tableId), r = Math.max(0, (((s = i[0]) == null ? void 0 : s.children.length) ?? 1) - 1);
      return t.push((u) => fo(u, l.tableId, l.colIdx ?? r)), e;
    },
    deleteTable: () => {
      const l = Le(n);
      return l && t.push((i) => W(i, l.tableId, [])), e;
    },
    // code language channel (plan 026 P0T3): setBlockAttrs on the focused
    // Fence (023's IAL ruling); converts the kind when not a Fence yet.
    setCodeBlockLanguage: (l) => (t.push((i) => ze(i, n, h.Fence, [{ key: "language", value: se.Str(String(l ?? "")) }])), e),
    setCodeBlock: (l) => (l == null ? void 0 : l.language) != null ? e.setCodeBlockLanguage(l.language) : (t.push((i) => ze(i, n, h.Fence)), e),
    // slash manifest's Details template carries { summary } (plan 026 P2T3):
    // kind conversion + summary attr so the mounted node-view shows it.
    // Converting an inline leaf moves its text into a child paragraph — a
    // Details renders children, inlines would serialize away (data loss).
    setDetails: (l) => (t.push((i) => {
      const r = Oe(n), s = q(i, r);
      if (!s) return i;
      const u = s.children.length > 0 ? s.children : Y(s).length > 0 ? [Ke(`${r}-p`, h.Paragraph, Y(s))] : [];
      let d = { ...s, kind: h.Details, children: u };
      return (l == null ? void 0 : l.summary) != null && (d = { ...d, attrs: He(d.attrs, "summary", se.Str(String(l.summary))) }), W(i, r, [d]);
    }), e),
    // slash Callout template carries { type, title } (plan 030 T7): same
    // conversion shape as setDetails — before this the kind-only KIND_COMMANDS
    // path silently dropped both attrs (the lost-title roundtrip break).
    setCallout: (l) => (t.push((i) => {
      const r = Oe(n), s = q(i, r);
      if (!s) return i;
      const u = s.children.length > 0 ? s.children : Y(s).length > 0 ? [Ke(`${r}-p`, h.Paragraph, Y(s))] : [];
      let d = { ...s, kind: h.Callout, children: u };
      return (l == null ? void 0 : l.type) != null && (d = { ...d, attrs: He(d.attrs, "type", se.Str(String(l.type))) }), (l == null ? void 0 : l.title) != null && (d = { ...d, attrs: He(d.attrs, "title", se.Str(String(l.title))) }), W(i, r, [d]);
    }), e),
    // task list (plan 030 T7): a real verb distinct from toggleBulletList —
    // the focused ListItem (a caret usually sits on its child paragraph, so
    // resolve the ListItem ancestor first — the list-commands 选中定位
    // discipline) gains/loses the `checked` attr (task ⇄ plain bullet);
    // outside a list it converts like the bullet verb.
    toggleTaskList: () => (t.push((l) => {
      const i = Oe(n);
      let r = q(l, i);
      for (; r != null && r.kind !== h.ListItem; )
        r = X(l, r.id);
      if (r == null) return ze(l, n, h.ListItem);
      const u = ct(r.attrs, "checked") != null ? r.attrs.filter((d) => d.key !== "checked") : He(r.attrs, "checked", se.Bool(!1));
      return W(l, r.id, [{ ...r, attrs: u }]);
    }), e),
    // inline mark toggles (plan 024 P3T1; adapter-routed plan 036 T3): wrap
    // the FOCUSED host's live DOM through the SelectionAdapter — the model
    // catches up on the blur writeback. No focused host → no-op.
    toggleBold: () => (Ee(ue, E.Strong), e),
    toggleItalic: () => (Ee(ue, E.Em), e),
    toggleStrike: () => (Ee(ue, E.Del), e),
    toggleCode: () => (Ee(ue, E.Code), e),
    // underline (plan 028 P2T2): same DOM-wrap protocol as the others —
    // the model catches up on the blur writeback (u → Mark.Underline)
    toggleUnderline: () => (Ee(ue, E.Underline), e),
    setLink: (l) => {
      const i = String((l == null ? void 0 : l.href) ?? "");
      return i ? ue.applyMark(E.Link, i) : ue.removeMark(E.Link), e;
    },
    unsetLink: () => (ue.removeMark(E.Link), e)
  }, o = e;
  for (const [l, i] of Object.entries(Dl))
    o[l] = () => (t.push((r) => ze(r, n, i)), e);
  return e;
}
function Oe(n) {
  var t;
  return n.selection.anchor.blockId || ((t = n.doc.children[0]) == null ? void 0 : t.id) || "";
}
function ze(n, t, e, o) {
  const l = Oe(t), i = q(n, l);
  if (!i) return n;
  let r = { ...i, kind: e };
  if (o)
    for (const s of o) r = { ...r, attrs: He(r.attrs, s.key, s.value) };
  return W(n, l, [r]);
}
function xt(n, t, e) {
  const o = Oe(t), l = q(n, o);
  if (!l) return n;
  const i = Y(l) + e;
  return W(n, o, [Ke(o, l.kind, i)]);
}
function Fl(n, t) {
  const o = n.slice(0, t).match(/(?:^|\s)\/([^\s/]*)$/);
  return o ? o[1] : null;
}
function Kl(n, t, e) {
  if (n == null) {
    document.dispatchEvent(new CustomEvent("autodown:slash-close", { detail: {} }));
    return;
  }
  const o = {
    query: n,
    range: { from: e - n.length - 1, to: e },
    items: [],
    blockId: t
  };
  document.dispatchEvent(new CustomEvent("autodown:slash-open", { detail: o }));
}
function Ul(n, t) {
  return Kn(n, t).tag;
}
function Wl(n, t) {
  const e = Kn(n, t);
  return e.cls ? `autodown-block-host ${e.cls}` : "autodown-block-host";
}
function Kn(n, t) {
  if (n === "Heading") {
    const e = Math.min(6, Math.max(1, t ?? 1));
    return { tag: `h${e}`, cls: `heading-node heading-${e}` };
  }
  return n === "Paragraph" ? { tag: "p", cls: "paragraph-node" } : { tag: "div", cls: "" };
}
const vt = /* @__PURE__ */ new WeakSet();
function Vl(n) {
  var o;
  const t = Jn(), e = ((o = t == null ? void 0 : t.proxy) == null ? void 0 : o.$el) ?? null;
  e && (vt.add(e), e.innerHTML = n, e.focus(), Un(e), Ge(() => {
    vt.delete(e), Fn() === e && Ut(null);
  }));
}
function Un(n) {
  const t = document.createRange();
  t.selectNodeContents(n), t.collapse(!1);
  const e = window.getSelection();
  e == null || e.removeAllRanges(), e == null || e.addRange(t);
}
function pt(n) {
  return (n.textContent ?? "").replace(/\u00A0/g, " ");
}
function Xe(n) {
  const t = window.getSelection();
  if (!t || t.rangeCount === 0) return 0;
  const e = t.getRangeAt(0).cloneRange();
  return e.selectNodeContents(n), e.setEnd(t.getRangeAt(0).endContainer, t.getRangeAt(0).endOffset), e.toString().length;
}
function Ql(n) {
  const t = n.previousElementSibling;
  return (t == null ? void 0 : t.dataset.blockId) ?? null;
}
function zl(n, t) {
  const e = pt(n);
  t.onInput(e), !t.composition.composing && pt(n) !== t.text && (n.innerHTML = Dn(t.inlines), Un(n)), typeof document < "u" && Kl(Fl(t.text, Xe(n)), t.id, Xe(n));
}
function Gl(n, t) {
  if (t.composition.composing) return;
  const e = n.currentTarget ?? n.target;
  if (n.ctrlKey || n.metaKey) {
    const o = n.key.toLowerCase();
    if (o === "b") {
      n.preventDefault(), Ee(ue, E.Strong);
      return;
    }
    if (o === "i") {
      n.preventDefault(), Ee(ue, E.Em);
      return;
    }
    if (o === "k") {
      n.preventDefault();
      const l = window.prompt("Enter URL");
      l && ue.applyMark(E.Link, l);
      return;
    }
  }
  if (n.key === "Enter")
    n.preventDefault(), t.onEnter(Xe(e), `b-${Math.random().toString(36).slice(2, 8)}`);
  else if (n.key === "Backspace" && Xe(e) === 0) {
    const o = Ql(e);
    o && n.preventDefault(), t.onBackspaceAtStart(o);
  } else n.key === "Tab" && t.onTab(n.shiftKey) && n.preventDefault();
}
function jl(n, t) {
  var l;
  const e = ((l = n.clipboardData) == null ? void 0 : l.getData("text/plain")) ?? "";
  if (!e) return;
  n.preventDefault();
  const o = e.trim();
  if (!o.includes(`
`) && !/^[#>*`\-\d]/.test(o)) {
    t.onInput(t.text + o);
    return;
  }
  t.onPasteMarkdown(o);
}
function Yl(n, t) {
  t.compositionBegin(t.text, Xe(n));
}
function Xl(n, t) {
  t.compositionUpdate(n.data ?? "");
}
function Jl(n, t) {
  t.compositionCommit(pt(n));
}
function Zl(n, t) {
  vt.has(n) && Ut(n);
}
function ei(n, t) {
  if (Ut(null), !vt.has(n)) return;
  const e = pt(n);
  e !== t.text && t.onInput(e), t.onRichBlur(n);
}
const ti = /* @__PURE__ */ Z({
  __name: "RichTextHost",
  props: {
    controller: {},
    blockId: {},
    blockKind: {},
    level: {},
    initial_html: {}
  },
  emits: ["Init", "ClickStop", "Input", "Keydown", "Paste", "Focus", "Blur", "CompositionStart", "CompositionUpdate", "CompositionEnd"],
  setup(n, { emit: t }) {
    const e = n, o = g(() => Ul(e.blockKind, e.level)), l = g(() => Wl(e.blockKind, e.level)), i = t;
    function r(m) {
      ei(m.target, e.controller), i("Blur", m);
    }
    function s(m) {
      i("ClickStop", m);
    }
    function u(m) {
      Jl(m.target, e.controller), i("CompositionEnd", m);
    }
    function d(m) {
      Yl(m.target, e.controller), i("CompositionStart", m);
    }
    function c(m) {
      Xl(m, e.controller), i("CompositionUpdate", m);
    }
    function f(m) {
      Zl(m.target, e.controller), i("Focus", m);
    }
    function v(m) {
      zl(m.target, e.controller), i("Input", m);
    }
    function k(m) {
      Gl(m, e.controller), i("Keydown", m);
    }
    function T(m) {
      jl(m, e.controller), i("Paste", m);
    }
    return ge(() => {
      Vl(e.initial_html);
    }), (m, w) => (b(), J(pe(o.value), {
      class: $e(l.value),
      contenteditable: !0,
      "data-block-id": n.blockId,
      "data-node-type": n.blockKind,
      dir: "auto",
      spellcheck: "false",
      onBlur: w[0] || (w[0] = (O) => r(O)),
      onClick: w[1] || (w[1] = Pe((O) => s(O), ["stop"])),
      onCompositionend: w[2] || (w[2] = (O) => u(O)),
      onCompositionstart: w[3] || (w[3] = (O) => d(O)),
      onCompositionupdate: w[4] || (w[4] = (O) => c(O)),
      onFocus: w[5] || (w[5] = (O) => f(O)),
      onInput: w[6] || (w[6] = (O) => v(O)),
      onKeydown: w[7] || (w[7] = (O) => k(O)),
      onPaste: w[8] || (w[8] = (O) => T(O))
    }, null, 40, ["class", "data-block-id", "data-node-type"]));
  }
}), ni = {
  class: "blockquote",
  dir: "auto"
}, oi = {
  key: 0,
  class: "markdown-renderer"
}, li = /* @__PURE__ */ Z({
  __name: "BlockquoteBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    children: {},
    version: {}
  },
  setup(n) {
    const t = n, e = g(() => t.mode === "edit");
    return (o, l) => (b(), R("blockquote", ni, [
      e.value ? (b(), R("div", oi, [
        (b(), J(de(dt), {
          children_slot: n.children,
          key: "BlockChildren-1"
        }, null, 8, ["children_slot"]))
      ])) : $("", !0),
      e.value ? $("", !0) : (b(), J(de(dt), {
        children_slot: n.children,
        key: "BlockChildren-2"
      }, null, 8, ["children_slot"]))
    ]));
  }
});
function ii(n) {
  return n.startsWith("bottom") ? { vertical: "bottom", horizontal: n.endsWith("end") ? "right" : "left" } : { vertical: "top", horizontal: n.endsWith("end") ? "right" : "left" };
}
function Ce(n, t, e, o, l = "bottom", i = 8, r = "left") {
  const { vertical: s, horizontal: u } = ii(l);
  let d;
  if (s === "bottom") {
    if (d = n.bottom + i, d + e > o.height) {
      const f = n.top - e - i;
      f >= 0 ? d = f : d = Math.max(0, o.height - e);
    }
  } else if (d = n.top - e - i, d < 0) {
    const f = n.bottom + i;
    f + e <= o.height ? d = f : d = 0;
  }
  let c = u === "right" || r === "right" ? n.right - t : n.left;
  return c + t > o.width && (c = Math.max(0, o.width - t)), c < 0 && (c = 0), { top: d, left: c };
}
const ri = {
  bold: Do,
  italic: $o,
  underline: Ao,
  strike: Oo,
  code: On,
  link: En
};
function qe(n) {
  return ri[n];
}
function si({
  editor: n,
  state: t
}) {
  const { empty: e } = t.selection;
  return !(!n.isEditable || e || n.isActive("image"));
}
function ai(n, t) {
  var e, o, l, i;
  if (n.isActive("link"))
    (o = (e = n.chain().focus()).unsetLink) == null || o.call(e).run();
  else {
    const r = window.prompt(t ?? "Enter URL");
    r && ((i = (l = n.chain().focus()).setLink) == null || i.call(l, { href: r }).run());
  }
}
function ui(n) {
  const t = n.selection;
  return { selection: { empty: t.anchor.blockId === t.head.blockId && t.anchor.offset === t.head.offset } };
}
const ci = Z({
  name: "EngineBubbleMenu",
  props: {
    editor: { type: Object, default: null },
    options: { type: Object, default: null },
    shouldShow: { type: Function, default: null }
  },
  setup(n, { slots: t }) {
    const e = y(!1), o = y("0px"), l = y("0px"), i = y(null);
    let r = null;
    const s = () => {
      var j, Q, ne, C;
      const f = (j = n.editor) == null ? void 0 : j.__engine;
      if (!f) {
        e.value = !1;
        return;
      }
      const v = typeof window > "u" ? null : window.getSelection(), k = v && v.rangeCount > 0 && !v.getRangeAt(0).collapsed ? v.getRangeAt(0) : null;
      if (!k) {
        e.value = !1;
        return;
      }
      const T = { editor: n.editor, state: ui(f) };
      if (e.value = n.shouldShow ? !!n.shouldShow(T) : !1, !e.value) return;
      const m = (Q = k.startContainer.nodeType === 3 ? k.startContainer.parentElement : k.startContainer) == null ? void 0 : Q.closest(".autodown-block-host"), w = m == null ? void 0 : m.closest(".autodown-editor");
      if (!m || !w || !w.contains(m)) {
        e.value = !1;
        return;
      }
      const O = k.getBoundingClientRect(), V = w.getBoundingClientRect(), fe = {
        top: O.top - V.top,
        left: O.left - V.left,
        bottom: O.bottom - V.top,
        right: O.right - V.left,
        width: O.width,
        height: O.height
      }, ie = Ce(
        fe,
        ((ne = i.value) == null ? void 0 : ne.offsetWidth) ?? 0,
        ((C = i.value) == null ? void 0 : C.offsetHeight) ?? 0,
        { width: w.clientWidth, height: w.clientHeight },
        "top"
      );
      l.value = `${ie.left}px`, o.value = `${ie.top}px`;
    }, u = (f) => {
      var v, k;
      e.value && !((k = (v = f.target) == null ? void 0 : v.closest) != null && k.call(v, ".autodown-bubble-menu")) && (e.value = !1);
    }, d = (f) => {
      f.key === "Escape" && e.value && (e.value = !1);
    }, c = () => s();
    return ge(() => {
      var v;
      const f = (v = n.editor) == null ? void 0 : v.__engine;
      f && (f.onChange(c), r = () => {
      }), document.addEventListener("pointerdown", u), document.addEventListener("keydown", d), document.addEventListener("selectionchange", c), s();
    }), Ge(() => {
      r == null || r(), document.removeEventListener("pointerdown", u), document.removeEventListener("keydown", d), document.removeEventListener("selectionchange", c);
    }), () => {
      var f;
      return e.value ? te(
        "div",
        {
          ref: i,
          class: "autodown-bubble-menu",
          style: { position: "absolute", top: o.value, left: l.value },
          // plan 024 P3T2: preventDefault keeps the contenteditable
          // host focused (and its selection alive) through button
          // clicks — the mark chains wrap the live host DOM.
          onMousedown: (v) => v.preventDefault()
        },
        (f = t.default) == null ? void 0 : f.call(t)
      ) : null;
    };
  }
}), di = ["title", "onClick"], fi = /* @__PURE__ */ Z({
  __name: "BubbleMenu",
  props: {
    editor: {},
    linkPrompt: { default: "Enter URL" },
    tooltips: { default: null }
  },
  emits: ["RunButton"],
  setup(n, { emit: t }) {
    const e = n, o = g(() => [{ name: "bold", title: e.tooltips && e.tooltips.bold || "Bold", icon: qe("bold"), active: e.editor.isActive("bold"), action: () => e.editor.chain().focus().toggleBold().run() }, { name: "italic", title: e.tooltips && e.tooltips.italic || "Italic", icon: qe("italic"), active: e.editor.isActive("italic"), action: () => e.editor.chain().focus().toggleItalic().run() }, { name: "underline", title: e.tooltips && e.tooltips.underline || "Underline", icon: qe("underline"), active: e.editor.isActive("underline"), action: () => e.editor.chain().focus().toggleUnderline().run() }, { name: "strike", title: e.tooltips && e.tooltips.strike || "Strikethrough", icon: qe("strike"), active: e.editor.isActive("strike"), action: () => e.editor.chain().focus().toggleStrike().run() }, { name: "code", title: e.tooltips && e.tooltips.code || "Inline Code", icon: qe("code"), active: e.editor.isActive("code"), action: () => e.editor.chain().focus().toggleCode().run() }, { name: "link", title: e.tooltips && e.tooltips.link || "Link", icon: qe("link"), active: e.editor.isActive("link"), action: () => ai(e.editor, e.linkPrompt) }]), l = t;
    function i(r) {
      r.action(), l("RunButton", r);
    }
    return (r, s) => n.editor ? (b(), J(de(ci), {
      class: $e("autodown-bubble-menu"),
      editor: n.editor,
      options: { placement: "top" },
      shouldShow: de(si),
      key: "TiptapBubbleMenu-1"
    }, {
      default: st(() => [
        (b(!0), R(xe, null, Je(o.value, (u) => (b(), R("button", {
          class: $e(["autodown-bubble-btn", { active: u.active }]),
          key: u.title,
          title: u.title,
          onClick: (d) => i(u)
        }, [
          (b(), J(pe(u.icon), { size: 14 }))
        ], 10, di))), 128))
      ]),
      _: 1
    }, 8, ["editor", "shouldShow"])) : $("", !0);
  }
});
function hi() {
  return qo;
}
const mi = [
  { id: "text", label: "Text", aliases: [] },
  { id: "bash", label: "Bash", aliases: ["sh", "shell", "zsh"] },
  { id: "c", label: "C", aliases: [] },
  { id: "cpp", label: "C++", aliases: ["c++", "cxx"] },
  { id: "csharp", label: "C#", aliases: ["c#", "cs"] },
  { id: "css", label: "CSS", aliases: [] },
  { id: "dockerfile", label: "Dockerfile", aliases: ["docker"] },
  { id: "go", label: "Go", aliases: ["golang"] },
  { id: "html", label: "HTML", aliases: [] },
  { id: "java", label: "Java", aliases: [] },
  { id: "javascript", label: "JavaScript", aliases: ["js"] },
  { id: "json", label: "JSON", aliases: [] },
  { id: "kotlin", label: "Kotlin", aliases: ["kt"] },
  { id: "lua", label: "Lua", aliases: [] },
  { id: "markdown", label: "Markdown", aliases: ["md"] },
  { id: "php", label: "PHP", aliases: [] },
  { id: "python", label: "Python", aliases: ["py"] },
  { id: "r", label: "R", aliases: [] },
  { id: "ruby", label: "Ruby", aliases: ["rb"] },
  { id: "rust", label: "Rust", aliases: ["rs"] },
  { id: "scss", label: "SCSS", aliases: ["sass"] },
  { id: "sql", label: "SQL", aliases: [] },
  { id: "swift", label: "Swift", aliases: [] },
  { id: "toml", label: "TOML", aliases: [] },
  { id: "typescript", label: "TypeScript", aliases: ["ts", "tsx"] },
  { id: "xml", label: "XML", aliases: [] },
  { id: "yaml", label: "YAML", aliases: ["yml"] }
];
function fn() {
  return mi;
}
const vi = { class: "autodown-codeblock-menu-header" }, pi = ["onKeydown"], gi = ["onClick", "onMouseenter"], ki = { class: "autodown-codeblock-menu-item-label" }, bi = {
  key: 0,
  class: "autodown-codeblock-menu-empty"
}, wi = /* @__PURE__ */ Z({
  __name: "CodeBlockMenu",
  props: {
    editor: {}
  },
  emits: ["Init", "Destroy", "SearchInput", "MoveDown", "MoveUp", "SelectHighlighted", "SelectItem", "HoverItem", "Close", "OutsideClick"],
  setup(n, { emit: t }) {
    const e = n, o = y(!1), l = y(""), i = y(0), r = y(""), s = y(""), u = y(""), d = y(""), c = y(null), f = y(null), v = y(null), k = y(null), T = y(0), m = y(null), w = y(null), O = y(null), V = y(null), fe = y(null), ie = y(null), j = y(null), Q = g(() => fn().filter((a) => [a.id, a.label].concat(a.aliases).join(" ").toLowerCase().includes(l.value.toLowerCase().trim()))), ne = g(() => Q.value.length === 0), C = g(() => hi()), L = t;
    function M() {
      o.value = !1, l.value = "", i.value = 0, c.value = null, L("Close");
    }
    function x(a) {
      i.value = a, L("HoverItem", a);
    }
    function B() {
      i.value < Q.value.length - 1 && (i.value = i.value + 1, ce(() => {
        let a = null;
        if (f.value != null && (a = f.value.querySelector(".autodown-codeblock-menu")), a != null) {
          let p = a.querySelector(".autodown-codeblock-menu-list"), I = a.querySelector(".autodown-codeblock-menu-item.active");
          if (p != null && I != null) {
            let D = p.getBoundingClientRect(), N = I.getBoundingClientRect(), S = N.top - D.top - D.height / 2 + N.height / 2;
            p.scrollTop = p.scrollTop + S;
          }
        }
      })), L("MoveDown");
    }
    function ee() {
      i.value > 0 && (i.value = i.value - 1, ce(() => {
        let a = null;
        if (f.value != null && (a = f.value.querySelector(".autodown-codeblock-menu")), a != null) {
          let p = a.querySelector(".autodown-codeblock-menu-list"), I = a.querySelector(".autodown-codeblock-menu-item.active");
          if (p != null && I != null) {
            let D = p.getBoundingClientRect(), N = I.getBoundingClientRect(), S = N.top - D.top - D.height / 2 + N.height / 2;
            p.scrollTop = p.scrollTop + S;
          }
        }
      })), L("MoveUp");
    }
    function be(a) {
      if (o.value) {
        let p = null;
        f.value != null && (p = f.value.querySelector(".autodown-codeblock-menu")), p != null && (p.contains(a.target) || (o.value = !1, l.value = "", i.value = 0, c.value = null));
      }
      L("OutsideClick", a);
    }
    function me(a) {
      l.value = a.target.value, L("SearchInput", a);
    }
    function we() {
      if (Q.value.length == 1) {
        let a = Q.value[0];
        e.editor.chain().focus().setCodeBlock({ language: a.id }).run(), o.value = !1, l.value = "", i.value = 0, c.value = null;
      }
      if (Q.value.length != 1) {
        let a = Q.value[i.value];
        a != null && (e.editor.chain().focus().setCodeBlock({ language: a.id }).run(), o.value = !1, l.value = "", i.value = 0, c.value = null);
      }
      L("SelectHighlighted");
    }
    function ye(a) {
      e.editor.chain().focus().setCodeBlock({ language: a.id }).run(), o.value = !1, l.value = "", i.value = 0, c.value = null, L("SelectItem", a);
    }
    ge(() => {
      let a = e.editor.view.dom;
      v.value = a, f.value = a.closest(".autodown-editor"), k.value = a.closest(".autodown-editor-content-wrapper");
      let p = () => {
        T.value != 0 && cancelAnimationFrame(T.value), T.value = requestAnimationFrame(() => {
          if (T.value = 0, o.value && f.value != null) {
            let S = f.value.getBoundingClientRect(), A = c.value;
            if (A == null) {
              let H = e.editor.view, G = H.nodeDOM(H.state.selection.from);
              G != null && G.closest != null && (A = G.closest("pre[data-language]"), A == null && (A = G.closest(".autodown-codeblock-node")));
            }
            if (A == null && (o.value = !1, l.value = "", i.value = 0, c.value = null), A != null) {
              let H = A.querySelector("[data-codeblock-language-badge]"), G = A;
              H != null && (G = H);
              let oe = G.getBoundingClientRect(), Te = { top: oe.top - S.top + 6, left: oe.left - S.left, bottom: oe.bottom - S.top + 6, right: oe.right - S.left, width: oe.width, height: oe.height }, K = { width: S.width, height: S.height }, P = Ce(Te, 0, 0, K, "bottom-end", 0);
              s.value = P.top + "px", u.value = P.left + "px", d.value = "hidden", ce(() => {
                let he = f.value.querySelector(".autodown-codeblock-menu");
                if (he != null) {
                  let Ve = he.getBoundingClientRect(), le = Ce(Te, Ve.width, Ve.height, K, "bottom-end", 0);
                  s.value = le.top + "px", u.value = le.left + "px", d.value = "visible";
                }
              });
            }
          }
        });
      };
      V.value = p;
      let I = (S) => {
        if (o.value) {
          let A = null;
          if (f.value != null && (A = f.value.querySelector(".autodown-codeblock-menu")), A != null && A.contains(S.target)) {
            S.preventDefault(), S.stopPropagation();
            let H = A.querySelector(".autodown-codeblock-menu-list");
            if (H != null) {
              let G = H.scrollTop + H.clientHeight < H.scrollHeight, oe = H.scrollTop > 0;
              S.deltaY > 0 && G && (H.scrollTop = H.scrollTop + S.deltaY), S.deltaY < 0 && oe && (H.scrollTop = H.scrollTop + S.deltaY);
            }
          }
          A == null && (S.preventDefault(), S.stopPropagation()), A != null && !A.contains(S.target) && (S.preventDefault(), S.stopPropagation());
        }
      };
      m.value = I, document.addEventListener("wheel", m.value, { passive: !1, capture: !0 });
      let D = (S) => {
        let A = S.target, H = null, G = null, oe = null, Te = null;
        A.closest != null && (H = A.closest("[data-codeblock-language-badge]"), G = A.closest("[data-codeblock-copy-btn]"), oe = A.closest("[data-codeblock-expand-btn]"), Te = A.closest("[data-codeblock-more-btn]")), (H != null || G != null || oe != null || Te != null) && (S.preventDefault(), S.stopPropagation());
      };
      w.value = D, a.addEventListener("mousedown", w.value, { capture: !0 });
      let N = (S) => {
        let A = S.target, H = null, G = null, oe = null, Te = null;
        if (A.closest != null && (H = A.closest("[data-codeblock-copy-btn]"), G = A.closest("[data-codeblock-expand-btn]"), oe = A.closest("[data-codeblock-language-badge]"), Te = A.closest("[data-codeblock-more-btn]")), H != null) {
          S.preventDefault(), S.stopPropagation();
          let K = H.closest("pre");
          if (K == null) {
            let he = H.closest(".code-block-container");
            he != null && (K = he.querySelector("pre[data-language]"));
          }
          let P = "";
          if (K != null) {
            let he = K.querySelector("code");
            he != null && (P = he.textContent ?? "");
          }
          navigator.clipboard.writeText(P);
        }
        if (H == null && G != null) {
          S.preventDefault(), S.stopPropagation();
          let K = G.closest("pre");
          if (K == null) {
            let P = G.closest(".code-block-container");
            P != null && (K = P.querySelector("pre[data-language]"));
          }
          K != null && K.classList.toggle("is-collapsed");
        }
        if (H == null && G == null) {
          let K = oe;
          if (K == null && (K = Te), K != null) {
            let P = K.closest("pre");
            P == null && (P = K.closest(".autodown-codeblock-node"));
            let he = !1;
            if (P == null && (P = K.closest(".code-block-container"), P != null && (he = !0)), S.preventDefault(), he == !1 && S.stopPropagation(), P == null && (P = K.closest(".code-block-container")), P == null) {
              let le = e.editor.view, z = le.nodeDOM(le.state.selection.from);
              z != null && z.closest != null && (P = z.closest("pre[data-language]"), P == null && (P = z.closest(".autodown-codeblock-node")));
            }
            if (c.value = P, r.value = "", c.value != null && (r.value = c.value.getAttribute("data-language") ?? "", r.value == "")) {
              let le = c.value.querySelector("pre[data-language]");
              le != null && (r.value = le.getAttribute("data-language") ?? "");
            }
            r.value == "" && (r.value = e.editor.getAttributes("codeBlock").language ?? ""), o.value = !0, l.value = "";
            let Ve = fn().findIndex((le) => le.id == r.value);
            i.value = Ve, Ve < 0 && (i.value = 0), ce(() => {
              let le = null;
              if (f.value != null && (le = f.value.querySelector(".autodown-codeblock-menu")), le != null) {
                let z = le.querySelector(".autodown-codeblock-menu-search");
                z != null && z.focus();
              }
              if (f.value != null) {
                let z = f.value.getBoundingClientRect(), re = c.value;
                if (re == null) {
                  let Re = e.editor.view, ve = Re.nodeDOM(Re.state.selection.from);
                  ve != null && ve.closest != null && (re = ve.closest("pre[data-language]"), re == null && (re = ve.closest(".autodown-codeblock-node")));
                }
                if (re == null && (o.value = !1, l.value = "", i.value = 0, c.value = null), re != null) {
                  let Re = re.querySelector("[data-codeblock-language-badge]"), ve = re;
                  Re != null && (ve = Re);
                  let _e = ve.getBoundingClientRect(), et = { top: _e.top - z.top + 6, left: _e.left - z.left, bottom: _e.bottom - z.top + 6, right: _e.right - z.left, width: _e.width, height: _e.height }, Vt = { width: z.width, height: z.height }, Qt = Ce(et, 0, 0, Vt, "bottom-end", 0);
                  s.value = Qt.top + "px", u.value = Qt.left + "px", d.value = "hidden", ce(() => {
                    let zt = f.value.querySelector(".autodown-codeblock-menu");
                    if (zt != null) {
                      let Gt = zt.getBoundingClientRect(), jt = Ce(et, Gt.width, Gt.height, Vt, "bottom-end", 0);
                      s.value = jt.top + "px", u.value = jt.left + "px", d.value = "visible";
                    }
                  });
                }
              }
              ce(() => {
                let z = null;
                if (f.value != null && (z = f.value.querySelector(".autodown-codeblock-menu")), z != null) {
                  let re = z.querySelector(".autodown-codeblock-menu-list"), Re = z.querySelector(".autodown-codeblock-menu-item.active");
                  if (re != null && Re != null) {
                    let ve = re.getBoundingClientRect(), _e = Re.getBoundingClientRect(), et = _e.top - ve.top - ve.height / 2 + _e.height / 2;
                    re.scrollTop = re.scrollTop + et;
                  }
                }
              });
            });
          }
        }
      };
      O.value = N, a.addEventListener("click", O.value, { capture: !0 }), k.value != null && k.value.addEventListener("scroll", V.value, { passive: !0 });
    }), St(() => {
      document.removeEventListener("wheel", m.value, { capture: !0 }), v.value != null && (v.value.removeEventListener("mousedown", w.value, { capture: !0 }), v.value.removeEventListener("click", O.value, { capture: !0 })), k.value != null && k.value.removeEventListener("scroll", V.value);
    });
    function We(a) {
      be(a);
    }
    return ge(() => {
      document.addEventListener("mousedown", We);
    }), St(() => {
      document.removeEventListener("mousedown", We);
    }), (a, p) => o.value ? (b(), R("div", {
      key: 0,
      class: "autodown-codeblock-menu",
      ref_key: "menuEl",
      ref: fe,
      style: _n({ top: s.value, left: u.value, visibility: d.value })
    }, [
      _("div", vi, [
        kt(_("input", {
          class: "autodown-codeblock-menu-search",
          placeholder: "Search language…",
          ref_key: "searchEl",
          ref: ie,
          "onUpdate:modelValue": p[0] || (p[0] = (I) => l.value = I),
          onInput: p[1] || (p[1] = (I) => me(I)),
          onKeydown: [
            tt(Pe(B, ["prevent"]), ["down"]),
            tt(Pe(we, ["prevent"]), ["enter"]),
            tt(M, ["esc"]),
            tt(Pe(ee, ["prevent"]), ["up"])
          ]
        }, null, 40, pi), [
          [Ot, l.value]
        ])
      ]),
      _("div", {
        class: "autodown-codeblock-menu-list",
        ref_key: "listEl",
        ref: j
      }, [
        (b(!0), R(xe, null, Je(Q.value, (I, D) => (b(), R("button", {
          class: $e(["autodown-codeblock-menu-item", { active: D == i.value, selected: I.id == r.value }]),
          key: I.id,
          onClick: (N) => ye(I),
          onMouseenter: (N) => x(D)
        }, [
          _("span", ki, [
            _("span", null, U(I.label), 1)
          ]),
          I.id == r.value ? (b(), J(pe(C.value), {
            key: 0,
            class: "autodown-codeblock-menu-check",
            size: 13
          })) : $("", !0)
        ], 42, gi))), 128)),
        ne.value ? (b(), R("div", bi, [...p[2] || (p[2] = [
          _("span", null, "No matching languages", -1)
        ])])) : $("", !0)
      ], 512)
    ], 4)) : $("", !0);
  }
}), yi = { class: "autodown-slash-menu-items" }, _i = ["onClick", "onMouseenter"], Ii = { class: "autodown-slash-menu-info" }, Ci = { class: "autodown-slash-menu-title" }, xi = { class: "autodown-slash-menu-desc" }, Bi = {
  key: 0,
  class: "autodown-slash-menu-empty"
}, Si = /* @__PURE__ */ Z({
  __name: "SlashMenu",
  props: {
    editor: {},
    items: {},
    noResultsText: { default: "No results" }
  },
  emits: ["OnOpen", "OnUpdate", "OnClose", "OnKeydown", "SelectItem", "HoverItem"],
  setup(n, { emit: t }) {
    const e = n, o = y(!1), l = y(""), i = y(null), r = y(0), s = y(""), u = y(""), d = y(""), c = y(null), f = g(() => e.items.filter((C) => [C.title, C.description].concat(C.searchTerms).join(" ").toLowerCase().includes(l.value.toLowerCase()))), v = g(() => f.value.length === 0), k = g(() => e.noResultsText ?? "No results"), T = t;
    De(f, () => {
      r.value = 0;
    });
    function m(C) {
      r.value = C, T("HoverItem", C);
    }
    function w() {
      o.value = !1, l.value = "", i.value = null, r.value = 0, T("OnClose");
    }
    function O(C) {
      if (o.value) {
        if (C.detail.event.key == "ArrowDown") {
          C.detail.event.preventDefault();
          let L = r.value + 1;
          r.value = L % f.value.length, ce(() => {
            if (c.value) {
              let M = c.value.querySelector(".autodown-slash-menu-item.active");
              M != null && M.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (C.detail.event.key == "ArrowUp") {
          C.detail.event.preventDefault();
          let L = r.value - 1 + f.value.length;
          r.value = L % f.value.length, ce(() => {
            if (c.value) {
              let M = c.value.querySelector(".autodown-slash-menu-item.active");
              M != null && M.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (C.detail.event.key == "Enter" || C.detail.event.key == "NumpadEnter") {
          C.detail.event.preventDefault();
          let L = f.value[r.value];
          L != null && i.value != null && (L.command({ editor: e.editor, range: i.value }), o.value = !1, l.value = "", i.value = null, r.value = 0), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        C.detail.event.key == "Escape" && (C.detail.event.preventDefault(), o.value = !1, l.value = "", i.value = null, r.value = 0, e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0));
      }
      T("OnKeydown", C);
    }
    function V(C) {
      l.value = C.detail.query, i.value = C.detail.range, o.value = !0, r.value = 0, ce(() => {
        if (i.value != null && e.editor.view) {
          let L = e.editor.view.coordsAtPos(i.value.from), M = e.editor.view.dom.closest(".autodown-editor");
          if (M != null) {
            let x = M.getBoundingClientRect(), B = { top: L.top - x.top, left: L.left - x.left, bottom: L.bottom - x.top, right: L.right - x.left, width: L.right - L.left, height: L.bottom - L.top }, ee = { width: x.width, height: x.height }, be = Ce(B, 0, 0, ee, "bottom", 8, "left");
            s.value = be.top + "px", u.value = be.left + "px", d.value = "hidden", ce(() => {
              let me = M.querySelector(".autodown-slash-menu");
              if (me != null) {
                let we = me.getBoundingClientRect(), ye = Ce(B, we.width, we.height, ee, "bottom", 8, "left");
                s.value = ye.top + "px", u.value = ye.left + "px", d.value = "visible";
              }
            });
          }
        }
      }), T("OnOpen", C);
    }
    function fe(C) {
      l.value = C.detail.query, i.value = C.detail.range, ce(() => {
        if (i.value != null && e.editor.view) {
          let L = e.editor.view.coordsAtPos(i.value.from), M = e.editor.view.dom.closest(".autodown-editor");
          if (M != null) {
            let x = M.getBoundingClientRect(), B = { top: L.top - x.top, left: L.left - x.left, bottom: L.bottom - x.top, right: L.right - x.left, width: L.right - L.left, height: L.bottom - L.top }, ee = { width: x.width, height: x.height }, be = Ce(B, 0, 0, ee, "bottom", 8, "left");
            s.value = be.top + "px", u.value = be.left + "px", d.value = "hidden", ce(() => {
              let me = M.querySelector(".autodown-slash-menu");
              if (me != null) {
                let we = me.getBoundingClientRect(), ye = Ce(B, we.width, we.height, ee, "bottom", 8, "left");
                s.value = ye.top + "px", u.value = ye.left + "px", d.value = "visible";
              }
            });
          }
        }
      }), T("OnUpdate", C);
    }
    function ie(C) {
      let L = f.value[C];
      L != null && i.value != null && (L.command({ editor: e.editor, range: i.value }), o.value = !1, l.value = "", i.value = null, r.value = 0), T("SelectItem", C);
    }
    function j(C) {
      O(C);
    }
    function Q(C) {
      V(C);
    }
    function ne(C) {
      fe(C);
    }
    return ge(() => {
      document.addEventListener("autodown:slash-close", w), document.addEventListener("autodown:slash-keydown", j), document.addEventListener("autodown:slash-open", Q), document.addEventListener("autodown:slash-update", ne);
    }), St(() => {
      document.removeEventListener("autodown:slash-close", w), document.removeEventListener("autodown:slash-keydown", j), document.removeEventListener("autodown:slash-open", Q), document.removeEventListener("autodown:slash-update", ne);
    }), (C, L) => o.value ? (b(), R("div", {
      key: 0,
      class: "autodown-slash-menu",
      ref_key: "menuEl",
      ref: c,
      style: _n({ top: s.value, left: u.value, visibility: d.value })
    }, [
      _("div", yi, [
        (b(!0), R(xe, null, Je(f.value, (M, x) => (b(), R("button", {
          class: $e(["autodown-slash-menu-item", { active: x == r.value }]),
          key: M.title,
          onClick: (B) => ie(x),
          onMouseenter: (B) => m(x)
        }, [
          (b(), J(pe(M.icon), {
            class: "autodown-slash-menu-icon",
            size: 16
          })),
          _("div", Ii, [
            _("div", Ci, [
              _("span", null, U(M.title), 1)
            ]),
            _("div", xi, [
              _("span", null, U(M.description), 1)
            ])
          ])
        ], 42, _i))), 128)),
        v.value ? (b(), R("div", Bi, [
          _("span", null, U(k.value), 1)
        ])) : $("", !0)
      ])
    ], 4)) : $("", !0);
  }
});
function Ti(n) {
  var o, l;
  const t = n == null ? void 0 : n.__engine, e = (l = (o = t == null ? void 0 : t.selection) == null ? void 0 : o.anchor) == null ? void 0 : l.blockId;
  return !t || !e ? null : ho(t, e);
}
function Ri(n) {
  return [...[
    {
      title: "Text",
      description: "Plain text",
      icon: Ho,
      searchTerms: ["p"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setParagraph().run()
    },
    {
      title: "Heading 1",
      description: "Big section heading",
      icon: No,
      searchTerms: ["h1"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 1 }).run()
    },
    {
      title: "Heading 2",
      description: "Medium section heading",
      icon: Po,
      searchTerms: ["h2"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 2 }).run()
    },
    {
      title: "Heading 3",
      description: "Small section heading",
      icon: Fo,
      searchTerms: ["h3"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 3 }).run()
    },
    {
      title: "Heading 4",
      description: "Fourth level heading",
      icon: Ko,
      searchTerms: ["h4"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 4 }).run()
    },
    {
      title: "Heading 5",
      description: "Fifth level heading",
      icon: Uo,
      searchTerms: ["h5"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 5 }).run()
    },
    {
      title: "Heading 6",
      description: "Sixth level heading",
      icon: Wo,
      searchTerms: ["h6"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 6 }).run()
    },
    {
      title: "Bullet List",
      description: "Bullet list",
      icon: Vo,
      searchTerms: ["ul"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleBulletList().run()
    },
    {
      title: "Numbered List",
      description: "Numbered list",
      icon: Qo,
      searchTerms: ["ol"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleOrderedList().run()
    },
    {
      title: "Task List",
      description: "Task list",
      icon: zo,
      searchTerms: ["task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleTaskList().run()
    },
    {
      title: "Code Block",
      description: "Code snippet",
      icon: On,
      searchTerms: ["code"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setCodeBlock({ language: "text" }).run();
      }
    },
    {
      title: "Quote",
      description: "Quote",
      icon: Go,
      searchTerms: ["blockquote"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleBlockquote().run()
    },
    {
      title: "Divider",
      description: "Horizontal rule",
      icon: jo,
      searchTerms: ["hr"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHorizontalRule().run()
    },
    {
      title: "Image",
      description: "Embed image",
      icon: Yo,
      searchTerms: ["img"],
      command: ({ editor: e, range: o }) => {
        const l = window.prompt(n.imageUrlPrompt);
        l && e.chain().focus().deleteRange(o).setImage({ src: l }).run();
      }
    },
    {
      title: "Table",
      description: "Add table",
      icon: Xo,
      searchTerms: ["table"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertTable({ rows: 3, cols: 3, withHeaderRow: !0 }).run()
    },
    {
      title: "Callout",
      description: "Admonition / callout box",
      icon: Jo,
      searchTerms: ["callout", "admonition", "warning", "tip", "note"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setCallout({ type: "note", title: "Note" }).run();
      }
    },
    {
      title: "Details",
      description: "Collapsible details block",
      icon: Zo,
      searchTerms: ["details", "toggle", "collapse", "accordion"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setDetails({ summary: "Details" }).run();
      }
    },
    {
      title: "Math",
      description: "Block math formula (KaTeX)",
      icon: el,
      searchTerms: ["math", "katex", "formula", "equation", "latex"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setMathBlock().run();
      }
    },
    {
      title: "Mermaid",
      description: "Mermaid diagram",
      icon: tl,
      searchTerms: ["mermaid", "diagram", "chart", "flowchart"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setMermaidBlock().run();
      }
    },
    {
      title: "TODO",
      description: "Insert a TODO task",
      icon: nl,
      searchTerms: ["todo", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- TODO ").run()
    },
    {
      title: "DOING",
      description: "Insert a DOING task",
      icon: ol,
      searchTerms: ["doing", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- DOING ").run()
    },
    {
      title: "DONE",
      description: "Insert a DONE task",
      icon: ll,
      searchTerms: ["done", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- DONE ").run()
    },
    {
      title: "NOW",
      description: "Insert a NOW task",
      icon: il,
      searchTerms: ["now", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- NOW ").run()
    },
    {
      title: "LATER",
      description: "Insert a LATER task",
      icon: rl,
      searchTerms: ["later", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- LATER ").run()
    },
    {
      title: "Priority A",
      description: "Insert [#A] priority",
      icon: It,
      searchTerms: ["priority", "A"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#A] ").run()
    },
    {
      title: "Priority B",
      description: "Insert [#B] priority",
      icon: It,
      searchTerms: ["priority", "B"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#B] ").run()
    },
    {
      title: "Priority C",
      description: "Insert [#C] priority",
      icon: It,
      searchTerms: ["priority", "C"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#C] ").run()
    },
    {
      title: "Query",
      description: "Insert a query macro",
      icon: sl,
      searchTerms: ["query", "macro"],
      command: ({ editor: e, range: o }) => {
        const l = window.prompt("Query (e.g. (task TODO DOING))", "(task TODO)");
        l && e.chain().focus().deleteRange(o).insertContent(`{{query ${l}}}`).run();
      }
    },
    {
      title: "Block link",
      description: "Copy link to current block",
      icon: En,
      searchTerms: ["block link", "anchor", "copy link"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).run();
        const l = n.pageTitle, i = Ti(e);
        if (l && i) {
          const r = `[[${l}#^${i}]]`;
          navigator.clipboard.writeText(r).catch(() => {
          });
        }
      }
    }
  ], ...n.extraSlashItems ?? []];
}
const Wn = "autodown-node-view-content";
Z({
  name: "NodeViewWrapper",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(n, { slots: t, attrs: e }) {
    return () => {
      var o;
      return te(n.as, { ...e, "data-node-view-wrapper": "" }, (o = t.default) == null ? void 0 : o.call(t));
    };
  }
});
Z({
  name: "NodeViewContent",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(n, { slots: t, attrs: e }) {
    const o = Zn(Wn, null);
    return () => {
      var i;
      const l = o ? o() : ((i = t.default) == null ? void 0 : i.call(t)) ?? [];
      return te(n.as, { ...e, "data-node-view-content": "" }, l);
    };
  }
});
let Vn = {};
function Bt(n) {
  Vn = { runQuery: n.runQuery, loadBlock: n.loadBlock };
}
function Wt() {
  return Vn;
}
const gt = [];
function Mi(n) {
  gt.push(n);
}
function Li() {
  gt.pop();
}
function Ei() {
  return gt[gt.length - 1];
}
Z({
  name: "NodeViewContentProvider",
  props: { content: { type: Function, required: !0 } },
  setup(n, { slots: t }) {
    return eo(Wn, () => n.content()), () => {
      var e;
      return (e = t.default) == null ? void 0 : e.call(t);
    };
  }
});
class Oi {
  constructor(t, e) {
    this.engine = t, this.tableId = e;
  }
  table() {
    return q(this.engine.doc, this.tableId);
  }
  get rows() {
    var t;
    return ((t = this.table()) == null ? void 0 : t.children) ?? [];
  }
  cellText(t) {
    const e = q(this.engine.doc, t);
    return e ? Y(e) : "";
  }
  /** Append an empty row after the last row (or after the header when only
   *  the header exists). One undo step. */
  addRow() {
    const t = this.rows[this.rows.length - 1];
    tn(this.engine, this.tableId, (t == null ? void 0 : t.id) ?? null);
  }
  /** Insert an empty row ABOVE the header (TableMenu absorption, plan 026
   *  adjudication #1 — single table entry). One undo step. */
  addRowAbove() {
    tn(this.engine, this.tableId, null);
  }
  /** Insert an empty column at index 0. One undo step. */
  addColumnBefore() {
    this.engine.applyTree((t) => Tt(t, this.tableId, 0));
  }
  /** Remove the whole table; the dangling selection collapses to the first
   *  block (the menu chain's deleteTable repair, same semantics). */
  deleteTable() {
    this.engine.applyTree((t) => W(t, this.tableId, [])), !q(this.engine.doc, this.engine.selection.anchor.blockId) && this.engine.doc.children[0] && this.engine.select(Be(this.engine.doc.children[0].id, 0));
  }
  /** Remove the last row. Refused (no-op) when only the header remains. */
  deleteRow() {
    return this.rows.length <= 1 ? !1 : (mo(this.engine, this.rows[this.rows.length - 1].id), !0);
  }
  /** Append an empty column. One undo step. */
  addColumn() {
    vo(this.engine, this.tableId);
  }
  /** Remove the last column. Refused (no-op) below one column. */
  deleteColumn() {
    var e;
    return (((e = this.rows[0]) == null ? void 0 : e.children.length) ?? 0) <= 1 ? !1 : (po(this.engine, this.tableId), !0);
  }
  /** Cell blur-commit: old→new text as one diff op (BlockHost protocol).
   *  The selection stays anchored on the TABLE — the op's position points at
   *  the cell and applyOp would otherwise drag the anchor into it, dropping
   *  the top-level focus that assembles this editing face (found live in the
   *  demo: committing a cell unmounted the table editor).
   *  Returns false when the text is unchanged or the cell is gone. */
  commitCell(t, e) {
    const o = q(this.engine.doc, t);
    if (!o || o.kind !== h.TableCell) return !1;
    const l = Y(o);
    if (l === e) return !1;
    const i = An(t, l, e);
    if (!i) return !1;
    const r = this.engine.selection;
    return this.engine.apply(i), this.engine.selection.anchor.blockId !== r.anchor.blockId && this.engine.select(r), !0;
  }
}
function hn(n) {
  const t = Rt(n, !0);
  return t.error === "" && Bn("MathBlock", n, { kind: "html", body: t.html, error: "" }), t;
}
function Qn(n) {
  const t = n.split(`
`).length;
  return String(Math.max(4, Math.min(24, t + 1)));
}
const Ai = ["data-block-id", "data-math-block", "data-node-type"], $i = {
  key: 0,
  class: "autodown-stream-banner"
}, Di = { class: "math-editor-stack" }, qi = ["innerHTML"], Hi = {
  key: 1,
  class: "autodown-math-error",
  title: "Math preview error"
}, Ni = ["disabled", "rows"], Pi = ["innerHTML"], Fi = {
  key: 1,
  class: "autodown-math-error",
  title: "Math preview error"
}, Ki = { class: "math-block-source" }, Ui = /* @__PURE__ */ Z({
  __name: "MathBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(n, { emit: t }) {
    const e = n, o = y(""), l = y(""), i = y(ft(e.node)), r = y(Sn(e.ctx)), s = y(null), u = g(() => e.mode === "edit"), d = g(() => ft(e.node)), c = g(() => qt(e.ctx)), f = g(() => u.value ? c.value ? "autodown-math-editor is-readonly" : "autodown-math-editor" : "autodown-math-block"), v = g(() => Tn(e.mode)), k = g(() => ht(e.mode, Ht(e.ctx))), T = g(() => ht(e.mode, "MathBlock")), m = g(() => {
      var M;
      return (M = Rt(i.value, !0)) == null ? void 0 : M.html;
    }), w = g(() => {
      var M;
      return (M = Rt(i.value, !0)) == null ? void 0 : M.error;
    }), O = g(() => !w.value), V = g(() => !!w.value), fe = g(() => Qn(i.value)), ie = g(() => !l.value), j = g(() => !!l.value), Q = g(() => "code"), ne = t;
    De(d, () => {
      if (!u.value) {
        let M = hn(d.value);
        o.value = M.html, l.value = M.error;
      }
    });
    function C(M) {
      i.value = M.target.value, ne("AreaInput", M);
    }
    function L(M) {
      c.value || r.value.commit(M.target.value), ne("Blur", M);
    }
    return ge(() => {
      if (u.value && Rn(s.value, c.value), !u.value) {
        let M = hn(d.value);
        o.value = M.html, l.value = M.error;
      }
    }), (M, x) => (b(), R("div", {
      class: $e(f.value),
      "data-block-id": k.value,
      "data-math-block": v.value,
      "data-node-type": T.value
    }, [
      u.value ? (b(), R(xe, { key: 0 }, [
        c.value ? (b(), R("div", $i, [...x[3] || (x[3] = [
          _("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        _("div", Di, [
          O.value ? (b(), R("div", {
            key: 0,
            class: "autodown-math-preview",
            innerHTML: m.value
          }, null, 8, qi)) : $("", !0),
          V.value ? (b(), R("div", Hi, [
            _("span", null, U(w.value), 1)
          ])) : $("", !0),
          kt(_("textarea", {
            class: "math-editor-textarea",
            disabled: c.value,
            ref_key: "area",
            ref: s,
            rows: fe.value,
            spellcheck: "false",
            "onUpdate:modelValue": x[0] || (x[0] = (B) => i.value = B),
            onBlur: x[1] || (x[1] = (B) => L(B)),
            onInput: x[2] || (x[2] = (B) => C(B))
          }, null, 40, Ni), [
            [Ot, i.value]
          ])
        ])
      ], 64)) : $("", !0),
      u.value ? $("", !0) : (b(), R(xe, { key: 1 }, [
        ie.value ? (b(), R("div", {
          key: 0,
          class: "autodown-math-preview",
          innerHTML: o.value
        }, null, 8, Pi)) : $("", !0),
        j.value ? (b(), R("div", Fi, [
          _("span", null, U(l.value), 1)
        ])) : $("", !0),
        _("pre", Ki, [
          x[4] || (x[4] = ut("          ", -1)),
          (b(), J(pe(Q.value))),
          x[5] || (x[5] = ut(`
        `, -1))
        ])
      ], 64))
    ], 10, Ai));
  }
}), zn = /* @__PURE__ */ bt(Ui, [["__scopeId", "data-v-bf30707d"]]);
async function mn(n) {
  const t = await Mn(n);
  return t.error === "" && Bn("Mermaid", n, { kind: "svg", body: t.svg, error: "" }), t;
}
let rt = null, vn = 0;
const Wi = 300;
function pn(n, t) {
  const e = ++vn;
  if (rt != null && clearTimeout(rt), n.trim() === "") {
    t({ svg: "", error: "", loading: !1 });
    return;
  }
  rt = setTimeout(() => {
    rt = null, t({ svg: "", error: "", loading: !0 }), Mn(n).then((o) => {
      e === vn && t({ svg: o.svg, error: o.error, loading: !1 });
    });
  }, Wi);
}
const Vi = ["data-block-id", "data-mermaid-block", "data-node-type"], Qi = {
  key: 0,
  class: "autodown-stream-banner"
}, zi = { class: "mermaid-editor-stack" }, Gi = ["innerHTML"], ji = {
  key: 1,
  class: "autodown-mermaid-error",
  title: "Mermaid render error"
}, Yi = {
  key: 2,
  class: "mermaid-editor-loading"
}, Xi = ["disabled", "rows"], Ji = ["innerHTML"], Zi = {
  key: 1,
  class: "autodown-mermaid-error",
  title: "Mermaid render error"
}, er = { class: "mermaid-source" }, tr = /* @__PURE__ */ Z({
  __name: "MermaidBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(n, { emit: t }) {
    const e = n, o = y(""), l = y(""), i = y(ft(e.node)), r = y(""), s = y(""), u = y(!1), d = y(Sn(e.ctx)), c = y(null), f = g(() => e.mode === "edit"), v = g(() => ft(e.node)), k = g(() => qt(e.ctx)), T = g(() => f.value ? k.value ? "autodown-mermaid-editor is-readonly" : "autodown-mermaid-editor" : "autodown-mermaid-block"), m = g(() => Tn(e.mode)), w = g(() => ht(e.mode, Ht(e.ctx))), O = g(() => ht(e.mode, "Mermaid")), V = g(() => u.value === !1 && !s.value && !!r.value), fe = g(() => u.value === !1 && !!s.value), ie = g(() => Qn(i.value)), j = g(() => !!o.value), Q = g(() => !o.value && !!l.value), ne = g(() => "code"), C = t;
    De(v, () => {
      f.value || (v.value.trim() == "" && (o.value = "", l.value = ""), v.value.trim() != "" && mn(v.value).then((B) => {
        o.value = B.svg, l.value = B.error;
      }));
    });
    function L(x) {
      i.value = x.target.value, u.value = !0, pn(i.value, (B) => {
        r.value = B.svg, s.value = B.error, u.value = B.loading;
      }), C("AreaInput", x);
    }
    function M(x) {
      k.value || d.value.commit(x.target.value), C("Blur", x);
    }
    return ge(() => {
      f.value && (Rn(c.value, k.value), u.value = !0, pn(v.value, (x) => {
        r.value = x.svg, s.value = x.error, u.value = x.loading;
      })), f.value || (v.value.trim() == "" && (o.value = "", l.value = ""), v.value.trim() != "" && mn(v.value).then((B) => {
        o.value = B.svg, l.value = B.error;
      }));
    }), (x, B) => (b(), R("div", {
      class: $e(T.value),
      "data-block-id": w.value,
      "data-mermaid-block": m.value,
      "data-node-type": O.value
    }, [
      f.value ? (b(), R(xe, { key: 0 }, [
        k.value ? (b(), R("div", Qi, [...B[3] || (B[3] = [
          _("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        _("div", zi, [
          V.value ? (b(), R("div", {
            key: 0,
            class: "autodown-mermaid-preview",
            innerHTML: r.value
          }, null, 8, Gi)) : $("", !0),
          fe.value ? (b(), R("div", ji, [
            _("span", null, U(s.value), 1)
          ])) : $("", !0),
          u.value ? (b(), R("div", Yi, [...B[4] || (B[4] = [
            _("span", null, "渲染中…", -1)
          ])])) : $("", !0),
          kt(_("textarea", {
            class: "mermaid-editor-textarea",
            disabled: k.value,
            ref_key: "area",
            ref: c,
            rows: ie.value,
            spellcheck: "false",
            "onUpdate:modelValue": B[0] || (B[0] = (ee) => i.value = ee),
            onBlur: B[1] || (B[1] = (ee) => M(ee)),
            onInput: B[2] || (B[2] = (ee) => L(ee))
          }, null, 40, Xi), [
            [Ot, i.value]
          ])
        ])
      ], 64)) : $("", !0),
      f.value ? $("", !0) : (b(), R(xe, { key: 1 }, [
        j.value ? (b(), R("div", {
          key: 0,
          class: "autodown-mermaid-preview",
          innerHTML: o.value
        }, null, 8, Ji)) : $("", !0),
        Q.value ? (b(), R("div", Zi, [
          _("span", null, U(l.value), 1)
        ])) : $("", !0),
        _("pre", er, [
          B[5] || (B[5] = ut("          ", -1)),
          (b(), J(pe(ne.value))),
          B[6] || (B[6] = ut(`
        `, -1))
        ])
      ], 64))
    ], 10, Vi));
  }
}), Gn = /* @__PURE__ */ bt(tr, [["__scopeId", "data-v-08feae74"]]), nr = ["data-open"], or = { class: "autodown-details-summary" }, lr = { class: "autodown-details-content" }, ir = {
  key: 0,
  class: "markdown-renderer"
}, jn = /* @__PURE__ */ Z({
  __name: "DetailsBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    children: {},
    version: {}
  },
  emits: ["ToggleOpen"],
  setup(n, { emit: t }) {
    const e = n, o = g(() => e.mode === "edit"), l = g(() => qt(e.ctx));
    g(() => Ht(e.ctx));
    const i = g(() => go(e.ctx)), r = g(() => ko(e.node, e.ctx)), s = g(() => bo(e.node, "open")), u = g(() => s.value ? "▼" : "▶"), d = g(() => wo(e.node, "summary")), c = g(() => d.value ? d.value : "Details"), f = t;
    function v() {
      _o(i.value, r.value, s.value), f("ToggleOpen");
    }
    return (k, T) => (b(), R("div", {
      class: "autodown-details",
      "data-open": s.value
    }, [
      _("div", or, [
        _("span", {
          class: "autodown-details-marker",
          "aria-hidden": "true",
          title: "点击展开详细内容",
          onClick: Pe(v, ["stop"])
        }, [
          _("span", null, U(u.value), 1)
        ]),
        o.value ? (b(), J(de(yo), {
          attr_key: "summary",
          blockId: r.value,
          controller: i.value,
          host_class: "autodown-details-summary-text",
          placeholder: "Details",
          readonly: l.value,
          value: d.value,
          version: n.version,
          key: "AttrHost-1"
        }, null, 8, ["blockId", "controller", "readonly", "value", "version"])) : $("", !0),
        o.value ? $("", !0) : (b(), R("span", {
          key: 1,
          class: "autodown-details-summary-text",
          onClick: Pe(v, ["stop"])
        }, [
          _("span", null, U(c.value), 1)
        ]))
      ]),
      kt(_("div", lr, [
        o.value ? (b(), R("div", ir, [
          (b(), J(de(dt), {
            children_slot: n.children,
            key: "BlockChildren-2"
          }, null, 8, ["children_slot"]))
        ])) : $("", !0),
        o.value ? $("", !0) : (b(), J(de(dt), {
          children_slot: n.children,
          key: "BlockChildren-3"
        }, null, 8, ["children_slot"]))
      ], 512), [
        [to, s.value]
      ])
    ], 8, nr));
  }
});
function rr(n) {
  return Ae((n == null ? void 0 : n.attrs) ?? [], "query", "");
}
function gn() {
  return Wt().runQuery ?? null;
}
function kn(n) {
  return (n && n.results || []).map((e) => ({
    ...e,
    source: e.title || e.page_path,
    priority_label: e.priority ? `[#${e.priority}]` : ""
  }));
}
function bn(n) {
  return (n == null ? void 0 : n.message) || String(n);
}
const sr = {
  class: "autodown-query-block",
  "data-query-block": ""
}, ar = { class: "query-header" }, ur = {
  key: 0,
  class: "query-state"
}, cr = {
  key: 1,
  class: "query-state query-error"
}, dr = { class: "result-marker" }, fr = {
  key: 0,
  class: "result-priority"
}, hr = { class: "result-content" }, mr = { class: "result-source" }, vr = {
  key: 3,
  class: "query-state"
}, pr = /* @__PURE__ */ Z({
  __name: "QueryBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(n, { emit: t }) {
    const e = n, o = y([]), l = y(!1), i = y(""), r = g(() => rr(e.node)), s = g(() => "code"), u = g(() => "ul"), d = g(() => "li"), c = g(() => l.value || !e.final), f = g(() => e.final && !l.value && !!i.value), v = g(() => e.final && !l.value && !i.value && o.value.length > 0), k = g(() => e.final && !l.value && !i.value && o.value.length === 0);
    return De(r, async () => {
      if (e.final) {
        let T = gn();
        if ((T == null || r.value == "") && (i.value = "No query runner configured"), T != null && r.value != "") {
          l.value = !0, i.value = "";
          try {
            let m = await T(r.value);
            o.value = kn(m);
          } catch (m) {
            i.value = bn(m), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), ge(async () => {
      if (e.final) {
        let T = gn();
        if ((T == null || r.value == "") && (i.value = "No query runner configured"), T != null && r.value != "") {
          l.value = !0, i.value = "";
          try {
            let m = await T(r.value);
            o.value = kn(m);
          } catch (m) {
            i.value = bn(m), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), (T, m) => (b(), R("div", sr, [
      _("div", ar, [
        m[0] || (m[0] = _("span", { class: "query-label" }, [
          _("span", null, "Query")
        ], -1)),
        (b(), J(pe(s.value), { class: "query-code" }, {
          default: st(() => [
            _("span", null, U(r.value), 1)
          ]),
          _: 1
        }))
      ]),
      c.value ? (b(), R("div", ur, [...m[1] || (m[1] = [
        _("span", null, "Loading query…", -1)
      ])])) : $("", !0),
      f.value ? (b(), R("div", cr, [
        _("span", null, U(i.value), 1)
      ])) : $("", !0),
      v.value ? (b(), J(pe(u.value), {
        key: 2,
        class: "query-results"
      }, {
        default: st(() => [
          (b(!0), R(xe, null, Je(o.value, (w, O) => (b(), J(pe(d.value), {
            class: "query-result",
            key: O
          }, {
            default: st(() => [
              _("span", dr, [
                _("span", null, U(w.marker), 1)
              ]),
              w.priority ? (b(), R("span", fr, [
                _("span", null, U(w.priority_label), 1)
              ])) : $("", !0),
              _("span", hr, [
                _("span", null, U(w.content), 1)
              ]),
              _("span", mr, [
                _("span", null, U(w.source), 1)
              ])
            ]),
            _: 2
          }, 1024))), 128))
        ]),
        _: 1
      })) : $("", !0),
      k.value ? (b(), R("div", vr, [...m[2] || (m[2] = [
        _("span", null, "No results", -1)
      ])])) : $("", !0)
    ]));
  }
}), gr = /* @__PURE__ */ bt(pr, [["__scopeId", "data-v-45f3e7c3"]]);
function Yn(n) {
  if (n.startsWith("^"))
    return { title: "", blockId: n.length > 1 ? n.slice(1) : null };
  const t = n.indexOf("#^");
  return t >= 0 ? { title: n.slice(0, t), blockId: n.slice(t + 2) || null } : { title: n, blockId: null };
}
function Xn(n) {
  return Ae((n == null ? void 0 : n.attrs) ?? [], "src", "");
}
function kr(n) {
  return Yn(Xn(n)).title;
}
function br(n) {
  return Yn(Xn(n)).blockId;
}
function wn() {
  return Wt().loadBlock ?? null;
}
function yn(n) {
  return (n == null ? void 0 : n.message) || String(n);
}
const wr = ["data-title"], yr = {
  key: 0,
  class: "embed-state"
}, _r = {
  key: 1,
  class: "embed-state embed-error"
}, Ir = {
  key: 2,
  class: "embed-header"
}, Cr = { class: "embed-title" }, xr = {
  key: 3,
  class: "embed-content"
}, Br = /* @__PURE__ */ Z({
  __name: "EmbedBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(n, { emit: t }) {
    const e = n, o = y(null), l = y(!1), i = y(""), r = g(() => kr(e.node)), s = g(() => br(e.node)), u = g(() => s.value != null ? r.value ? r.value + "#" + s.value : s.value : r.value), d = g(() => "Loading " + u.value + "…" || "Loading…"), c = g(() => o.value && o.value.content || ""), f = g(() => l.value || !e.final), v = g(() => e.final && !l.value && !!i.value), k = g(() => e.final && !l.value && !i.value && o.value), T = g(() => e.final && !l.value && !i.value);
    return De(s, async () => {
      if (e.final && s.value != null) {
        let m = wn();
        if (m == null && (i.value = "No block loader configured"), m != null) {
          l.value = !0, i.value = "";
          try {
            let w = await m(s.value);
            o.value = w, w || (i.value = "Block not found");
          } catch (w) {
            i.value = yn(w), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), ge(async () => {
      if (e.final && s.value != null) {
        let m = wn();
        if (m == null && (i.value = "No block loader configured"), m != null) {
          l.value = !0, i.value = "";
          try {
            let w = await m(s.value);
            o.value = w, w || (i.value = "Block not found");
          } catch (w) {
            i.value = yn(w), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), (m, w) => (b(), R("div", {
      class: "autodown-block-embed",
      "data-title": r.value
    }, [
      f.value ? (b(), R("div", yr, [
        _("span", null, U(d.value), 1)
      ])) : $("", !0),
      v.value ? (b(), R("div", _r, [
        _("span", null, U(i.value), 1)
      ])) : $("", !0),
      T.value ? (b(), R("div", Ir, [
        _("span", Cr, [
          _("span", null, U(u.value), 1)
        ])
      ])) : $("", !0),
      k.value ? (b(), R("div", xr, [
        _("span", null, U(c.value), 1)
      ])) : $("", !0)
    ], 8, wr));
  }
}), Sr = /* @__PURE__ */ bt(Br, [["__scopeId", "data-v-5badd3b4"]]);
Nt("Fence", Ro);
Nt("MathBlock", zn);
Nt("Mermaid", Gn);
function Tr(n, t) {
  var l;
  const e = (i) => ({
    id: i.id,
    text: Y(i),
    cls: Rr(Ae(i.attrs, "align", "left"))
  }), o = n.children;
  return te(Eo, {
    mode: "edit",
    controller: new Oi(t.engine, t.blockId),
    blockId: t.blockId,
    readonly: t.readonly,
    final: !0,
    header_cells: (((l = o[0]) == null ? void 0 : l.children) ?? []).map(e),
    body_rows: o.slice(1).map((i) => ({ id: i.id, cells: i.children.map(e) })),
    columns: [],
    rows: []
  });
}
function Rr(n) {
  return n === "center" ? "text-center" : n === "right" ? "text-right" : "text-left";
}
Io("Table", { edit: Tr });
function Mr(n, t) {
  const e = [];
  return (t == null ? void 0 : t.type) === "details" && (e.push({ key: "summary", value: se.Str(String(t.text ?? "")) }), (t == null ? void 0 : t.loading) === !0 && e.push({ key: "open", value: se.Bool(!0) })), {
    id: "nv",
    kind: n,
    attrs: e,
    children: [],
    inlines: [],
    source: { start: 0, end: 0 }
  };
}
Ze(
  "Details",
  // plan 035 T6: the family widget replaces the retired DetailsNodeView —
  // same view face, plus the marker verb riding the live host window's
  // engine (the preview-side toggle writes `open` back through the model,
  // the host-protocol contract) and the body through the BlockChildren
  // closure instead of the node-view injection key.
  (n) => {
    const t = Ei(), e = Co(n.node) ?? Mr(h.Details, n.node);
    return te(jn, {
      mode: "view",
      node: e,
      final: n.final ?? !0,
      ctx: t ? { engine: t.engine, blockId: e.id, readonly: !0 } : null,
      children: xo(Mo(), () => Ln(n.node.children ?? [], !0)),
      version: 0
    });
  }
);
Ze("MathBlock", wt(zn));
Ze("Mermaid", wt(Gn));
Ze("Query", wt(gr));
Ze("Embed", wt(Sr));
const Dr = /* @__PURE__ */ Z({
  __name: "EngineEditor",
  props: {
    content: {},
    modelValue: {},
    placeholder: {},
    extraSlashItems: {},
    streaming: { type: Boolean },
    runQuery: { type: Function },
    loadBlock: { type: Function }
  },
  emits: ["update", "update:modelValue", "save", "open-wiki-link"],
  setup(n, { expose: t, emit: e }) {
    const o = n, l = e, i = (a, p) => l("open-wiki-link", a, p);
    nn(i), Ge(() => {
      Lo() === i && nn(null);
    });
    let r = !1;
    De(
      () => [o.runQuery, o.loadBlock],
      () => {
        o.runQuery != null || o.loadBlock != null ? (Bt({ runQuery: o.runQuery, loadBlock: o.loadBlock }), r = !0) : r && (Bt({}), r = !1);
      },
      { immediate: !0 }
    ), Ge(() => {
      if (!r) return;
      const a = Wt();
      a.runQuery === o.runQuery && a.loadBlock === o.loadBlock && Bt({});
    });
    const s = y(null), u = y(null);
    function d(a) {
      return Dt(a ?? "", !0);
    }
    const c = new al(d(o.modelValue ?? o.content ?? "")), f = Nl(c), v = Ri({ extraSlashItems: o.extraSlashItems });
    let k = ot(c.doc, !0);
    c.onChange(() => {
      w.value++, T();
    });
    function T() {
      const a = ot(c.doc, !0);
      a !== k && (k = a, l("update", a), l("update:modelValue", a));
    }
    De(
      () => o.modelValue ?? o.content,
      (a) => {
        a != null && a !== ot(c.doc, !0) && c.replaceDoc(d(a));
      }
    );
    function m() {
      var H;
      const a = typeof window > "u" ? null : window.getSelection();
      if (!a || a.rangeCount === 0) return;
      const p = a.getRangeAt(0);
      if (p.collapsed) return;
      const I = p.startContainer, D = I.nodeType === 3 ? I.parentElement : I, N = D == null ? void 0 : D.closest(".autodown-block-host");
      if (!N || !((H = s.value) != null && H.contains(N))) return;
      const S = N.dataset.blockId;
      if (!S) return;
      const A = Ol(N, S);
      !A || A.lo === A.hi || c.selection.anchor.blockId === S && c.selection.anchor.offset === A.lo && c.selection.head.offset === A.hi || c.select(new Fe(new ae(S, A.lo), new ae(S, A.hi)));
    }
    ge(() => document.addEventListener("selectionchange", m)), Ge(() => document.removeEventListener("selectionchange", m));
    const w = y(0), O = y(0), V = Z({
      name: "AssemblyView",
      props: { render: { type: Function, required: !0 } },
      setup(a) {
        return () => a.render();
      }
    }), fe = g(() => {
      w.value;
      const a = c.selection.anchor.blockId, p = { path: Ll(c.doc, a), focusedId: a, counter: { n: 0 } };
      return c.doc.children.map((I) => M(I, p, !0));
    });
    function ie(a) {
      Mi({ engine: c, adapter: f });
      try {
        return Ln(Bo([a]), !0)[0] ?? te("div", { class: "unknown-node" }, "");
      } finally {
        Li();
      }
    }
    function j(a, p, I, D, N = !0) {
      return te(
        "div",
        {
          class: "node-slot",
          "data-node-index": String(D.n++),
          "data-node-type": h[a.kind],
          "data-block-id": a.id,
          // the innermost addressable slot wins — an expanded container's outer
          // chrome must not re-handle the bubbled click (it would resolve the
          // whole container back to its first leaf)
          onClick: (S) => {
            S.stopPropagation(), me(a.id);
          }
        },
        [
          te("div", { class: "node-content" }, [p]),
          ...I && N ? [te("div", { class: "autodown-block-boundary", "data-boundary-for": a.id })] : []
        ]
      );
    }
    function Q(a, p) {
      const I = () => a.children.map((N) => C(N, p)), D = {
        mode: "edit",
        node: a,
        ctx: { engine: c, blockId: a.id, readonly: o.streaming === !0 },
        final: !0,
        version: w.value
      };
      return a.kind === h.Callout ? te(So, { ...D, children: I }) : a.kind === h.Details ? te(jn, { ...D, children: I }) : a.kind === h.Blockquote ? te(li, { ...D, children: I }) : te(To, { ...D, items: ne(a, p) });
    }
    function ne(a, p) {
      return a.children.map((I) => {
        const D = ct(I.attrs, "checked") != null;
        return {
          id: I.id,
          task: D,
          checked: ao(I.attrs, "checked", !1),
          cls: "list-item" + (D ? " task-item" : ""),
          children_slot: () => I.children.map((N) => C(N, p))
        };
      });
    }
    function C(a, p) {
      return a.id === p.focusedId || p.path.has(a.id) ? L(M(a, p, !1)) : j(a, ie(a), !1, p.counter);
    }
    function L(a) {
      return te(a.view, a.props);
    }
    function M(a, p, I) {
      if (a.id === p.focusedId) {
        const D = xn(h[a.kind]);
        if (D)
          return {
            id: a.id,
            view: V,
            props: {
              render: () => j(a, D(a, { engine: c, blockId: a.id, readonly: o.streaming === !0 }), I, p.counter, !1),
              key: `edit:${a.id}:${O.value}`
            }
          };
        if (Pt(a)) {
          const N = ee(a.id), S = a.kind === h.Heading ? so(a.attrs, "level", 1) : void 0;
          return {
            id: a.id,
            view: V,
            props: {
              render: () => j(
                a,
                te(ti, {
                  controller: N,
                  // flat chrome data (plan 034 D4): the widget derives tag/cls from
                  // blockKind/level itself (the host-face computation is absorbed);
                  // initial_html is the mount-once rich snapshot, evaluated here —
                  // the engine is not Vue-reactive, the snapshot never invalidates.
                  blockId: N.id,
                  blockKind: h[a.kind],
                  level: S ?? 0,
                  initial_html: Dn(N.inlines),
                  // The face lives in the key: a kind/level flip mid-typing (input
                  // rules) must REMOUNT the host. <component :is> would swap the
                  // DOM element under the caret without re-running onMounted —
                  // focus lands nowhere and every post-flip keystroke is lost.
                  // The remount re-focuses at end (plan 029; rules match only a
                  // whole-block marker, so the caret IS at end on every flip).
                  key: `host:${a.id}:${h[a.kind]}:${S ?? ""}:${O.value}`
                }),
                I,
                p.counter,
                !1
              )
            }
          };
        }
      }
      return p.path.has(a.id) && x(a) ? {
        id: a.id,
        view: V,
        props: { render: () => j(a, Q(a, p), I, p.counter) }
      } : {
        id: a.id,
        view: V,
        props: { render: () => j(a, ie(a), I, p.counter) }
      };
    }
    function x(a) {
      return a.children.length > 0 && (a.kind === h.ListBlock || a.kind === h.Blockquote || a.kind === h.Callout || a.kind === h.Details);
    }
    const B = /* @__PURE__ */ new Map();
    function ee(a) {
      let p = B.get(a);
      return p || (p = new Tl(c, a), B.set(a, p)), p;
    }
    function be() {
      const a = [], p = u.value;
      return p && Array.from(p.querySelectorAll("[data-block-id]")).forEach((D, N) => {
        a.push({
          id: D.dataset.blockId ?? "",
          index: N,
          pos: N,
          el: D,
          top: D.offsetTop,
          height: D.offsetHeight
        });
      }), a;
    }
    function me(a) {
      const p = q(c.doc, a);
      if (!p) return;
      const I = Lt(p) ?? p, D = new ae(I.id, 0);
      c.select(new Fe(D, D)), w.value++;
    }
    function we(a) {
      const p = Rl(a);
      if (p) {
        if (Array.from(B.values()).some((I) => I.composition.composing)) return;
        a.preventDefault(), Ml(c, B.values(), p) && O.value++;
        return;
      }
      if (a.ctrlKey && a.key === "End") {
        a.preventDefault();
        const I = Hn(c.doc);
        I && me(I.id);
      }
    }
    function ye() {
      const a = Lt(c.doc);
      if (a) {
        const p = new ae(a.id, 0);
        c.select(new Fe(p, p));
      }
    }
    ye();
    function We() {
      l("save", ot(c.doc, !0));
    }
    return t({ getBlockMap: be, handleSave: We }), (a, p) => (b(), R("div", {
      ref_key: "root",
      ref: s,
      class: "autodown-editor"
    }, [
      _("div", {
        ref_key: "wrapper",
        ref: u,
        class: "autodown-editor-content-wrapper"
      }, [
        _("div", {
          class: "autodown-editor-content",
          "data-engine-editor": "",
          tabindex: "-1",
          onKeydown: we
        }, [
          yt(de(Si), {
            editor: de(f),
            items: de(v)
          }, null, 8, ["editor", "items"]),
          yt(fi, { editor: de(f) }, null, 8, ["editor"]),
          yt(wi, { editor: de(f) }, null, 8, ["editor"]),
          (b(!0), R(xe, null, Je(fe.value, (I) => (b(), J(pe(I.view), no({
            key: I.id
          }, { ref_for: !0 }, I.props), null, 16))), 128))
        ], 32)
      ], 512),
      _("div", { class: "autodown-editor-actions" }, [
        _("button", {
          type: "button",
          class: "autodown-editor-save",
          onClick: We
        }, "Save")
      ])
    ], 512));
  }
}), qr = "block-";
function Hr(n) {
  const e = Array.from((n ?? document).querySelectorAll("[data-block-id]")), o = /* @__PURE__ */ new Set(), l = [];
  for (const i of e) {
    const r = i.dataset.blockId ?? "";
    r !== "" && o.has(r) || (r !== "" && o.add(r), l.push({ id: r, index: l.length, pos: l.length, el: i, top: i.offsetTop, height: i.offsetHeight }));
  }
  return l;
}
export {
  qr as B,
  al as E,
  Dr as _,
  Tl as a,
  Wt as b,
  Nl as c,
  Hr as g,
  Bt as s
};
