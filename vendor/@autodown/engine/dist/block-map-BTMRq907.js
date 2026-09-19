import { ref as b, getCurrentInstance as Wn, onBeforeUnmount as Me, defineComponent as Z, computed as v, onMounted as fe, openBlock as p, createBlock as X, resolveDynamicComponent as ge, withModifiers as Oe, normalizeClass as Te, h as ie, unref as ce, withCtx as Ye, createElementBlock as R, Fragment as _e, renderList as Pe, createCommentVNode as $, onUnmounted as yt, normalizeStyle as gn, createElementVNode as w, withDirectives as st, withKeys as Ve, vModelText as Bt, toDisplayString as G, nextTick as re, watch as Ae, inject as Vn, provide as Qn, createTextVNode as Xe, vShow as zn, createVNode as vt, mergeProps as Gn } from "vue";
import { a0 as Ze, O as jn, J as Yn, ab as Ft, T as kn, M as q, B as x, f as Y, a1 as et, u as Jn, a2 as Ut, i as Tt, Q as qe, V as Se, r as we, b as pe, $ as Je, a7 as bn, d as ut, ac as Qe, k as Xn, S as mt, K as ze, h as wn } from "./parser-BfX0E-c9.js";
import { H as Zn, I as eo, J as _t, K as Kt, L as to, M as no, N as yn, O as oo, a as Wt, c as lo, t as ao, b as io, P as ro, Q as Ct, R as _n, S as tt, T as Cn, U as xt, V as Sn, W as nt, X as Rt, Y as In, _ as ct, Z as Bn, $ as so, a0 as uo, a1 as co, a2 as fo, a3 as ho, a4 as ot, a5 as vo, y as xe, l as Fe, a6 as mo, a7 as po, w as dt, a8 as go, p as Vt, a9 as ko, B as bo, F as Ue, aa as ft, ab as wo, ac as yo, ad as pt, ae as _o, af as Co, ag as Qt, ah as zt, G as So, ai as gt, aj as Io, ak as Gt, al as Bo } from "./render-node-jJKN3LNR.js";
import { Link as Tn, Code as xn, Strikethrough as To, Underline as xo, Italic as Ro, Bold as Mo, Check as Eo, Text as Ao, Heading1 as Lo, Heading2 as Oo, Heading3 as Do, Heading4 as $o, Heading5 as qo, Heading6 as No, List as Ho, ListOrdered as Po, CheckSquare as Fo, Quote as Uo, Minus as Ko, Image as Wo, Table as Vo, AlertCircle as Qo, PanelTop as zo, Sigma as Go, Workflow as jo, Square as Yo, CircleDot as Jo, CheckCircle2 as Xo, Clock as Zo, Timer as el, ArrowUp as kt, Search as tl } from "lucide-vue-next";
class nl {
  constructor(n, e) {
    var o;
    this.undoStack = [], this.redoStack = [], this.listeners = [], this.tree = n, this.sel = e ?? Ze(((o = n.children[0]) == null ? void 0 : o.id) ?? "", 0);
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
  onChange(n) {
    this.listeners.push(n);
  }
  /** Apply one op through the 016 kernel, recording an undo entry.
   *  Adjacent InsertText typing coalesces into the previous entry. */
  apply(n, e = {}) {
    const o = this.undoStack[this.undoStack.length - 1], l = o != null && o.ops.length === 1 ? o.ops[0] : void 0;
    if (e.coalesce !== !1 && l != null && l._tag === "InsertText" && n._tag === "InsertText" && l.value.pos.blockId === n.value.pos.blockId && l.value.pos.offset + l.value.text.length === n.value.pos.offset && o) {
      const r = l.value, u = n.value;
      o.ops[0] = jn.InsertText(new Yn(r.pos, r.text + u.text));
    } else
      this.undoStack.push({ preTree: this.tree, preSel: this.sel, ops: [n] });
    this.redoStack = [];
    const i = Ft(this.tree, this.sel, n);
    return this.tree = i.tree, this.sel = i.selection, this.emit(!1), i;
  }
  /** Apply a composed op group as ONE undo step (input rules etc.). */
  applyGroup(n, e) {
    n.length === 0 && e == null || (this.undoStack.push({ preTree: this.tree, preSel: this.sel, ops: [...n], after: e }), this.redoStack = [], this.thread(n, e), this.emit(!1));
  }
  /** Apply a pure tree transform as ONE undo step (command layer:
   *  insertTemplate / table ops / moveBlock — Phase 3). */
  applyTree(n) {
    this.applyGroup([], n);
  }
  /** Set the selection without a document change (focus moves). */
  select(n) {
    this.sel = n, this.emit(!1);
  }
  thread(n, e) {
    let o = this.tree, l = this.sel;
    for (const a of n) {
      const i = Ft(o, l, a);
      o = i.tree, l = i.selection;
    }
    e && (o = e(o)), this.tree = o, this.sel = l;
  }
  undo() {
    const n = this.undoStack.pop();
    return n ? (this.redoStack.push(n), this.tree = n.preTree, this.sel = n.preSel, this.emit(!0), !0) : !1;
  }
  redo() {
    const n = this.redoStack.pop();
    if (!n) return !1;
    const e = this.tree, o = this.sel;
    return this.thread(n.ops, n.after), this.undoStack.push({ preTree: e, preSel: o, ops: n.ops, after: n.after }), this.emit(!0), !0;
  }
  /** Streaming append (plan 018 待澄清 1 — 追加分流裁定): AI/stream blocks
   *  land at the document tail without touching the focused block or the
   *  selection; not an undoable user edit. */
  appendBlocks(n) {
    n.length !== 0 && (this.tree = kn(this.tree, [...this.tree.children, ...n]), this.emit(!1));
  }
  /** External document replacement (file load, full paste). Not undoable —
   *  callers that need undo wrap it in their own op. */
  replaceDoc(n, e) {
    var o;
    this.tree = n, this.sel = e ?? Ze(((o = n.children[0]) == null ? void 0 : o.id) ?? "", 0), this.undoStack = [], this.redoStack = [], this.emit(!1);
  }
  emit(n) {
    for (const e of this.listeners) e({ tree: this.tree, selection: this.sel, history: n });
  }
}
function ol(t) {
  if (!t.ctrlKey && !t.metaKey) return null;
  const n = t.key.toLowerCase();
  return n === "z" ? t.shiftKey ? "redo" : "undo" : n === "y" ? "redo" : null;
}
function ll(t, n, e) {
  const o = e === "undo" ? t.undo() : t.redo();
  if (o) for (const l of n) l.syncFromModel();
  return o;
}
function Mt(t) {
  return t.nodeType === 3 ? { raw: t, isText: !0, text: t.textContent ?? "", children: [] } : { raw: t, isText: !1, text: "", children: Array.from(t.childNodes).map(Mt) };
}
function Rn(t) {
  const n = [];
  let e = 0;
  const o = (l) => {
    l.isText ? (n.push({ node: l, start: e, len: l.text.length }), e += l.text.length) : l.children.forEach(o);
  };
  return o(t), { leaves: n, total: e };
}
function Et(t, n) {
  if (t.raw === n) return t;
  for (const e of t.children) {
    const o = Et(e, n);
    if (o) return o;
  }
  return null;
}
function jt(t, n, e) {
  const o = Et(t, n);
  if (!o) return -1;
  const { leaves: l } = Rn(t);
  if (o.isText) {
    const c = l.find((f) => f.node.raw === n);
    return c ? c.start + Math.max(0, Math.min(e, c.len)) : -1;
  }
  const a = Math.max(0, e), i = l.find((c) => {
    let f = c.node, d;
    for (; f && f !== o; )
      d = f, f = Mn(t, f);
    return !f || !d ? !1 : o.children.indexOf(d) >= a;
  });
  if (i) return i.start;
  const r = l.filter((c) => al(o, c.node.raw));
  if (r.length === 0) return -1;
  const u = r[r.length - 1];
  return u.start + u.len;
}
function Mn(t, n) {
  for (const e of t.children) {
    if (e === n) return t;
    const o = Mn(e, n);
    if (o) return o;
  }
}
function al(t, n) {
  return !!Et(t, n);
}
function Yt(t, n) {
  const { leaves: e, total: o } = Rn(t);
  if (e.length === 0) return null;
  const l = Math.max(0, Math.min(n, o));
  for (const i of e)
    if (l <= i.start + i.len) return { raw: i.node.raw, inner: l - i.start };
  const a = e[e.length - 1];
  return { raw: a.node.raw, inner: a.len };
}
function il(t, n) {
  const e = typeof window > "u" ? null : window.getSelection();
  if (!e || e.rangeCount === 0) return null;
  const o = e.getRangeAt(0);
  if (o.collapsed || !t.contains(o.startContainer) || !t.contains(o.endContainer)) return null;
  const l = Mt(t), a = jt(l, o.startContainer, o.startOffset), i = jt(l, o.endContainer, o.endOffset);
  return a < 0 || i < 0 ? null : { blockId: n, lo: Math.min(a, i), hi: Math.max(a, i) };
}
function rl(t, n, e) {
  const o = Mt(t), a = t.ownerDocument.createRange(), i = Yt(o, Math.min(n, e)), r = Yt(o, Math.max(n, e));
  return !i || !r ? (a.selectNodeContents(t), a) : (a.setStart(i.raw, i.inner), a.setEnd(r.raw, r.inner), a);
}
let Be = null;
function At(t) {
  Be = t;
}
function En() {
  return Be;
}
const lt = {
  strong: ["strong", "b"],
  em: ["em", "i"],
  del: ["del", "s"],
  u: ["u"],
  code: ["code"]
}, Ne = {
  [q.Strong]: "strong",
  [q.Em]: "em",
  [q.Del]: "del",
  [q.Underline]: "u",
  [q.Code]: "code"
};
function Ge(t) {
  const n = typeof window > "u" ? null : window.getSelection();
  if (!n || n.rangeCount === 0) return null;
  const e = n.getRangeAt(0);
  return e.collapsed || !t.contains(e.startContainer) || !t.contains(e.endContainer) ? null : e;
}
function de(t, n, e) {
  let o = n;
  for (; o && o !== t; ) {
    if (o.nodeType === 1) {
      const l = o;
      if (e.includes(l.tagName.toLowerCase())) return l;
    }
    o = o.parentNode;
  }
  return null;
}
function Jt(t, n, e) {
  const o = document.createRange();
  o.selectNodeContents(t);
  try {
    o.setEnd(n, e);
  } catch {
    return 0;
  }
  return o.toString().replace(/\u00A0/g, " ").length;
}
function Xt(t, n, e) {
  const o = de(t, n.startContainer, e);
  return o != null && o === de(t, n.endContainer, e) && o.contains(n.commonAncestorContainer);
}
function An(t) {
  const n = t.parentNode;
  if (n) {
    for (; t.firstChild; ) n.insertBefore(t.firstChild, t);
    n.removeChild(t), n.normalize();
  }
}
function Zt(t, n, e) {
  try {
    t.surroundContents(n);
  } catch {
    const o = t.extractContents();
    n.appendChild(o), t.insertNode(n);
  }
}
const ne = {
  getSelection() {
    const t = Be;
    if (!t) return null;
    const n = Ge(t);
    return n ? {
      blockId: t.dataset.blockId ?? "",
      start: Jt(t, n.startContainer, n.startOffset),
      end: Jt(t, n.endContainer, n.endOffset)
    } : null;
  },
  isActive(t) {
    const n = Be;
    if (!n) return !1;
    const e = Ge(n);
    if (!e) return !1;
    if (t === q.Link) {
      const l = de(n, e.startContainer, ["a"]);
      return l != null && l === de(n, e.endContainer, ["a"]);
    }
    const o = Ne[t];
    return o == null ? !1 : Xt(n, e, lt[o]);
  },
  applyMark(t, n) {
    const e = Be;
    if (!e) return !1;
    const o = Ge(e);
    if (!o) return !1;
    if (t === q.Link) {
      if (n == null || n === "") return ne.removeMark(q.Link);
      const i = de(e, o.startContainer, ["a"]);
      if (i && i === de(e, o.endContainer, ["a"]))
        return i.setAttribute("href", n), i.setAttribute("contenteditable", "false"), i.setAttribute("data-autodown-link", ""), !0;
      const r = e.ownerDocument.createElement("a");
      return r.setAttribute("href", n), r.setAttribute("contenteditable", "false"), r.setAttribute("data-autodown-link", ""), Zt(o, r), !0;
    }
    const l = Ne[t];
    if (l == null) return !1;
    const a = e.ownerDocument.createElement(l);
    return Zt(o, a), !0;
  },
  removeMark(t) {
    const n = Be;
    if (!n) return !1;
    const e = Ge(n);
    if (!e) return !1;
    if (t === q.Link) {
      const a = de(n, e.startContainer, ["a"]);
      if (a && a === de(n, e.endContainer, ["a"])) {
        const i = a.parentNode;
        if (i) {
          for (; a.firstChild; ) i.insertBefore(a.firstChild, a);
          i.removeChild(a);
        }
        return !0;
      }
      return !1;
    }
    const o = Ne[t];
    if (o == null) return !1;
    const l = lt[o];
    return Xt(n, e, l) ? (An(de(n, e.startContainer, l)), !0) : !1;
  }
};
function Ln() {
  const t = Be, n = typeof window > "u" ? null : window.getSelection();
  if (!t || !n || n.rangeCount === 0) return [];
  const e = n.getRangeAt(0);
  if (!t.contains(e.startContainer)) return [];
  const o = [];
  for (const l of [q.Strong, q.Em, q.Underline, q.Del, q.Code]) {
    const a = Ne[l];
    a != null && de(t, e.startContainer, lt[a] ?? [a]) && o.push(l);
  }
  return o;
}
function sl(t) {
  const n = Be, e = typeof window > "u" ? null : window.getSelection();
  if (!n || !e || e.rangeCount === 0) return !1;
  const o = e.getRangeAt(0);
  if (!n.contains(o.startContainer)) return !1;
  const l = Ne[t];
  if (l == null) return !1;
  const a = de(n, o.startContainer, lt[l] ?? [l]);
  return a ? (An(a), !0) : !1;
}
function Ie(t, n) {
  return t === ne && t.getSelection() == null ? sl(n) : t.isActive(n) ? t.removeMark(n) : t.applyMark(n);
}
const ul = {
  setParagraph: x.Paragraph,
  setMathBlock: x.MathBlock,
  setMermaidBlock: x.Mermaid,
  setHorizontalRule: x.ThematicBreak,
  toggleBulletList: x.ListItem,
  toggleOrderedList: x.ListItem,
  toggleBlockquote: x.Blockquote
}, cl = {
  bold: q.Strong,
  strong: q.Strong,
  italic: q.Em,
  em: q.Em,
  strike: q.Del,
  strikethrough: q.Del,
  underline: q.Underline,
  code: q.Code,
  link: q.Link
}, en = {
  table: x.Table,
  codeBlock: x.Fence,
  fence: x.Fence,
  blockquote: x.Blockquote,
  bulletList: x.ListBlock,
  orderedList: x.ListBlock,
  listItem: x.ListItem,
  heading: x.Heading,
  details: x.Details,
  callout: x.Callout,
  mathBlock: x.MathBlock,
  mermaid: x.Mermaid,
  queryBlock: x.QueryBlock,
  blockEmbed: x.BlockEmbed
};
function St(t, n, e) {
  if (t.id === n)
    return e.add(t.kind), !0;
  for (const o of t.children)
    if (St(o, n, e))
      return e.add(t.kind), !0;
  return !1;
}
function tn(t) {
  const n = {};
  for (const e of t) {
    const o = e.value;
    n[e.key] = o != null && (o._tag === "Str" || o._tag === "Int" || o._tag === "Bool") ? o.value : null;
  }
  return n;
}
function dl(t, n) {
  return t.anchor.blockId === n.anchor.blockId && t.anchor.offset === n.anchor.offset && t.head.blockId === n.head.blockId && t.head.offset === n.head.offset;
}
function fl(t) {
  const n = b(0), e = /* @__PURE__ */ new Map();
  let o = t.selection;
  const l = (i) => {
    const r = e.get(i);
    if (r)
      for (const u of [...r]) u();
  };
  return t.onChange((i) => {
    n.value++, dl(i.selection, o) || (o = i.selection, l("selectionUpdate"));
  }), {
    storage: { "slash-command": { query: "", range: null, handled: !1 } },
    isEditable: !0,
    on: (i, r) => {
      let u = e.get(i);
      u || (u = /* @__PURE__ */ new Set(), e.set(i, u)), u.add(r);
    },
    off: (i, r) => {
      var u;
      (u = e.get(i)) == null || u.delete(r);
    },
    isActive: (i) => {
      n.value;
      const r = cl[i];
      if (r != null) {
        const f = Ln();
        return f.length > 0 ? f.includes(r) : Jn(Zn(t, t.selection), r);
      }
      const u = en[i];
      if (u == null) return !1;
      const c = /* @__PURE__ */ new Set();
      return St(t.doc, t.selection.anchor.blockId, c), c.has(u);
    },
    getAttributes: (i) => {
      n.value;
      const r = en[i];
      if (r == null) return {};
      const u = Y(t.doc, t.selection.anchor.blockId);
      if (u && u.kind === r) return tn(u.attrs);
      if (u) {
        const c = /* @__PURE__ */ new Set();
        if (St(t.doc, u.id, c) && c.has(r)) {
          let f = u;
          for (; f; ) {
            if (f.kind === r) return tn(f.attrs);
            f = et(t.doc, f.id) ?? null;
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
            from: t.selection.anchor.offset,
            to: t.selection.head.offset
          }
        };
      },
      nodeDOM(i) {
        if (typeof document > "u") return null;
        const r = document.querySelector(".autodown-editor-content");
        if (!r) return null;
        const u = t.selection.anchor.blockId;
        for (const c of r.querySelectorAll("[data-block-id]"))
          if (c.dataset.blockId === u) return c;
        return null;
      },
      /** Caret viewport coords (plan 028 P3T1, 021-F5): the focused rich
       *  host's char offset → blockRangeToDomRange → first client rect
       *  (whole-host rect fallback). ProseMirror coordsAtPos shape — the
       *  generated floating menus (SlashMenu two-stage positioning) consume
       *  it to open at the caret instead of the default corner. */
      coordsAtPos(i) {
        if (typeof document > "u") return null;
        const r = t.selection.anchor.blockId, u = En() ?? document.querySelector(`.autodown-block-host[data-block-id="${r}"]`);
        if (!u) return null;
        const c = rl(u, i, i), f = c.getClientRects(), d = f.length > 0 ? f[0] : c.getBoundingClientRect();
        return { top: d.top, left: d.left, right: d.right, bottom: d.bottom };
      }
    },
    chain: () => hl(t),
    __engine: t,
    __bump: () => {
      n.value++;
    }
  };
}
function Re(t) {
  const n = t.selection.anchor.blockId, e = Y(t.doc, n);
  if (!e) return null;
  if (e.kind === x.Table) return { tableId: n, rowId: null, rowIdx: null, colIdx: null };
  if (e.kind !== x.TableCell) return null;
  const o = et(t.doc, n);
  if (!o || o.kind !== x.TableRow) return null;
  const l = et(t.doc, o.id);
  return !l || l.kind !== x.Table ? null : { tableId: l.id, rowId: o.id, rowIdx: Ut(l, o.id), colIdx: Ut(o, n) };
}
function De(t, n) {
  var e;
  return ((e = Y(t.doc, n)) == null ? void 0 : e.children) ?? [];
}
function hl(t) {
  const n = [], e = {
    focus: () => e,
    run: () => (t.applyTree((l) => n.reduce((a, i) => i(a), l)), !Y(t.doc, t.selection.anchor.blockId) && t.doc.children[0] && t.select(Ze(t.doc.children[0].id, 0)), !0),
    setHeading: (l) => (n.push((a) => $e(a, t, x.Heading, [{ key: "level", value: Se.Int((l == null ? void 0 : l.level) ?? 1) }])), e),
    insertContent: (l) => {
      const a = String(l ?? "");
      return n.push((i) => {
        if (!a.includes(`
`)) return bt(i, t, a);
        const u = bn(a, !0).children;
        if (u.length === 0) return i;
        const c = Ee(t), f = Y(i, c);
        if (!f) return i;
        const d = i.children, B = d.findIndex((h) => h.id === c), D = pe(f) === "" ? [] : [f], O = [...d.slice(0, B), ...D, ...u, ...d.slice(B + 1)];
        return kn(i, O);
      }), e;
    },
    deleteRange: (l) => (n.push((a) => {
      const i = Ee(t), r = Y(a, i);
      if (!r) return a;
      const u = pe(r), c = Math.max(0, Math.min(l.from, u.length)), f = Math.max(c, Math.min(l.to, u.length));
      return we(a, i, [Je(i, r.kind, u.slice(0, c) + u.slice(f))]);
    }), e),
    insertTable: (l) => (n.push((a) => bt(a, t, `| a | b |
| --- | --- |
|  |  |
|  |  |`)), e),
    setImage: (l) => {
      const a = String((l == null ? void 0 : l.src) ?? ""), i = String((l == null ? void 0 : l.alt) ?? "");
      return n.push((r) => bt(r, t, `![${i}](${a})`)), e;
    },
    // table verbs (plan 026 P0T3): forward to the commands.ts table transforms,
    // resolved against the focused cell (table-level focus takes the
    // table-ends defaults); tree-level so a chain stays ONE undo.
    addRowAfter: () => {
      var r;
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = l.rowId ?? ((r = a[a.length - 1]) == null ? void 0 : r.id) ?? null;
      return n.push((u) => Kt(u, l.tableId, i)), e;
    },
    addRowBefore: () => {
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = l.rowIdx != null && l.rowIdx > 0 ? a[l.rowIdx - 1].id : null;
      return n.push((r) => Kt(r, l.tableId, i)), e;
    },
    deleteRow: () => {
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId);
      if (a.length <= 1) return e;
      const i = l.rowId ?? a[a.length - 1].id;
      return n.push((r) => we(r, i, [])), e;
    },
    addColumnBefore: () => {
      const l = Re(t);
      return l && n.push((a) => _t(a, l.tableId, l.colIdx ?? 0)), e;
    },
    addColumnAfter: () => {
      var i;
      const l = Re(t);
      if (!l) return e;
      const a = ((i = De(t, l.tableId)[0]) == null ? void 0 : i.children.length) ?? 0;
      return n.push((r) => _t(r, l.tableId, l.colIdx != null ? l.colIdx + 1 : a)), e;
    },
    deleteColumn: () => {
      var r;
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = Math.max(0, (((r = a[0]) == null ? void 0 : r.children.length) ?? 1) - 1);
      return n.push((u) => eo(u, l.tableId, l.colIdx ?? i)), e;
    },
    deleteTable: () => {
      const l = Re(t);
      return l && n.push((a) => we(a, l.tableId, [])), e;
    },
    // code language channel (plan 026 P0T3): setBlockAttrs on the focused
    // Fence (023's IAL ruling); converts the kind when not a Fence yet.
    setCodeBlockLanguage: (l) => (n.push((a) => $e(a, t, x.Fence, [{ key: "language", value: Se.Str(String(l ?? "")) }])), e),
    setCodeBlock: (l) => (l == null ? void 0 : l.language) != null ? e.setCodeBlockLanguage(l.language) : (n.push((a) => $e(a, t, x.Fence)), e),
    // slash manifest's Details template carries { summary } (plan 026 P2T3):
    // kind conversion + summary attr so the mounted node-view shows it.
    // Converting an inline leaf moves its text into a child paragraph — a
    // Details renders children, inlines would serialize away (data loss).
    setDetails: (l) => (n.push((a) => {
      const i = Ee(t), r = Y(a, i);
      if (!r) return a;
      const u = r.children.length > 0 ? r.children : pe(r).length > 0 ? [Je(`${i}-p`, x.Paragraph, pe(r))] : [];
      let c = { ...r, kind: x.Details, children: u };
      return (l == null ? void 0 : l.summary) != null && (c = { ...c, attrs: qe(c.attrs, "summary", Se.Str(String(l.summary))) }), we(a, i, [c]);
    }), e),
    // slash Callout template carries { type, title } (plan 030 T7): same
    // conversion shape as setDetails — before this the kind-only KIND_COMMANDS
    // path silently dropped both attrs (the lost-title roundtrip break).
    setCallout: (l) => (n.push((a) => {
      const i = Ee(t), r = Y(a, i);
      if (!r) return a;
      const u = r.children.length > 0 ? r.children : pe(r).length > 0 ? [Je(`${i}-p`, x.Paragraph, pe(r))] : [];
      let c = { ...r, kind: x.Callout, children: u };
      return (l == null ? void 0 : l.type) != null && (c = { ...c, attrs: qe(c.attrs, "type", Se.Str(String(l.type))) }), (l == null ? void 0 : l.title) != null && (c = { ...c, attrs: qe(c.attrs, "title", Se.Str(String(l.title))) }), we(a, i, [c]);
    }), e),
    // task list (plan 030 T7): a real verb distinct from toggleBulletList —
    // the focused ListItem (a caret usually sits on its child paragraph, so
    // resolve the ListItem ancestor first — the list-commands 选中定位
    // discipline) gains/loses the `checked` attr (task ⇄ plain bullet);
    // outside a list it converts like the bullet verb.
    toggleTaskList: () => (n.push((l) => {
      const a = Ee(t);
      let i = Y(l, a);
      for (; i != null && i.kind !== x.ListItem; )
        i = et(l, i.id);
      if (i == null) return $e(l, t, x.ListItem);
      const u = Tt(i.attrs, "checked") != null ? i.attrs.filter((c) => c.key !== "checked") : qe(i.attrs, "checked", Se.Bool(!1));
      return we(l, i.id, [{ ...i, attrs: u }]);
    }), e),
    // inline mark toggles (plan 024 P3T1; adapter-routed plan 036 T3): wrap
    // the FOCUSED host's live DOM through the SelectionAdapter — the model
    // catches up on the blur writeback. No focused host → no-op.
    toggleBold: () => (Ie(ne, q.Strong), e),
    toggleItalic: () => (Ie(ne, q.Em), e),
    toggleStrike: () => (Ie(ne, q.Del), e),
    toggleCode: () => (Ie(ne, q.Code), e),
    // underline (plan 028 P2T2): same DOM-wrap protocol as the others —
    // the model catches up on the blur writeback (u → Mark.Underline)
    toggleUnderline: () => (Ie(ne, q.Underline), e),
    setLink: (l) => {
      const a = String((l == null ? void 0 : l.href) ?? "");
      return a ? ne.applyMark(q.Link, a) : ne.removeMark(q.Link), e;
    },
    unsetLink: () => (ne.removeMark(q.Link), e)
  }, o = e;
  for (const [l, a] of Object.entries(ul))
    o[l] = () => (n.push((i) => $e(i, t, a)), e);
  return e;
}
function Ee(t) {
  var n;
  return t.selection.anchor.blockId || ((n = t.doc.children[0]) == null ? void 0 : n.id) || "";
}
function $e(t, n, e, o) {
  const l = Ee(n), a = Y(t, l);
  if (!a) return t;
  let i = { ...a, kind: e };
  if (o)
    for (const r of o) i = { ...i, attrs: qe(i.attrs, r.key, r.value) };
  return we(t, l, [i]);
}
function bt(t, n, e) {
  const o = Ee(n), l = Y(t, o);
  if (!l) return t;
  const a = pe(l) + e;
  return we(t, o, [Je(o, l.kind, a)]);
}
function vl(t, n) {
  const o = t.slice(0, n).match(/(?:^|\s)\/([^\s/]*)$/);
  return o ? o[1] : null;
}
function ml(t, n, e) {
  if (t == null) {
    document.dispatchEvent(new CustomEvent("autodown:slash-close", { detail: {} }));
    return;
  }
  const o = {
    query: t,
    range: { from: e - t.length - 1, to: e },
    items: [],
    blockId: n
  };
  document.dispatchEvent(new CustomEvent("autodown:slash-open", { detail: o }));
}
function pl(t, n) {
  return On(t, n).tag;
}
function gl(t, n) {
  const e = On(t, n);
  return e.cls ? `autodown-block-host ${e.cls}` : "autodown-block-host";
}
function On(t, n) {
  if (t === "Heading") {
    const e = Math.min(6, Math.max(1, n ?? 1));
    return { tag: `h${e}`, cls: `heading-node heading-${e}` };
  }
  return t === "Paragraph" ? { tag: "p", cls: "paragraph-node" } : { tag: "div", cls: "" };
}
const at = /* @__PURE__ */ new WeakSet();
function kl(t, n) {
  var a;
  const e = Wn(), o = ((a = e == null ? void 0 : e.proxy) == null ? void 0 : a.$el) ?? null;
  if (!o) return;
  at.add(o), o.innerHTML = t, o.focus();
  const l = n ? to(n) : null;
  (!l || !no(o, l)) && Lt(o), Me(() => {
    at.delete(o), En() === o && At(null);
  });
}
function Lt(t) {
  const n = document.createRange();
  n.selectNodeContents(t), n.collapse(!1);
  const e = window.getSelection();
  e == null || e.removeAllRanges(), e == null || e.addRange(n);
}
function bl(t, n) {
  const e = t.ownerDocument.createTreeWalker(t, NodeFilter.SHOW_TEXT);
  let o = Math.max(0, n), l = null, a = !1;
  for (; l = e.nextNode(); ) {
    const i = l.data.length;
    if (o <= i) {
      const r = document.createRange();
      if (o === i) {
        const c = l.parentElement;
        if (c && c !== t) {
          const f = t.ownerDocument.createTextNode(wl);
          c.after(f), r.setStart(f, 1), r.collapse(!0);
          const d = window.getSelection();
          d == null || d.removeAllRanges(), d == null || d.addRange(r), a = !0;
          break;
        }
      }
      r.setStart(l, o), r.collapse(!0);
      const u = window.getSelection();
      u == null || u.removeAllRanges(), u == null || u.addRange(r), a = !0;
      break;
    }
    o -= i;
  }
  a || Lt(t);
}
const wl = "​";
function it(t) {
  return (t.textContent ?? "").replace(/\u00A0/g, " ").replace(/\u200B/g, "");
}
function He(t) {
  const n = window.getSelection();
  if (!n || n.rangeCount === 0) return 0;
  const e = n.getRangeAt(0).cloneRange();
  return e.selectNodeContents(t), e.setEnd(n.getRangeAt(0).endContainer, n.getRangeAt(0).endOffset), e.toString().length;
}
function nn(t) {
  const n = t.getClientRects(), e = [];
  for (let o = 0; o < n.length; o++)
    n[o].height > 0 && e.push(n[o].top);
  return e;
}
function Dn(t) {
  const n = window.getSelection();
  if (!n || n.rangeCount === 0) return { preTops: [], postTops: [] };
  const e = n.getRangeAt(0);
  if (!e.collapsed) return { preTops: [], postTops: [] };
  const o = document.createRange();
  o.selectNodeContents(t), o.setEnd(e.startContainer, e.startOffset);
  const l = document.createRange();
  return l.selectNodeContents(t), l.setStart(e.startContainer, e.startOffset), { preTops: nn(o), postTops: nn(l) };
}
function yl(t) {
  const { preTops: n, postTops: e } = Dn(t), o = e.length > 0 ? e[0] : n[n.length - 1];
  return o == null ? !0 : !n.some((l) => l < o - 1);
}
function _l(t) {
  const { preTops: n, postTops: e } = Dn(t), o = n.length > 0 ? n[n.length - 1] : e[0];
  return o == null ? !0 : !e.some((l) => l > o + 1);
}
function Cl(t) {
  setTimeout(() => {
    const n = document.querySelector(`.autodown-block-host[data-block-id="${t}"]`);
    if (!n) return;
    const e = document.createRange();
    e.selectNodeContents(n), e.collapse(!0);
    const o = window.getSelection();
    o == null || o.removeAllRanges(), o == null || o.addRange(e);
  }, 0);
}
function Sl(t, n) {
  const e = it(t);
  if (n.onInput(e), !n.composition.composing && it(t) !== n.text) {
    t.innerHTML = yn(n.inlines);
    const o = n.desiredCaretOffset();
    o == null ? Lt(t) : bl(t, o);
  }
  typeof document < "u" && ml(vl(n.text, He(t)), n.id, He(t));
}
function Il(t, n) {
  if (n.composition.composing) return;
  const e = t.currentTarget ?? t.target;
  if (t.ctrlKey || t.metaKey) {
    const o = t.key.toLowerCase();
    if (o === "b") {
      t.preventDefault(), Ie(ne, q.Strong);
      return;
    }
    if (o === "i") {
      t.preventDefault(), Ie(ne, q.Em);
      return;
    }
    if (o === "u") {
      t.preventDefault(), Ie(ne, q.Underline);
      return;
    }
    if (o === "k") {
      t.preventDefault();
      const l = window.prompt("Enter URL");
      l && ne.applyMark(q.Link, l);
      return;
    }
  }
  if (t.key === "Enter")
    t.preventDefault(), n.onEnter(He(e), `b-${Math.random().toString(36).slice(2, 8)}`);
  else if (t.key === "Backspace" && He(e) === 0)
    n.onBackspaceAtStart(n.prevSiblingId()) && t.preventDefault();
  else if (t.key === "ArrowUp" && yl(e))
    n.navigateUp() && t.preventDefault();
  else if (t.key === "ArrowDown" && _l(e)) {
    const o = n.navigateDown();
    o && (t.preventDefault(), Cl(o));
  } else t.key === "Tab" && n.onTab(t.shiftKey) && t.preventDefault();
}
function Bl(t, n) {
  var l;
  const e = ((l = t.clipboardData) == null ? void 0 : l.getData("text/plain")) ?? "";
  if (!e) return;
  t.preventDefault();
  const o = e.trim();
  if (!o.includes(`
`) && !/^[#>*`\-\d]/.test(o)) {
    n.onInput(n.text + o);
    return;
  }
  n.onPasteMarkdown(o);
}
function Tl(t, n) {
  n.compositionBegin(n.text, He(t));
}
function xl(t, n) {
  n.compositionUpdate(t.data ?? "");
}
function Rl(t, n) {
  n.compositionCommit(it(t));
}
function Ml(t, n) {
  at.has(t) && At(t);
}
function El(t, n) {
  if (At(null), !at.has(t)) return;
  const e = it(t);
  e !== n.text && n.onInput(e), n.onRichBlur(t);
}
const Al = /* @__PURE__ */ Z({
  __name: "RichTextHost",
  props: {
    controller: {},
    blockId: {},
    blockKind: {},
    level: {},
    initial_html: {}
  },
  emits: ["Init", "ClickStop", "Input", "Keydown", "Paste", "Focus", "Blur", "CompositionStart", "CompositionUpdate", "CompositionEnd"],
  setup(t, { emit: n }) {
    const e = t, o = v(() => pl(e.blockKind, e.level)), l = v(() => gl(e.blockKind, e.level)), a = n;
    function i(h) {
      El(h.target, e.controller), a("Blur", h);
    }
    function r(h) {
      a("ClickStop", h);
    }
    function u(h) {
      Rl(h.target, e.controller), a("CompositionEnd", h);
    }
    function c(h) {
      Tl(h.target, e.controller), a("CompositionStart", h);
    }
    function f(h) {
      xl(h, e.controller), a("CompositionUpdate", h);
    }
    function d(h) {
      Ml(h.target, e.controller), a("Focus", h);
    }
    function B(h) {
      Sl(h.target, e.controller), a("Input", h);
    }
    function D(h) {
      Il(h, e.controller), a("Keydown", h);
    }
    function O(h) {
      Bl(h, e.controller), a("Paste", h);
    }
    return fe(() => {
      kl(e.initial_html, e.blockId);
    }), (h, g) => (p(), X(ge(o.value), {
      class: Te(l.value),
      contenteditable: !0,
      "data-block-id": t.blockId,
      "data-node-type": t.blockKind,
      dir: "auto",
      spellcheck: "false",
      onBlur: g[0] || (g[0] = (A) => i(A)),
      onClick: g[1] || (g[1] = Oe((A) => r(A), ["stop"])),
      onCompositionend: g[2] || (g[2] = (A) => u(A)),
      onCompositionstart: g[3] || (g[3] = (A) => c(A)),
      onCompositionupdate: g[4] || (g[4] = (A) => f(A)),
      onFocus: g[5] || (g[5] = (A) => d(A)),
      onInput: g[6] || (g[6] = (A) => B(A)),
      onKeydown: g[7] || (g[7] = (A) => D(A)),
      onPaste: g[8] || (g[8] = (A) => O(A))
    }, null, 40, ["class", "data-block-id", "data-node-type"]));
  }
});
function Ll(t) {
  return t.startsWith("bottom") ? { vertical: "bottom", horizontal: t.endsWith("end") ? "right" : "left" } : { vertical: "top", horizontal: t.endsWith("end") ? "right" : "left" };
}
function ye(t, n, e, o, l = "bottom", a = 8, i = "left") {
  const { vertical: r, horizontal: u } = Ll(l);
  let c;
  if (r === "bottom") {
    if (c = t.bottom + a, c + e > o.height) {
      const d = t.top - e - a;
      d >= 0 ? c = d : c = Math.max(0, o.height - e);
    }
  } else if (c = t.top - e - a, c < 0) {
    const d = t.bottom + a;
    d + e <= o.height ? c = d : c = 0;
  }
  let f = u === "right" || i === "right" ? t.right - n : t.left;
  return f + n > o.width && (f = Math.max(0, o.width - n)), f < 0 && (f = 0), { top: c, left: f };
}
const Ol = {
  bold: Mo,
  italic: Ro,
  underline: xo,
  strike: To,
  code: xn,
  link: Tn
};
function Le(t) {
  return Ol[t];
}
function Dl({
  editor: t,
  state: n
}) {
  if (!t.isEditable || t.isActive("image"))
    return !1;
  const { empty: e } = n.selection;
  return e ? Array.isArray(n.marks) && n.marks.length > 0 : !0;
}
function $l(t, n) {
  var e, o, l, a;
  if (t.isActive("link"))
    (o = (e = t.chain().focus()).unsetLink) == null || o.call(e).run();
  else {
    const i = window.prompt(n ?? "Enter URL");
    i && ((a = (l = t.chain().focus()).setLink) == null || a.call(l, { href: i }).run());
  }
}
function ql(t) {
  const n = t.selection;
  return { selection: { empty: n.anchor.blockId === n.head.blockId && n.anchor.offset === n.head.offset } };
}
const Nl = Z({
  name: "EngineBubbleMenu",
  props: {
    editor: { type: Object, default: null },
    options: { type: Object, default: null },
    shouldShow: { type: Function, default: null }
  },
  setup(t, { slots: n }) {
    const e = b(!1), o = b("0px"), l = b("0px"), a = b(null);
    let i = null;
    const r = () => {
      var z, J, C, I, S, y;
      const d = (z = t.editor) == null ? void 0 : z.__engine;
      if (!d) {
        e.value = !1;
        return;
      }
      (C = (J = t.editor) == null ? void 0 : J.__bump) == null || C.call(J);
      const B = typeof window > "u" ? null : window.getSelection(), D = B && B.rangeCount > 0 ? B.getRangeAt(0) : null;
      if (!D) {
        e.value = !1;
        return;
      }
      let O;
      D.collapsed && (O = Ln());
      const h = { editor: t.editor, state: { ...ql(d), marks: O } };
      if (e.value = t.shouldShow ? !!t.shouldShow(h) : !1, !e.value) return;
      const g = (I = D.startContainer.nodeType === 3 ? D.startContainer.parentElement : D.startContainer) == null ? void 0 : I.closest(".autodown-block-host"), A = g == null ? void 0 : g.closest(".autodown-editor");
      if (!g || !A || !A.contains(g)) {
        e.value = !1;
        return;
      }
      const j = D.getBoundingClientRect(), E = A.getBoundingClientRect(), te = {
        top: j.top - E.top,
        left: j.left - E.left,
        bottom: j.bottom - E.top,
        right: j.right - E.left,
        width: j.width,
        height: j.height
      }, oe = ye(
        te,
        ((S = a.value) == null ? void 0 : S.offsetWidth) ?? 0,
        ((y = a.value) == null ? void 0 : y.offsetHeight) ?? 0,
        { width: A.clientWidth, height: A.clientHeight },
        "top"
      );
      l.value = `${oe.left}px`, o.value = `${oe.top}px`;
    }, u = (d) => {
      var B, D;
      e.value && !((D = (B = d.target) == null ? void 0 : B.closest) != null && D.call(B, ".autodown-bubble-menu")) && (e.value = !1);
    }, c = (d) => {
      d.key === "Escape" && e.value && (e.value = !1);
    }, f = () => r();
    return fe(() => {
      var B;
      const d = (B = t.editor) == null ? void 0 : B.__engine;
      d && (d.onChange(f), i = () => {
      }), document.addEventListener("pointerdown", u), document.addEventListener("keydown", c), document.addEventListener("selectionchange", f), r();
    }), Me(() => {
      i == null || i(), document.removeEventListener("pointerdown", u), document.removeEventListener("keydown", c), document.removeEventListener("selectionchange", f);
    }), () => {
      var d;
      return e.value ? ie(
        "div",
        {
          ref: a,
          class: "autodown-bubble-menu",
          style: { position: "absolute", top: o.value, left: l.value },
          // plan 024 P3T2: preventDefault keeps the contenteditable
          // host focused (and its selection alive) through button
          // clicks — the mark chains wrap the live host DOM.
          onMousedown: (B) => B.preventDefault()
        },
        (d = n.default) == null ? void 0 : d.call(n)
      ) : null;
    };
  }
}), Hl = ["title", "onClick"], Pl = /* @__PURE__ */ Z({
  __name: "BubbleMenu",
  props: {
    editor: {},
    linkPrompt: { default: "Enter URL" },
    tooltips: { default: null }
  },
  emits: ["RunButton"],
  setup(t, { emit: n }) {
    const e = t, o = v(() => [{ name: "bold", title: e.tooltips && e.tooltips.bold || "Bold", icon: Le("bold"), active: e.editor.isActive("bold"), action: () => e.editor.chain().focus().toggleBold().run() }, { name: "italic", title: e.tooltips && e.tooltips.italic || "Italic", icon: Le("italic"), active: e.editor.isActive("italic"), action: () => e.editor.chain().focus().toggleItalic().run() }, { name: "underline", title: e.tooltips && e.tooltips.underline || "Underline", icon: Le("underline"), active: e.editor.isActive("underline"), action: () => e.editor.chain().focus().toggleUnderline().run() }, { name: "strike", title: e.tooltips && e.tooltips.strike || "Strikethrough", icon: Le("strike"), active: e.editor.isActive("strike"), action: () => e.editor.chain().focus().toggleStrike().run() }, { name: "code", title: e.tooltips && e.tooltips.code || "Inline Code", icon: Le("code"), active: e.editor.isActive("code"), action: () => e.editor.chain().focus().toggleCode().run() }, { name: "link", title: e.tooltips && e.tooltips.link || "Link", icon: Le("link"), active: e.editor.isActive("link"), action: () => $l(e.editor, e.linkPrompt) }]), l = n;
    function a(i) {
      i.action(), l("RunButton", i);
    }
    return (i, r) => t.editor ? (p(), X(ce(Nl), {
      class: Te("autodown-bubble-menu"),
      editor: t.editor,
      options: { placement: "top" },
      shouldShow: ce(Dl),
      key: "TiptapBubbleMenu-1"
    }, {
      default: Ye(() => [
        (p(!0), R(_e, null, Pe(o.value, (u) => (p(), R("button", {
          class: Te(["autodown-bubble-btn", { active: u.active }]),
          key: u.title,
          title: u.title,
          onClick: (c) => a(u)
        }, [
          (p(), X(ge(u.icon), { size: 14 }))
        ], 10, Hl))), 128))
      ]),
      _: 1
    }, 8, ["editor", "shouldShow"])) : $("", !0);
  }
});
function Fl() {
  return Eo;
}
const Ul = [
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
function on() {
  return Ul;
}
const Kl = { class: "autodown-codeblock-menu-header" }, Wl = ["onKeydown"], Vl = ["onClick", "onMouseenter"], Ql = { class: "autodown-codeblock-menu-item-label" }, zl = {
  key: 0,
  class: "autodown-codeblock-menu-empty"
}, Gl = /* @__PURE__ */ Z({
  __name: "CodeBlockMenu",
  props: {
    editor: {}
  },
  emits: ["Init", "Destroy", "SearchInput", "MoveDown", "MoveUp", "SelectHighlighted", "SelectItem", "HoverItem", "Close", "OutsideClick"],
  setup(t, { emit: n }) {
    const e = t, o = b(!1), l = b(""), a = b(0), i = b(""), r = b(""), u = b(""), c = b(""), f = b(null), d = b(null), B = b(null), D = b(null), O = b(0), h = b(null), g = b(null), A = b(null), j = b(null), E = b(null), te = b(null), oe = b(null), z = v(() => on().filter((M) => [M.id, M.label].concat(M.aliases).join(" ").toLowerCase().includes(l.value.toLowerCase().trim()))), J = v(() => z.value.length === 0), C = v(() => Fl()), I = n;
    function S() {
      o.value = !1, l.value = "", a.value = 0, f.value = null, I("Close");
    }
    function y(M) {
      a.value = M, I("HoverItem", M);
    }
    function T() {
      a.value < z.value.length - 1 && (a.value = a.value + 1, re(() => {
        let M = null;
        if (d.value != null && (M = d.value.querySelector(".autodown-codeblock-menu")), M != null) {
          let U = M.querySelector(".autodown-codeblock-menu-list"), K = M.querySelector(".autodown-codeblock-menu-item.active");
          if (U != null && K != null) {
            let se = U.getBoundingClientRect(), ue = K.getBoundingClientRect(), L = ue.top - se.top - se.height / 2 + ue.height / 2;
            U.scrollTop = U.scrollTop + L;
          }
        }
      })), I("MoveDown");
    }
    function ee() {
      a.value > 0 && (a.value = a.value - 1, re(() => {
        let M = null;
        if (d.value != null && (M = d.value.querySelector(".autodown-codeblock-menu")), M != null) {
          let U = M.querySelector(".autodown-codeblock-menu-list"), K = M.querySelector(".autodown-codeblock-menu-item.active");
          if (U != null && K != null) {
            let se = U.getBoundingClientRect(), ue = K.getBoundingClientRect(), L = ue.top - se.top - se.height / 2 + ue.height / 2;
            U.scrollTop = U.scrollTop + L;
          }
        }
      })), I("MoveUp");
    }
    function he(M) {
      if (o.value) {
        let U = null;
        d.value != null && (U = d.value.querySelector(".autodown-codeblock-menu")), U != null && (U.contains(M.target) || (o.value = !1, l.value = "", a.value = 0, f.value = null));
      }
      I("OutsideClick", M);
    }
    function le(M) {
      l.value = M.target.value, I("SearchInput", M);
    }
    function ke() {
      if (z.value.length == 1) {
        let M = z.value[0];
        e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null;
      }
      if (z.value.length != 1) {
        let M = z.value[a.value];
        M != null && (e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null);
      }
      I("SelectHighlighted");
    }
    function ve(M) {
      e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null, I("SelectItem", M);
    }
    fe(() => {
      let M = e.editor.view.dom;
      B.value = M, d.value = M.closest(".autodown-editor"), D.value = M.closest(".autodown-editor-content-wrapper");
      let U = () => {
        O.value != 0 && cancelAnimationFrame(O.value), O.value = requestAnimationFrame(() => {
          if (O.value = 0, o.value && d.value != null) {
            let L = d.value.getBoundingClientRect(), H = f.value;
            if (H == null) {
              let F = e.editor.view, Q = F.nodeDOM(F.state.selection.from);
              Q != null && Q.closest != null && (H = Q.closest("pre[data-language]"), H == null && (H = Q.closest(".autodown-codeblock-node")));
            }
            if (H == null && (o.value = !1, l.value = "", a.value = 0, f.value = null), H != null) {
              let F = H.querySelector("[data-codeblock-language-badge]"), Q = H;
              F != null && (Q = F);
              let s = Q.getBoundingClientRect(), m = { top: s.top - L.top + 6, left: s.left - L.left, bottom: s.bottom - L.top + 6, right: s.right - L.left, width: s.width, height: s.height }, k = { width: L.width, height: L.height }, _ = ye(m, 0, 0, k, "bottom-end", 0);
              r.value = _.top + "px", u.value = _.left + "px", c.value = "hidden", re(() => {
                let N = d.value.querySelector(".autodown-codeblock-menu");
                if (N != null) {
                  let W = N.getBoundingClientRect(), P = ye(m, W.width, W.height, k, "bottom-end", 0);
                  r.value = P.top + "px", u.value = P.left + "px", c.value = "visible";
                }
              });
            }
          }
        });
      };
      j.value = U;
      let K = (L) => {
        if (o.value) {
          let H = null;
          if (d.value != null && (H = d.value.querySelector(".autodown-codeblock-menu")), H != null && H.contains(L.target)) {
            L.preventDefault(), L.stopPropagation();
            let F = H.querySelector(".autodown-codeblock-menu-list");
            if (F != null) {
              let Q = F.scrollTop + F.clientHeight < F.scrollHeight, s = F.scrollTop > 0;
              L.deltaY > 0 && Q && (F.scrollTop = F.scrollTop + L.deltaY), L.deltaY < 0 && s && (F.scrollTop = F.scrollTop + L.deltaY);
            }
          }
          H == null && (L.preventDefault(), L.stopPropagation()), H != null && !H.contains(L.target) && (L.preventDefault(), L.stopPropagation());
        }
      };
      h.value = K, document.addEventListener("wheel", h.value, { passive: !1, capture: !0 });
      let se = (L) => {
        let H = L.target, F = null, Q = null, s = null, m = null;
        H.closest != null && (F = H.closest("[data-codeblock-language-badge]"), Q = H.closest("[data-codeblock-copy-btn]"), s = H.closest("[data-codeblock-expand-btn]"), m = H.closest("[data-codeblock-more-btn]")), (F != null || Q != null || s != null || m != null) && (L.preventDefault(), L.stopPropagation());
      };
      g.value = se, M.addEventListener("mousedown", g.value, { capture: !0 });
      let ue = (L) => {
        let H = L.target, F = null, Q = null, s = null, m = null;
        if (H.closest != null && (F = H.closest("[data-codeblock-copy-btn]"), Q = H.closest("[data-codeblock-expand-btn]"), s = H.closest("[data-codeblock-language-badge]"), m = H.closest("[data-codeblock-more-btn]")), F != null) {
          L.preventDefault(), L.stopPropagation();
          let k = F.closest("pre");
          if (k == null) {
            let N = F.closest(".code-block-container");
            N != null && (k = N.querySelector("pre[data-language]"));
          }
          let _ = "";
          if (k != null) {
            let N = k.querySelector("code");
            N != null && (_ = N.textContent ?? "");
          }
          navigator.clipboard.writeText(_);
        }
        if (F == null && Q != null) {
          L.preventDefault(), L.stopPropagation();
          let k = Q.closest("pre");
          if (k == null) {
            let _ = Q.closest(".code-block-container");
            _ != null && (k = _.querySelector("pre[data-language]"));
          }
          k != null && k.classList.toggle("is-collapsed");
        }
        if (F == null && Q == null) {
          let k = s;
          if (k == null && (k = m), k != null) {
            let _ = k.closest("pre");
            _ == null && (_ = k.closest(".autodown-codeblock-node"));
            let N = !1;
            if (_ == null && (_ = k.closest(".code-block-container"), _ != null && (N = !0)), L.preventDefault(), N == !1 && L.stopPropagation(), _ == null && (_ = k.closest(".code-block-container")), _ == null) {
              let P = e.editor.view, V = P.nodeDOM(P.state.selection.from);
              V != null && V.closest != null && (_ = V.closest("pre[data-language]"), _ == null && (_ = V.closest(".autodown-codeblock-node")));
            }
            if (f.value = _, i.value = "", f.value != null && (i.value = f.value.getAttribute("data-language") ?? "", i.value == "")) {
              let P = f.value.querySelector("pre[data-language]");
              P != null && (i.value = P.getAttribute("data-language") ?? "");
            }
            i.value == "" && (i.value = e.editor.getAttributes("codeBlock").language ?? ""), o.value = !0, l.value = "";
            let W = on().findIndex((P) => P.id == i.value);
            a.value = W, W < 0 && (a.value = 0), re(() => {
              let P = null;
              if (d.value != null && (P = d.value.querySelector(".autodown-codeblock-menu")), P != null) {
                let V = P.querySelector(".autodown-codeblock-menu-search");
                V != null && V.focus();
              }
              if (d.value != null) {
                let V = d.value.getBoundingClientRect(), ae = f.value;
                if (ae == null) {
                  let Ce = e.editor.view, me = Ce.nodeDOM(Ce.state.selection.from);
                  me != null && me.closest != null && (ae = me.closest("pre[data-language]"), ae == null && (ae = me.closest(".autodown-codeblock-node")));
                }
                if (ae == null && (o.value = !1, l.value = "", a.value = 0, f.value = null), ae != null) {
                  let Ce = ae.querySelector("[data-codeblock-language-badge]"), me = ae;
                  Ce != null && (me = Ce);
                  let be = me.getBoundingClientRect(), We = { top: be.top - V.top + 6, left: be.left - V.left, bottom: be.bottom - V.top + 6, right: be.right - V.left, width: be.width, height: be.height }, $t = { width: V.width, height: V.height }, qt = ye(We, 0, 0, $t, "bottom-end", 0);
                  r.value = qt.top + "px", u.value = qt.left + "px", c.value = "hidden", re(() => {
                    let Nt = d.value.querySelector(".autodown-codeblock-menu");
                    if (Nt != null) {
                      let Ht = Nt.getBoundingClientRect(), Pt = ye(We, Ht.width, Ht.height, $t, "bottom-end", 0);
                      r.value = Pt.top + "px", u.value = Pt.left + "px", c.value = "visible";
                    }
                  });
                }
              }
              re(() => {
                let V = null;
                if (d.value != null && (V = d.value.querySelector(".autodown-codeblock-menu")), V != null) {
                  let ae = V.querySelector(".autodown-codeblock-menu-list"), Ce = V.querySelector(".autodown-codeblock-menu-item.active");
                  if (ae != null && Ce != null) {
                    let me = ae.getBoundingClientRect(), be = Ce.getBoundingClientRect(), We = be.top - me.top - me.height / 2 + be.height / 2;
                    ae.scrollTop = ae.scrollTop + We;
                  }
                }
              });
            });
          }
        }
      };
      A.value = ue, M.addEventListener("click", A.value, { capture: !0 }), D.value != null && D.value.addEventListener("scroll", j.value, { passive: !0 });
    }), yt(() => {
      document.removeEventListener("wheel", h.value, { capture: !0 }), B.value != null && (B.value.removeEventListener("mousedown", g.value, { capture: !0 }), B.value.removeEventListener("click", A.value, { capture: !0 })), D.value != null && D.value.removeEventListener("scroll", j.value);
    });
    function Ke(M) {
      he(M);
    }
    return fe(() => {
      document.addEventListener("mousedown", Ke);
    }), yt(() => {
      document.removeEventListener("mousedown", Ke);
    }), (M, U) => o.value ? (p(), R("div", {
      key: 0,
      class: "autodown-codeblock-menu",
      ref_key: "menuEl",
      ref: E,
      style: gn({ top: r.value, left: u.value, visibility: c.value })
    }, [
      w("div", Kl, [
        st(w("input", {
          class: "autodown-codeblock-menu-search",
          placeholder: "Search language…",
          ref_key: "searchEl",
          ref: te,
          "onUpdate:modelValue": U[0] || (U[0] = (K) => l.value = K),
          onInput: U[1] || (U[1] = (K) => le(K)),
          onKeydown: [
            Ve(Oe(T, ["prevent"]), ["down"]),
            Ve(Oe(ke, ["prevent"]), ["enter"]),
            Ve(S, ["esc"]),
            Ve(Oe(ee, ["prevent"]), ["up"])
          ]
        }, null, 40, Wl), [
          [Bt, l.value]
        ])
      ]),
      w("div", {
        class: "autodown-codeblock-menu-list",
        ref_key: "listEl",
        ref: oe
      }, [
        (p(!0), R(_e, null, Pe(z.value, (K, se) => (p(), R("button", {
          class: Te(["autodown-codeblock-menu-item", { active: se == a.value, selected: K.id == i.value }]),
          key: K.id,
          onClick: (ue) => ve(K),
          onMouseenter: (ue) => y(se)
        }, [
          w("span", Ql, [
            w("span", null, G(K.label), 1)
          ]),
          K.id == i.value ? (p(), X(ge(C.value), {
            key: 0,
            class: "autodown-codeblock-menu-check",
            size: 13
          })) : $("", !0)
        ], 42, Vl))), 128)),
        J.value ? (p(), R("div", zl, [...U[2] || (U[2] = [
          w("span", null, "No matching languages", -1)
        ])])) : $("", !0)
      ], 512)
    ], 4)) : $("", !0);
  }
}), jl = { class: "autodown-slash-menu-items" }, Yl = ["onClick", "onMouseenter"], Jl = { class: "autodown-slash-menu-info" }, Xl = { class: "autodown-slash-menu-title" }, Zl = { class: "autodown-slash-menu-desc" }, ea = {
  key: 0,
  class: "autodown-slash-menu-empty"
}, ta = /* @__PURE__ */ Z({
  __name: "SlashMenu",
  props: {
    editor: {},
    items: {},
    noResultsText: { default: "No results" }
  },
  emits: ["OnOpen", "OnUpdate", "OnClose", "OnKeydown", "SelectItem", "HoverItem"],
  setup(t, { emit: n }) {
    const e = t, o = b(!1), l = b(""), a = b(null), i = b(0), r = b(""), u = b(""), c = b(""), f = b(null), d = v(() => e.items.filter((C) => [C.title, C.description].concat(C.searchTerms).join(" ").toLowerCase().includes(l.value.toLowerCase()))), B = v(() => d.value.length === 0), D = v(() => e.noResultsText ?? "No results"), O = n;
    Ae(d, () => {
      i.value = 0;
    });
    function h(C) {
      i.value = C, O("HoverItem", C);
    }
    function g() {
      o.value = !1, l.value = "", a.value = null, i.value = 0, O("OnClose");
    }
    function A(C) {
      if (o.value) {
        if (C.detail.event.key == "ArrowDown") {
          C.detail.event.preventDefault();
          let I = i.value + 1;
          i.value = I % d.value.length, re(() => {
            if (f.value) {
              let S = f.value.querySelector(".autodown-slash-menu-item.active");
              S != null && S.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (C.detail.event.key == "ArrowUp") {
          C.detail.event.preventDefault();
          let I = i.value - 1 + d.value.length;
          i.value = I % d.value.length, re(() => {
            if (f.value) {
              let S = f.value.querySelector(".autodown-slash-menu-item.active");
              S != null && S.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (C.detail.event.key == "Enter" || C.detail.event.key == "NumpadEnter") {
          C.detail.event.preventDefault();
          let I = d.value[i.value];
          I != null && a.value != null && (I.command({ editor: e.editor, range: a.value }), o.value = !1, l.value = "", a.value = null, i.value = 0), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        C.detail.event.key == "Escape" && (C.detail.event.preventDefault(), o.value = !1, l.value = "", a.value = null, i.value = 0, e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0));
      }
      O("OnKeydown", C);
    }
    function j(C) {
      l.value = C.detail.query, a.value = C.detail.range, o.value = !0, i.value = 0, re(() => {
        if (a.value != null && e.editor.view) {
          let I = e.editor.view.coordsAtPos(a.value.from), S = e.editor.view.dom.closest(".autodown-editor");
          if (S != null) {
            let y = S.getBoundingClientRect(), T = { top: I.top - y.top, left: I.left - y.left, bottom: I.bottom - y.top, right: I.right - y.left, width: I.right - I.left, height: I.bottom - I.top }, ee = { width: y.width, height: y.height }, he = ye(T, 0, 0, ee, "bottom", 8, "left");
            r.value = he.top + "px", u.value = he.left + "px", c.value = "hidden", re(() => {
              let le = S.querySelector(".autodown-slash-menu");
              if (le != null) {
                let ke = le.getBoundingClientRect(), ve = ye(T, ke.width, ke.height, ee, "bottom", 8, "left");
                r.value = ve.top + "px", u.value = ve.left + "px", c.value = "visible";
              }
            });
          }
        }
      }), O("OnOpen", C);
    }
    function E(C) {
      l.value = C.detail.query, a.value = C.detail.range, re(() => {
        if (a.value != null && e.editor.view) {
          let I = e.editor.view.coordsAtPos(a.value.from), S = e.editor.view.dom.closest(".autodown-editor");
          if (S != null) {
            let y = S.getBoundingClientRect(), T = { top: I.top - y.top, left: I.left - y.left, bottom: I.bottom - y.top, right: I.right - y.left, width: I.right - I.left, height: I.bottom - I.top }, ee = { width: y.width, height: y.height }, he = ye(T, 0, 0, ee, "bottom", 8, "left");
            r.value = he.top + "px", u.value = he.left + "px", c.value = "hidden", re(() => {
              let le = S.querySelector(".autodown-slash-menu");
              if (le != null) {
                let ke = le.getBoundingClientRect(), ve = ye(T, ke.width, ke.height, ee, "bottom", 8, "left");
                r.value = ve.top + "px", u.value = ve.left + "px", c.value = "visible";
              }
            });
          }
        }
      }), O("OnUpdate", C);
    }
    function te(C) {
      let I = d.value[C];
      I != null && a.value != null && (I.command({ editor: e.editor, range: a.value }), o.value = !1, l.value = "", a.value = null, i.value = 0), O("SelectItem", C);
    }
    function oe(C) {
      A(C);
    }
    function z(C) {
      j(C);
    }
    function J(C) {
      E(C);
    }
    return fe(() => {
      document.addEventListener("autodown:slash-close", g), document.addEventListener("autodown:slash-keydown", oe), document.addEventListener("autodown:slash-open", z), document.addEventListener("autodown:slash-update", J);
    }), yt(() => {
      document.removeEventListener("autodown:slash-close", g), document.removeEventListener("autodown:slash-keydown", oe), document.removeEventListener("autodown:slash-open", z), document.removeEventListener("autodown:slash-update", J);
    }), (C, I) => o.value ? (p(), R("div", {
      key: 0,
      class: "autodown-slash-menu",
      ref_key: "menuEl",
      ref: f,
      style: gn({ top: r.value, left: u.value, visibility: c.value })
    }, [
      w("div", jl, [
        (p(!0), R(_e, null, Pe(d.value, (S, y) => (p(), R("button", {
          class: Te(["autodown-slash-menu-item", { active: y == i.value }]),
          key: S.title,
          onClick: (T) => te(y),
          onMouseenter: (T) => h(y)
        }, [
          (p(), X(ge(S.icon), {
            class: "autodown-slash-menu-icon",
            size: 16
          })),
          w("div", Jl, [
            w("div", Xl, [
              w("span", null, G(S.title), 1)
            ]),
            w("div", Zl, [
              w("span", null, G(S.description), 1)
            ])
          ])
        ], 42, Yl))), 128)),
        B.value ? (p(), R("div", ea, [
          w("span", null, G(D.value), 1)
        ])) : $("", !0)
      ])
    ], 4)) : $("", !0);
  }
});
function na(t) {
  var o, l;
  const n = t == null ? void 0 : t.__engine, e = (l = (o = n == null ? void 0 : n.selection) == null ? void 0 : o.anchor) == null ? void 0 : l.blockId;
  return !n || !e ? null : oo(n, e);
}
function oa(t) {
  return [...[
    {
      title: "Text",
      description: "Plain text",
      icon: Ao,
      searchTerms: ["p"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setParagraph().run()
    },
    {
      title: "Heading 1",
      description: "Big section heading",
      icon: Lo,
      searchTerms: ["h1"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 1 }).run()
    },
    {
      title: "Heading 2",
      description: "Medium section heading",
      icon: Oo,
      searchTerms: ["h2"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 2 }).run()
    },
    {
      title: "Heading 3",
      description: "Small section heading",
      icon: Do,
      searchTerms: ["h3"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 3 }).run()
    },
    {
      title: "Heading 4",
      description: "Fourth level heading",
      icon: $o,
      searchTerms: ["h4"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 4 }).run()
    },
    {
      title: "Heading 5",
      description: "Fifth level heading",
      icon: qo,
      searchTerms: ["h5"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 5 }).run()
    },
    {
      title: "Heading 6",
      description: "Sixth level heading",
      icon: No,
      searchTerms: ["h6"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHeading({ level: 6 }).run()
    },
    {
      title: "Bullet List",
      description: "Bullet list",
      icon: Ho,
      searchTerms: ["ul"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleBulletList().run()
    },
    {
      title: "Numbered List",
      description: "Numbered list",
      icon: Po,
      searchTerms: ["ol"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleOrderedList().run()
    },
    {
      title: "Task List",
      description: "Task list",
      icon: Fo,
      searchTerms: ["task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleTaskList().run()
    },
    {
      title: "Code Block",
      description: "Code snippet",
      icon: xn,
      searchTerms: ["code"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setCodeBlock({ language: "text" }).run();
      }
    },
    {
      title: "Quote",
      description: "Quote",
      icon: Uo,
      searchTerms: ["blockquote"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).toggleBlockquote().run()
    },
    {
      title: "Divider",
      description: "Horizontal rule",
      icon: Ko,
      searchTerms: ["hr"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).setHorizontalRule().run()
    },
    {
      title: "Image",
      description: "Embed image",
      icon: Wo,
      searchTerms: ["img"],
      command: ({ editor: e, range: o }) => {
        const l = window.prompt(t.imageUrlPrompt);
        l && e.chain().focus().deleteRange(o).setImage({ src: l }).run();
      }
    },
    {
      title: "Table",
      description: "Add table",
      icon: Vo,
      searchTerms: ["table"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertTable({ rows: 3, cols: 3, withHeaderRow: !0 }).run()
    },
    {
      title: "Callout",
      description: "Admonition / callout box",
      icon: Qo,
      searchTerms: ["callout", "admonition", "warning", "tip", "note"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setCallout({ type: "note", title: "Note" }).run();
      }
    },
    {
      title: "Details",
      description: "Collapsible details block",
      icon: zo,
      searchTerms: ["details", "toggle", "collapse", "accordion"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setDetails({ summary: "Details" }).run();
      }
    },
    {
      title: "Math",
      description: "Block math formula (KaTeX)",
      icon: Go,
      searchTerms: ["math", "katex", "formula", "equation", "latex"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setMathBlock().run();
      }
    },
    {
      title: "Mermaid",
      description: "Mermaid diagram",
      icon: jo,
      searchTerms: ["mermaid", "diagram", "chart", "flowchart"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).setMermaidBlock().run();
      }
    },
    {
      title: "TODO",
      description: "Insert a TODO task",
      icon: Yo,
      searchTerms: ["todo", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- TODO ").run()
    },
    {
      title: "DOING",
      description: "Insert a DOING task",
      icon: Jo,
      searchTerms: ["doing", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- DOING ").run()
    },
    {
      title: "DONE",
      description: "Insert a DONE task",
      icon: Xo,
      searchTerms: ["done", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- DONE ").run()
    },
    {
      title: "NOW",
      description: "Insert a NOW task",
      icon: Zo,
      searchTerms: ["now", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- NOW ").run()
    },
    {
      title: "LATER",
      description: "Insert a LATER task",
      icon: el,
      searchTerms: ["later", "task"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("- LATER ").run()
    },
    {
      title: "Priority A",
      description: "Insert [#A] priority",
      icon: kt,
      searchTerms: ["priority", "A"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#A] ").run()
    },
    {
      title: "Priority B",
      description: "Insert [#B] priority",
      icon: kt,
      searchTerms: ["priority", "B"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#B] ").run()
    },
    {
      title: "Priority C",
      description: "Insert [#C] priority",
      icon: kt,
      searchTerms: ["priority", "C"],
      command: ({ editor: e, range: o }) => e.chain().focus().deleteRange(o).insertContent("[#C] ").run()
    },
    {
      title: "Query",
      description: "Insert a query macro",
      icon: tl,
      searchTerms: ["query", "macro"],
      command: ({ editor: e, range: o }) => {
        const l = window.prompt("Query (e.g. (task TODO DOING))", "(task TODO)");
        l && e.chain().focus().deleteRange(o).insertContent(`{{query ${l}}}`).run();
      }
    },
    {
      title: "Block link",
      description: "Copy link to current block",
      icon: Tn,
      searchTerms: ["block link", "anchor", "copy link"],
      command: ({ editor: e, range: o }) => {
        e.chain().focus().deleteRange(o).run();
        const l = t.pageTitle, a = na(e);
        if (l && a) {
          const i = `[[${l}#^${a}]]`;
          navigator.clipboard.writeText(i).catch(() => {
          });
        }
      }
    }
  ], ...t.extraSlashItems ?? []];
}
const $n = "autodown-node-view-content";
Z({
  name: "NodeViewWrapper",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(t, { slots: n, attrs: e }) {
    return () => {
      var o;
      return ie(t.as, { ...e, "data-node-view-wrapper": "" }, (o = n.default) == null ? void 0 : o.call(n));
    };
  }
});
Z({
  name: "NodeViewContent",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(t, { slots: n, attrs: e }) {
    const o = Vn($n, null);
    return () => {
      var a;
      const l = o ? o() : ((a = n.default) == null ? void 0 : a.call(n)) ?? [];
      return ie(t.as, { ...e, "data-node-view-content": "" }, l);
    };
  }
});
let qn = {};
function wt(t) {
  qn = { runQuery: t.runQuery, loadBlock: t.loadBlock };
}
function Ot() {
  return qn;
}
const rt = [];
function la(t) {
  rt.push(t);
}
function aa() {
  rt.pop();
}
function ia() {
  return rt[rt.length - 1];
}
Z({
  name: "NodeViewContentProvider",
  props: { content: { type: Function, required: !0 } },
  setup(t, { slots: n }) {
    return Qn($n, () => t.content()), () => {
      var e;
      return (e = n.default) == null ? void 0 : e.call(n);
    };
  }
});
class ra {
  constructor(n, e) {
    this.engine = n, this.tableId = e;
  }
  table() {
    return Y(this.engine.doc, this.tableId);
  }
  get rows() {
    var n;
    return ((n = this.table()) == null ? void 0 : n.children) ?? [];
  }
  cellText(n) {
    const e = Y(this.engine.doc, n);
    return e ? pe(e) : "";
  }
  /** Append an empty row after the last row (or after the header when only
   *  the header exists). One undo step. */
  addRow() {
    const n = this.rows[this.rows.length - 1];
    Wt(this.engine, this.tableId, (n == null ? void 0 : n.id) ?? null);
  }
  /** Insert an empty row ABOVE the header (TableMenu absorption, plan 026
   *  adjudication #1 — single table entry). One undo step. */
  addRowAbove() {
    Wt(this.engine, this.tableId, null);
  }
  /** Insert an empty column at index 0. One undo step. */
  addColumnBefore() {
    this.engine.applyTree((n) => _t(n, this.tableId, 0));
  }
  /** Remove the whole table; the dangling selection collapses to the first
   *  block (the menu chain's deleteTable repair, same semantics). */
  deleteTable() {
    this.engine.applyTree((n) => we(n, this.tableId, [])), !Y(this.engine.doc, this.engine.selection.anchor.blockId) && this.engine.doc.children[0] && this.engine.select(Ze(this.engine.doc.children[0].id, 0));
  }
  /** Remove the last row. Refused (no-op) when only the header remains. */
  deleteRow() {
    return this.rows.length <= 1 ? !1 : (lo(this.engine, this.rows[this.rows.length - 1].id), !0);
  }
  /** Append an empty column. One undo step. */
  addColumn() {
    ao(this.engine, this.tableId);
  }
  /** Remove the last column. Refused (no-op) below one column. */
  deleteColumn() {
    var e;
    return (((e = this.rows[0]) == null ? void 0 : e.children.length) ?? 0) <= 1 ? !1 : (io(this.engine, this.tableId), !0);
  }
  /** Cell blur-commit: old→new text as one diff op (BlockHost protocol).
   *  The selection stays anchored on the TABLE — the op's position points at
   *  the cell and applyOp would otherwise drag the anchor into it, dropping
   *  the top-level focus that assembles this editing face (found live in the
   *  demo: committing a cell unmounted the table editor).
   *  Returns false when the text is unchanged or the cell is gone. */
  commitCell(n, e) {
    const o = Y(this.engine.doc, n);
    if (!o || o.kind !== x.TableCell) return !1;
    const l = pe(o);
    if (l === e) return !1;
    const a = ro(n, l, e);
    if (!a) return !1;
    const i = this.engine.selection;
    return this.engine.apply(a), this.engine.selection.anchor.blockId !== i.anchor.blockId && this.engine.select(i), !0;
  }
}
function ln(t) {
  const n = Ct(t, !0);
  return n.error === "" && _n("MathBlock", t, { kind: "html", body: n.html, error: "" }), n;
}
function Nn(t) {
  const n = t.split(`
`).length;
  return String(Math.max(4, Math.min(24, n + 1)));
}
const sa = ["data-block-id", "data-math-block", "data-node-type"], ua = {
  key: 0,
  class: "autodown-stream-banner"
}, ca = { class: "math-editor-stack" }, da = ["innerHTML"], fa = {
  key: 1,
  class: "autodown-math-error",
  title: "Math preview error"
}, ha = ["disabled", "rows"], va = ["innerHTML"], ma = {
  key: 1,
  class: "autodown-math-error",
  title: "Math preview error"
}, pa = { class: "math-block-source" }, ga = /* @__PURE__ */ Z({
  __name: "MathBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(t, { emit: n }) {
    const e = t, o = b(""), l = b(""), a = b(tt(e.node)), i = b(Cn(e.ctx)), r = b(null), u = v(() => e.mode === "edit"), c = v(() => tt(e.node)), f = v(() => xt(e.ctx)), d = v(() => u.value ? f.value ? "autodown-math-editor is-readonly" : "autodown-math-editor" : "autodown-math-block"), B = v(() => Sn(e.mode)), D = v(() => nt(e.mode, Rt(e.ctx))), O = v(() => nt(e.mode, "MathBlock")), h = v(() => {
      var S;
      return (S = Ct(a.value, !0)) == null ? void 0 : S.html;
    }), g = v(() => {
      var S;
      return (S = Ct(a.value, !0)) == null ? void 0 : S.error;
    }), A = v(() => !g.value), j = v(() => !!g.value), E = v(() => Nn(a.value)), te = v(() => !l.value), oe = v(() => !!l.value), z = v(() => "code"), J = n;
    Ae(c, () => {
      if (!u.value) {
        let S = ln(c.value);
        o.value = S.html, l.value = S.error;
      }
    });
    function C(S) {
      a.value = S.target.value, J("AreaInput", S);
    }
    function I(S) {
      f.value || i.value.commit(S.target.value), J("Blur", S);
    }
    return fe(() => {
      if (u.value && In(r.value, f.value), !u.value) {
        let S = ln(c.value);
        o.value = S.html, l.value = S.error;
      }
    }), (S, y) => (p(), R("div", {
      class: Te(d.value),
      "data-block-id": D.value,
      "data-math-block": B.value,
      "data-node-type": O.value
    }, [
      u.value ? (p(), R(_e, { key: 0 }, [
        f.value ? (p(), R("div", ua, [...y[3] || (y[3] = [
          w("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        w("div", ca, [
          A.value ? (p(), R("div", {
            key: 0,
            class: "autodown-math-preview",
            innerHTML: h.value
          }, null, 8, da)) : $("", !0),
          j.value ? (p(), R("div", fa, [
            w("span", null, G(g.value), 1)
          ])) : $("", !0),
          st(w("textarea", {
            class: "math-editor-textarea",
            disabled: f.value,
            ref_key: "area",
            ref: r,
            rows: E.value,
            spellcheck: "false",
            "onUpdate:modelValue": y[0] || (y[0] = (T) => a.value = T),
            onBlur: y[1] || (y[1] = (T) => I(T)),
            onInput: y[2] || (y[2] = (T) => C(T))
          }, null, 40, ha), [
            [Bt, a.value]
          ])
        ])
      ], 64)) : $("", !0),
      u.value ? $("", !0) : (p(), R(_e, { key: 1 }, [
        te.value ? (p(), R("div", {
          key: 0,
          class: "autodown-math-preview",
          innerHTML: o.value
        }, null, 8, va)) : $("", !0),
        oe.value ? (p(), R("div", ma, [
          w("span", null, G(l.value), 1)
        ])) : $("", !0),
        w("pre", pa, [
          y[4] || (y[4] = Xe("          ", -1)),
          (p(), X(ge(z.value))),
          y[5] || (y[5] = Xe(`
        `, -1))
        ])
      ], 64))
    ], 10, sa));
  }
}), Hn = /* @__PURE__ */ ct(ga, [["__scopeId", "data-v-bf30707d"]]);
async function an(t) {
  const n = await Bn(t);
  return n.error === "" && _n("Mermaid", t, { kind: "svg", body: n.svg, error: "" }), n;
}
let je = null, rn = 0;
const ka = 300;
function sn(t, n) {
  const e = ++rn;
  if (je != null && clearTimeout(je), t.trim() === "") {
    n({ svg: "", error: "", loading: !1 });
    return;
  }
  je = setTimeout(() => {
    je = null, n({ svg: "", error: "", loading: !0 }), Bn(t).then((o) => {
      e === rn && n({ svg: o.svg, error: o.error, loading: !1 });
    });
  }, ka);
}
const ba = ["data-block-id", "data-mermaid-block", "data-node-type"], wa = {
  key: 0,
  class: "autodown-stream-banner"
}, ya = { class: "mermaid-editor-stack" }, _a = ["innerHTML"], Ca = {
  key: 1,
  class: "autodown-mermaid-error",
  title: "Mermaid render error"
}, Sa = {
  key: 2,
  class: "mermaid-editor-loading"
}, Ia = ["disabled", "rows"], Ba = ["innerHTML"], Ta = {
  key: 1,
  class: "autodown-mermaid-error",
  title: "Mermaid render error"
}, xa = { class: "mermaid-source" }, Ra = /* @__PURE__ */ Z({
  __name: "MermaidBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(t, { emit: n }) {
    const e = t, o = b(""), l = b(""), a = b(tt(e.node)), i = b(""), r = b(""), u = b(!1), c = b(Cn(e.ctx)), f = b(null), d = v(() => e.mode === "edit"), B = v(() => tt(e.node)), D = v(() => xt(e.ctx)), O = v(() => d.value ? D.value ? "autodown-mermaid-editor is-readonly" : "autodown-mermaid-editor" : "autodown-mermaid-block"), h = v(() => Sn(e.mode)), g = v(() => nt(e.mode, Rt(e.ctx))), A = v(() => nt(e.mode, "Mermaid")), j = v(() => u.value === !1 && !r.value && !!i.value), E = v(() => u.value === !1 && !!r.value), te = v(() => Nn(a.value)), oe = v(() => !!o.value), z = v(() => !o.value && !!l.value), J = v(() => "code"), C = n;
    Ae(B, () => {
      d.value || (B.value.trim() == "" && (o.value = "", l.value = ""), B.value.trim() != "" && an(B.value).then((T) => {
        o.value = T.svg, l.value = T.error;
      }));
    });
    function I(y) {
      a.value = y.target.value, u.value = !0, sn(a.value, (T) => {
        i.value = T.svg, r.value = T.error, u.value = T.loading;
      }), C("AreaInput", y);
    }
    function S(y) {
      D.value || c.value.commit(y.target.value), C("Blur", y);
    }
    return fe(() => {
      d.value && (In(f.value, D.value), u.value = !0, sn(B.value, (y) => {
        i.value = y.svg, r.value = y.error, u.value = y.loading;
      })), d.value || (B.value.trim() == "" && (o.value = "", l.value = ""), B.value.trim() != "" && an(B.value).then((T) => {
        o.value = T.svg, l.value = T.error;
      }));
    }), (y, T) => (p(), R("div", {
      class: Te(O.value),
      "data-block-id": g.value,
      "data-mermaid-block": h.value,
      "data-node-type": A.value
    }, [
      d.value ? (p(), R(_e, { key: 0 }, [
        D.value ? (p(), R("div", wa, [...T[3] || (T[3] = [
          w("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        w("div", ya, [
          j.value ? (p(), R("div", {
            key: 0,
            class: "autodown-mermaid-preview",
            innerHTML: i.value
          }, null, 8, _a)) : $("", !0),
          E.value ? (p(), R("div", Ca, [
            w("span", null, G(r.value), 1)
          ])) : $("", !0),
          u.value ? (p(), R("div", Sa, [...T[4] || (T[4] = [
            w("span", null, "渲染中…", -1)
          ])])) : $("", !0),
          st(w("textarea", {
            class: "mermaid-editor-textarea",
            disabled: D.value,
            ref_key: "area",
            ref: f,
            rows: te.value,
            spellcheck: "false",
            "onUpdate:modelValue": T[0] || (T[0] = (ee) => a.value = ee),
            onBlur: T[1] || (T[1] = (ee) => S(ee)),
            onInput: T[2] || (T[2] = (ee) => I(ee))
          }, null, 40, Ia), [
            [Bt, a.value]
          ])
        ])
      ], 64)) : $("", !0),
      d.value ? $("", !0) : (p(), R(_e, { key: 1 }, [
        oe.value ? (p(), R("div", {
          key: 0,
          class: "autodown-mermaid-preview",
          innerHTML: o.value
        }, null, 8, Ba)) : $("", !0),
        z.value ? (p(), R("div", Ta, [
          w("span", null, G(l.value), 1)
        ])) : $("", !0),
        w("pre", xa, [
          T[5] || (T[5] = Xe("          ", -1)),
          (p(), X(ge(J.value))),
          T[6] || (T[6] = Xe(`
        `, -1))
        ])
      ], 64))
    ], 10, ba));
  }
}), Pn = /* @__PURE__ */ ct(Ra, [["__scopeId", "data-v-08feae74"]]), Ma = ["data-open"], Ea = { class: "autodown-details-summary" }, Aa = { class: "autodown-details-content" }, La = {
  key: 0,
  class: "markdown-renderer"
}, It = /* @__PURE__ */ Z({
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
  setup(t, { emit: n }) {
    const e = t, o = v(() => e.mode === "edit"), l = v(() => xt(e.ctx));
    v(() => Rt(e.ctx));
    const a = v(() => so(e.ctx)), i = v(() => uo(e.node, e.ctx)), r = v(() => co(e.node, "open")), u = v(() => r.value ? "▼" : "▶"), c = v(() => fo(e.node, "summary")), f = v(() => c.value ? c.value : "Details"), d = n;
    function B() {
      vo(a.value, i.value, r.value), d("ToggleOpen");
    }
    return (D, O) => (p(), R("div", {
      class: "autodown-details",
      "data-open": r.value
    }, [
      w("div", Ea, [
        w("span", {
          class: "autodown-details-marker",
          "aria-hidden": "true",
          title: "点击展开详细内容",
          onClick: Oe(B, ["stop"])
        }, [
          w("span", null, G(u.value), 1)
        ]),
        o.value ? (p(), X(ce(ho), {
          attr_key: "summary",
          blockId: i.value,
          controller: a.value,
          host_class: "autodown-details-summary-text",
          placeholder: "Details",
          readonly: l.value,
          value: c.value,
          version: t.version,
          key: "AttrHost-1"
        }, null, 8, ["blockId", "controller", "readonly", "value", "version"])) : $("", !0),
        o.value ? $("", !0) : (p(), R("span", {
          key: 1,
          class: "autodown-details-summary-text",
          onClick: Oe(B, ["stop"])
        }, [
          w("span", null, G(f.value), 1)
        ]))
      ]),
      st(w("div", Aa, [
        o.value ? (p(), R("div", La, [
          (p(), X(ce(ot), {
            children_slot: t.children,
            key: "BlockChildren-2"
          }, null, 8, ["children_slot"]))
        ])) : $("", !0),
        o.value ? $("", !0) : (p(), X(ce(ot), {
          children_slot: t.children,
          key: "BlockChildren-3"
        }, null, 8, ["children_slot"]))
      ], 512), [
        [zn, r.value]
      ])
    ], 8, Ma));
  }
});
function Oa(t) {
  return ut((t == null ? void 0 : t.attrs) ?? [], "query", "");
}
function un() {
  return Ot().runQuery ?? null;
}
function cn(t) {
  return (t && t.results || []).map((e) => ({
    ...e,
    source: e.title || e.page_path,
    priority_label: e.priority ? `[#${e.priority}]` : ""
  }));
}
function dn(t) {
  return (t == null ? void 0 : t.message) || String(t);
}
const Da = {
  class: "autodown-query-block",
  "data-query-block": ""
}, $a = { class: "query-header" }, qa = {
  key: 0,
  class: "query-state"
}, Na = {
  key: 1,
  class: "query-state query-error"
}, Ha = { class: "result-marker" }, Pa = {
  key: 0,
  class: "result-priority"
}, Fa = { class: "result-content" }, Ua = { class: "result-source" }, Ka = {
  key: 3,
  class: "query-state"
}, Wa = /* @__PURE__ */ Z({
  __name: "QueryBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(t, { emit: n }) {
    const e = t, o = b([]), l = b(!1), a = b(""), i = v(() => Oa(e.node)), r = v(() => "code"), u = v(() => "ul"), c = v(() => "li"), f = v(() => l.value || !e.final), d = v(() => e.final && !l.value && !!a.value), B = v(() => e.final && !l.value && !a.value && o.value.length > 0), D = v(() => e.final && !l.value && !a.value && o.value.length === 0);
    return Ae(i, async () => {
      if (e.final) {
        let O = un();
        if ((O == null || i.value == "") && (a.value = "No query runner configured"), O != null && i.value != "") {
          l.value = !0, a.value = "";
          try {
            let h = await O(i.value);
            o.value = cn(h);
          } catch (h) {
            a.value = dn(h), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), fe(async () => {
      if (e.final) {
        let O = un();
        if ((O == null || i.value == "") && (a.value = "No query runner configured"), O != null && i.value != "") {
          l.value = !0, a.value = "";
          try {
            let h = await O(i.value);
            o.value = cn(h);
          } catch (h) {
            a.value = dn(h), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), (O, h) => (p(), R("div", Da, [
      w("div", $a, [
        h[0] || (h[0] = w("span", { class: "query-label" }, [
          w("span", null, "Query")
        ], -1)),
        (p(), X(ge(r.value), { class: "query-code" }, {
          default: Ye(() => [
            w("span", null, G(i.value), 1)
          ]),
          _: 1
        }))
      ]),
      f.value ? (p(), R("div", qa, [...h[1] || (h[1] = [
        w("span", null, "Loading query…", -1)
      ])])) : $("", !0),
      d.value ? (p(), R("div", Na, [
        w("span", null, G(a.value), 1)
      ])) : $("", !0),
      B.value ? (p(), X(ge(u.value), {
        key: 2,
        class: "query-results"
      }, {
        default: Ye(() => [
          (p(!0), R(_e, null, Pe(o.value, (g, A) => (p(), X(ge(c.value), {
            class: "query-result",
            key: A
          }, {
            default: Ye(() => [
              w("span", Ha, [
                w("span", null, G(g.marker), 1)
              ]),
              g.priority ? (p(), R("span", Pa, [
                w("span", null, G(g.priority_label), 1)
              ])) : $("", !0),
              w("span", Fa, [
                w("span", null, G(g.content), 1)
              ]),
              w("span", Ua, [
                w("span", null, G(g.source), 1)
              ])
            ]),
            _: 2
          }, 1024))), 128))
        ]),
        _: 1
      })) : $("", !0),
      D.value ? (p(), R("div", Ka, [...h[2] || (h[2] = [
        w("span", null, "No results", -1)
      ])])) : $("", !0)
    ]));
  }
}), Va = /* @__PURE__ */ ct(Wa, [["__scopeId", "data-v-45f3e7c3"]]);
function Fn(t) {
  if (t.startsWith("^"))
    return { title: "", blockId: t.length > 1 ? t.slice(1) : null };
  const n = t.indexOf("#^");
  return n >= 0 ? { title: t.slice(0, n), blockId: t.slice(n + 2) || null } : { title: t, blockId: null };
}
function Un(t) {
  return ut((t == null ? void 0 : t.attrs) ?? [], "src", "");
}
function Qa(t) {
  return Fn(Un(t)).title;
}
function za(t) {
  return Fn(Un(t)).blockId;
}
function fn() {
  return Ot().loadBlock ?? null;
}
function hn(t) {
  return (t == null ? void 0 : t.message) || String(t);
}
const Ga = ["data-title"], ja = {
  key: 0,
  class: "embed-state"
}, Ya = {
  key: 1,
  class: "embed-state embed-error"
}, Ja = {
  key: 2,
  class: "embed-header"
}, Xa = { class: "embed-title" }, Za = {
  key: 3,
  class: "embed-content"
}, ei = /* @__PURE__ */ Z({
  __name: "EmbedBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(t, { emit: n }) {
    const e = t, o = b(null), l = b(!1), a = b(""), i = v(() => Qa(e.node)), r = v(() => za(e.node)), u = v(() => r.value != null ? i.value ? i.value + "#" + r.value : r.value : i.value), c = v(() => "Loading " + u.value + "…" || "Loading…"), f = v(() => o.value && o.value.content || ""), d = v(() => l.value || !e.final), B = v(() => e.final && !l.value && !!a.value), D = v(() => e.final && !l.value && !a.value && o.value), O = v(() => e.final && !l.value && !a.value);
    return Ae(r, async () => {
      if (e.final && r.value != null) {
        let h = fn();
        if (h == null && (a.value = "No block loader configured"), h != null) {
          l.value = !0, a.value = "";
          try {
            let g = await h(r.value);
            o.value = g, g || (a.value = "Block not found");
          } catch (g) {
            a.value = hn(g), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), fe(async () => {
      if (e.final && r.value != null) {
        let h = fn();
        if (h == null && (a.value = "No block loader configured"), h != null) {
          l.value = !0, a.value = "";
          try {
            let g = await h(r.value);
            o.value = g, g || (a.value = "Block not found");
          } catch (g) {
            a.value = hn(g), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), (h, g) => (p(), R("div", {
      class: "autodown-block-embed",
      "data-title": i.value
    }, [
      d.value ? (p(), R("div", ja, [
        w("span", null, G(c.value), 1)
      ])) : $("", !0),
      B.value ? (p(), R("div", Ya, [
        w("span", null, G(a.value), 1)
      ])) : $("", !0),
      O.value ? (p(), R("div", Ja, [
        w("span", Xa, [
          w("span", null, G(u.value), 1)
        ])
      ])) : $("", !0),
      D.value ? (p(), R("div", Za, [
        w("span", null, G(f.value), 1)
      ])) : $("", !0)
    ], 8, Ga));
  }
}), ti = /* @__PURE__ */ ct(ei, [["__scopeId", "data-v-5badd3b4"]]), ni = {
  class: "blockquote",
  dir: "auto"
}, oi = {
  key: 0,
  class: "markdown-renderer"
}, vn = /* @__PURE__ */ Z({
  __name: "BlockquoteBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    children: {},
    version: {}
  },
  setup(t) {
    const n = t, e = v(() => n.mode === "edit");
    return (o, l) => (p(), R("blockquote", ni, [
      e.value ? (p(), R("div", oi, [
        (p(), X(ce(ot), {
          children_slot: t.children,
          key: "BlockChildren-1"
        }, null, 8, ["children_slot"]))
      ])) : $("", !0),
      e.value ? $("", !0) : (p(), X(ce(ot), {
        children_slot: t.children,
        key: "BlockChildren-2"
      }, null, 8, ["children_slot"]))
    ]));
  }
}), li = ["data-accent"];
xe("Fence", Co);
xe("MathBlock", Hn);
xe("Mermaid", Pn);
function Dt(t) {
  return () => Ue(ft(t.children), !0);
}
function ai(t) {
  return t.children.map((n) => {
    const e = Tt(n.attrs, "checked") != null;
    return {
      id: n.id,
      task: e,
      checked: wn(n.attrs, "checked", !1),
      cls: "list-item" + (e ? " task-item" : ""),
      children_slot: () => Ue(ft(n.children), !0)
    };
  });
}
function ht(t, n) {
  return (e, o) => ie(t, {
    mode: "edit",
    node: e,
    ctx: { engine: o.engine, blockId: o.blockId, readonly: o.readonly },
    final: !0,
    version: o.version ?? 0,
    ...n(e, o)
  });
}
xe("Callout", Qt, {
  edit: ht(Qt, (t, n) => ({ children: n.children ?? Dt(t) }))
});
xe("Blockquote", vn, {
  edit: ht(vn, (t, n) => ({ children: n.children ?? Dt(t) }))
});
xe("ListBlock", zt, {
  // items is FLAT DATA (the widget v-fors the array itself), unlike the
  // children closure — call the injection at slot-call time (the assembly's
  // render closure re-runs per repaint, so the timing matches the retired
  // hand-rolled dispatch)
  edit: ht(zt, (t, n) => ({ items: n.items ? n.items() : ai(t) }))
});
xe("Details", It, {
  edit: ht(It, (t, n) => ({ children: n.children ?? Dt(t) }))
});
function Kn(t) {
  return t === "center" ? "text-center" : t === "right" ? "text-right" : "text-left";
}
function mn(t) {
  return { id: t.id, text: pe(t), cls: Kn(ut(t.attrs, "align", "left")) };
}
function pn(t) {
  return {
    id: t.id,
    cls: Kn(ut(t.attrs, "align", "left")),
    children_slot: () => Ue(ft(t.children), !0)
  };
}
xe("Table", gt, {
  edit: (t, n) => {
    var o;
    const e = t.children;
    return ie(gt, {
      mode: "edit",
      controller: new ra(n.engine, n.blockId),
      blockId: n.blockId,
      readonly: n.readonly,
      final: !0,
      header_cells: (((o = e[0]) == null ? void 0 : o.children) ?? []).map(mn),
      body_rows: e.slice(1).map((l) => ({ id: l.id, cells: l.children.map(mn) })),
      // filler for the generated required-prop checks (the 033 ctx:null
      // idiom) — the edit face reads none
      columns: [],
      rows: []
    });
  },
  stream: So,
  view: (t, n) => {
    var o;
    const e = t.children;
    return ie(gt, {
      mode: "view",
      controller: null,
      blockId: "",
      readonly: !1,
      final: n,
      header_cells: (((o = e[0]) == null ? void 0 : o.children) ?? []).map(pn),
      body_rows: e.slice(1).map((l) => ({ id: l.id, cells: l.children.map(pn) })),
      columns: [],
      rows: []
    });
  }
});
function ii(t, n) {
  const e = [];
  return (n == null ? void 0 : n.type) === "details" && (e.push({ key: "summary", value: Se.Str(String(n.text ?? "")) }), (n == null ? void 0 : n.loading) === !0 && e.push({ key: "open", value: Se.Bool(!0) })), {
    id: "nv",
    kind: t,
    attrs: e,
    children: [],
    inlines: [],
    source: { start: 0, end: 0 }
  };
}
Fe(
  "Details",
  // plan 035 T6: the family widget replaces the retired DetailsNodeView —
  // same view face, plus the marker verb riding the live host window's
  // engine (the preview-side toggle writes `open` back through the model,
  // the host-protocol contract) and the body through the BlockChildren
  // closure instead of the node-view injection key.
  (t) => {
    const n = ia(), e = mo(t.node) ?? ii(x.Details, t.node);
    return ie(It, {
      mode: "view",
      node: e,
      final: t.final ?? !0,
      ctx: n ? { engine: n.engine, blockId: e.id, readonly: !0 } : null,
      children: po(Io(), () => Ue(t.node.children ?? [], !0)),
      version: 0
    });
  }
);
Fe("MathBlock", dt(Hn));
Fe("Mermaid", dt(Pn));
Fe("Query", dt(Va));
Fe("Embed", dt(ti));
const di = /* @__PURE__ */ Z({
  __name: "EngineEditor",
  props: {
    content: {},
    modelValue: {},
    placeholder: {},
    extraSlashItems: {},
    streaming: { type: Boolean },
    runQuery: { type: Function },
    loadBlock: { type: Function },
    darkMode: { type: Boolean },
    accent: {}
  },
  emits: ["update", "update:modelValue", "save", "focusblock", "open-wiki-link"],
  setup(t, { expose: n, emit: e }) {
    const o = t, l = ["indigo", "coral", "ocean", "sage", "amber"], a = v(
      () => l.includes(o.accent) ? o.accent : "indigo"
    ), i = b(!1), r = v(() => o.darkMode || i.value);
    fe(() => {
      var s;
      if (typeof document < "u") {
        const m = () => {
          var _;
          i.value = !!(document.documentElement.classList.contains("dark") || document.documentElement.getAttribute("data-theme") === "dark" || (_ = g.value) != null && _.closest('.dark, [data-theme="dark"]'));
        };
        m();
        const k = new MutationObserver(m);
        k.observe(document.documentElement, { attributes: !0, attributeFilter: ["class", "data-theme"] }), (s = g.value) != null && s.parentElement && k.observe(g.value.parentElement, { attributes: !0, attributeFilter: ["class", "data-theme"] }), Me(() => k.disconnect());
      }
    });
    const u = e;
    let c = null, f = null;
    const d = 250;
    function B(s) {
      const m = s ?? "\0null";
      m !== c && (c = m, re(() => {
        const k = s ? ue().find((_) => _.id === s) : void 0;
        u("focusblock", s && k ? { id: s, height: k.height } : null);
      }));
    }
    function D(s) {
      if (C) {
        C = !1, c = null;
        return;
      }
      if (f != null && (clearTimeout(f), f = null), !s) {
        B(null);
        return;
      }
      const m = E.selection.anchor.blockId || null;
      m && (f = setTimeout(() => {
        f = null, B(m);
      }, d));
    }
    const O = (s, m) => u("open-wiki-link", s, m);
    Gt(O), Me(() => {
      Bo() === O && Gt(null);
    });
    let h = !1;
    Ae(
      () => [o.runQuery, o.loadBlock],
      () => {
        o.runQuery != null || o.loadBlock != null ? (wt({ runQuery: o.runQuery, loadBlock: o.loadBlock }), h = !0) : h && (wt({}), h = !1);
      },
      { immediate: !0 }
    ), Me(() => {
      if (!h) return;
      const s = Ot();
      s.runQuery === o.runQuery && s.loadBlock === o.loadBlock && wt({});
    });
    const g = b(null), A = b(null);
    function j(s) {
      return bn(s ?? "", !0);
    }
    const E = new nl(j(o.modelValue ?? o.content ?? "")), te = fl(E), oe = oa({ extraSlashItems: o.extraSlashItems });
    let z = Qe(E.doc, !0);
    E.onChange(() => {
      S.value++;
      const s = J();
      D(s);
    });
    function J() {
      const s = Qe(E.doc, !0);
      return s === z ? !1 : (z = s, u("update", s), u("update:modelValue", s), !0);
    }
    let C = !1;
    Ae(
      () => o.modelValue ?? o.content,
      (s) => {
        s != null && s !== Qe(E.doc, !0) && (C = !0, E.replaceDoc(j(s)));
      }
    );
    function I() {
      var V;
      const s = typeof window > "u" ? null : window.getSelection();
      if (!s || s.rangeCount === 0) return;
      const m = s.getRangeAt(0);
      if (m.collapsed) return;
      const k = m.startContainer, _ = k.nodeType === 3 ? k.parentElement : k, N = _ == null ? void 0 : _.closest(".autodown-block-host");
      if (!N || !((V = g.value) != null && V.contains(N))) return;
      const W = N.dataset.blockId;
      if (!W) return;
      const P = il(N, W);
      !P || P.lo === P.hi || E.selection.anchor.blockId === W && E.selection.anchor.offset === P.lo && E.selection.head.offset === P.hi || E.select(new mt(new ze(W, P.lo), new ze(W, P.hi)));
    }
    fe(() => document.addEventListener("selectionchange", I)), Me(() => document.removeEventListener("selectionchange", I)), Me(() => {
      f != null && clearTimeout(f);
    });
    const S = b(0), y = b(0), T = Z({
      name: "AssemblyView",
      props: { render: { type: Function, required: !0 } },
      setup(s) {
        return () => s.render();
      }
    }), ee = v(() => {
      S.value;
      const s = E.selection.anchor.blockId, m = { path: go(E.doc, s), focusedId: s, counter: { n: 0 } };
      return E.doc.children.map((k) => M(k, m, !0));
    });
    function he(s) {
      la({ engine: E, adapter: te });
      try {
        return Ue(ft([s]), !0)[0] ?? ie("div", { class: "unknown-node" }, "");
      } finally {
        aa();
      }
    }
    function le(s, m, k, _, N = !0) {
      return ie(
        "div",
        {
          class: "node-slot",
          "data-node-index": String(_.n++),
          "data-node-type": x[s.kind],
          "data-block-id": s.id,
          // the innermost addressable slot wins — an expanded container's outer
          // chrome must not re-handle the bubbled click (it would resolve the
          // whole container back to its first leaf)
          onClick: (W) => {
            W.stopPropagation();
            const P = wo(s, W.currentTarget, W);
            P.anchor && yo(W, P.targetId, P.anchor, P.cellId), L(s.id, P.targetId);
          }
        },
        [
          ie("div", { class: "node-content" }, [m]),
          ...k && N ? [ie("div", { class: "autodown-block-boundary", "data-boundary-for": s.id })] : []
        ]
      );
    }
    function ke(s, m) {
      return s.children.map((k) => {
        const _ = Tt(k.attrs, "checked") != null;
        return {
          id: k.id,
          task: _,
          checked: wn(k.attrs, "checked", !1),
          cls: "list-item" + (_ ? " task-item" : ""),
          children_slot: () => k.children.map((N) => ve(N, m))
        };
      });
    }
    function ve(s, m) {
      return s.id === m.focusedId || m.path.has(s.id) ? Ke(M(s, m, !1)) : le(s, he(s), !1, m.counter);
    }
    function Ke(s) {
      return ie(s.view, s.props);
    }
    function M(s, m, k) {
      if (s.id === m.focusedId) {
        const _ = Vt(x[s.kind]);
        if (_)
          return {
            id: s.id,
            view: T,
            props: {
              render: () => le(s, _(s, { engine: E, blockId: s.id, readonly: o.streaming === !0 }), k, m.counter, !1),
              key: `edit:${s.id}:${y.value}`
            }
          };
        if (ko(s)) {
          const N = se(s.id), W = s.kind === x.Heading ? Xn(s.attrs, "level", 1) : void 0;
          return {
            id: s.id,
            view: T,
            props: {
              render: () => le(
                s,
                ie(Al, {
                  controller: N,
                  // flat chrome data (plan 034 D4): the widget derives tag/cls from
                  // blockKind/level itself (the host-face computation is absorbed);
                  // initial_html is the mount-once rich snapshot, evaluated here —
                  // the engine is not Vue-reactive, the snapshot never invalidates.
                  blockId: N.id,
                  blockKind: x[s.kind],
                  level: W ?? 0,
                  initial_html: yn(N.inlines),
                  // The face lives in the key: a kind/level flip mid-typing (input
                  // rules) must REMOUNT the host. <component :is> would swap the
                  // DOM element under the caret without re-running onMounted —
                  // focus lands nowhere and every post-flip keystroke is lost.
                  // The remount re-focuses at end (plan 029; rules match only a
                  // whole-block marker, so the caret IS at end on every flip).
                  key: `host:${s.id}:${x[s.kind]}:${W ?? ""}:${y.value}`
                }),
                k,
                m.counter,
                !1
              )
            }
          };
        }
      }
      if (m.path.has(s.id) && U(s)) {
        const _ = Vt(x[s.kind]);
        if (_)
          return {
            id: s.id,
            view: T,
            props: {
              render: () => le(s, _(s, {
                engine: E,
                blockId: s.id,
                readonly: o.streaming === !0,
                children: () => s.children.map((N) => ve(N, m)),
                items: () => ke(s, m),
                version: S.value
              }), k, m.counter)
            }
          };
      }
      return {
        id: s.id,
        view: T,
        props: { render: () => le(s, he(s), k, m.counter) }
      };
    }
    function U(s) {
      return s.children.length > 0 && (s.kind === x.ListBlock || s.kind === x.Blockquote || s.kind === x.Callout || s.kind === x.Details);
    }
    const K = /* @__PURE__ */ new Map();
    function se(s) {
      let m = K.get(s);
      return m || (m = new bo(E, s), K.set(s, m)), m;
    }
    function ue() {
      const s = [], m = A.value;
      return m && Array.from(m.querySelectorAll("[data-block-id]")).forEach((_, N) => {
        s.push({
          id: _.dataset.blockId ?? "",
          index: N,
          pos: N,
          el: _,
          top: _.offsetTop,
          height: _.offsetHeight
        });
      }), s;
    }
    function L(s, m) {
      const k = Y(E.doc, s);
      if (!k) return;
      const _ = m && m !== s ? Y(E.doc, m) : null, N = (_ && pt(_) === _ ? _ : null) ?? pt(k) ?? k, W = new ze(N.id, 0);
      E.select(new mt(W, W)), S.value++;
    }
    function H(s) {
      const m = ol(s);
      if (m) {
        if (Array.from(K.values()).some((k) => k.composition.composing)) return;
        s.preventDefault(), ll(E, K.values(), m) && y.value++;
        return;
      }
      if (s.ctrlKey && s.key === "End") {
        s.preventDefault();
        const k = _o(E.doc);
        k && L(k.id);
      }
    }
    function F() {
      const s = pt(E.doc);
      if (s) {
        const m = new ze(s.id, 0);
        E.select(new mt(m, m));
      }
    }
    F();
    function Q() {
      u("save", Qe(E.doc, !0));
    }
    return n({ getBlockMap: ue, handleSave: Q }), (s, m) => (p(), R("div", {
      ref_key: "root",
      ref: g,
      class: Te(["autodown-editor", { "is-dark": r.value }]),
      "data-accent": a.value
    }, [
      w("div", {
        ref_key: "wrapper",
        ref: A,
        class: "autodown-editor-content-wrapper"
      }, [
        w("div", {
          class: "autodown-editor-content",
          "data-engine-editor": "",
          tabindex: "-1",
          onKeydown: H
        }, [
          vt(ce(ta), {
            editor: ce(te),
            items: ce(oe)
          }, null, 8, ["editor", "items"]),
          vt(Pl, { editor: ce(te) }, null, 8, ["editor"]),
          vt(Gl, { editor: ce(te) }, null, 8, ["editor"]),
          (p(!0), R(_e, null, Pe(ee.value, (k) => (p(), X(ge(k.view), Gn({
            key: k.id
          }, { ref_for: !0 }, k.props), null, 16))), 128))
        ], 32)
      ], 512),
      w("div", { class: "autodown-editor-actions" }, [
        w("button", {
          type: "button",
          class: "autodown-editor-save",
          onClick: Q
        }, "Save")
      ])
    ], 10, li));
  }
}), fi = "block-";
function hi(t) {
  const e = Array.from((t ?? document).querySelectorAll("[data-block-id]")), o = /* @__PURE__ */ new Set(), l = [];
  for (const a of e) {
    const i = a.dataset.blockId ?? "";
    i !== "" && o.has(i) || (i !== "" && o.add(i), l.push({ id: i, index: l.length, pos: l.length, el: a, top: a.offsetTop, height: a.offsetHeight }));
  }
  return l;
}
export {
  fi as B,
  nl as E,
  di as _,
  Ot as a,
  fl as c,
  hi as g,
  wt as s
};
