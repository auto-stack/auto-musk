import { ref as w, getCurrentInstance as Wn, onBeforeUnmount as Le, defineComponent as J, computed as m, onMounted as me, openBlock as g, createBlock as Y, resolveDynamicComponent as ve, withModifiers as Oe, normalizeClass as Te, h as le, unref as re, withCtx as Ye, createElementBlock as x, Fragment as ye, renderList as Pe, createCommentVNode as $, onUnmounted as yt, normalizeStyle as gn, createElementVNode as b, withDirectives as st, withKeys as Ve, vModelText as Bt, toDisplayString as V, nextTick as ae, watch as Ee, inject as Vn, provide as Qn, createTextVNode as Xe, vShow as zn, createVNode as vt, mergeProps as Gn } from "vue";
import { a0 as Ze, O as jn, J as Yn, ab as Ft, T as kn, M as N, B as T, f as j, a1 as et, u as Jn, a2 as Ut, i as Tt, Q as qe, V as Se, r as we, b as he, $ as Je, a7 as wn, d as ut, ac as Qe, k as Xn, S as mt, K as ze, h as bn } from "./parser-BfX0E-c9.js";
import { H as Zn, I as eo, J as _t, K as Kt, L as to, M as no, N as yn, O as oo, a as Wt, c as lo, t as ao, b as io, P as ro, Q as Ct, R as _n, S as tt, T as Cn, U as xt, V as Sn, W as nt, X as Rt, Y as In, _ as ct, Z as Bn, $ as so, a0 as uo, a1 as co, a2 as fo, a3 as ho, a4 as ot, a5 as vo, y as xe, l as Fe, a6 as mo, a7 as po, w as dt, a8 as go, p as Vt, a9 as ko, B as wo, F as Ue, aa as ft, ab as bo, ac as yo, ad as pt, ae as _o, af as Co, ag as Qt, ah as zt, G as So, ai as gt, aj as Io, ak as Gt, al as Bo } from "./render-node-Di_WXNK8.js";
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
      const s = l.value, u = n.value;
      o.ops[0] = jn.InsertText(new Yn(s.pos, s.text + u.text));
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
  const s = l.filter((c) => al(o, c.node.raw));
  if (s.length === 0) return -1;
  const u = s[s.length - 1];
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
  const o = Mt(t), a = t.ownerDocument.createRange(), i = Yt(o, Math.min(n, e)), s = Yt(o, Math.max(n, e));
  return !i || !s ? (a.selectNodeContents(t), a) : (a.setStart(i.raw, i.inner), a.setEnd(s.raw, s.inner), a);
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
  [N.Strong]: "strong",
  [N.Em]: "em",
  [N.Del]: "del",
  [N.Underline]: "u",
  [N.Code]: "code"
};
function Ge(t) {
  const n = typeof window > "u" ? null : window.getSelection();
  if (!n || n.rangeCount === 0) return null;
  const e = n.getRangeAt(0);
  return e.collapsed || !t.contains(e.startContainer) || !t.contains(e.endContainer) ? null : e;
}
function ue(t, n, e) {
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
  const o = ue(t, n.startContainer, e);
  return o != null && o === ue(t, n.endContainer, e) && o.contains(n.commonAncestorContainer);
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
const ee = {
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
    if (t === N.Link) {
      const l = ue(n, e.startContainer, ["a"]);
      return l != null && l === ue(n, e.endContainer, ["a"]);
    }
    const o = Ne[t];
    return o == null ? !1 : Xt(n, e, lt[o]);
  },
  applyMark(t, n) {
    const e = Be;
    if (!e) return !1;
    const o = Ge(e);
    if (!o) return !1;
    if (t === N.Link) {
      if (n == null || n === "") return ee.removeMark(N.Link);
      const i = ue(e, o.startContainer, ["a"]);
      if (i && i === ue(e, o.endContainer, ["a"]))
        return i.setAttribute("href", n), i.setAttribute("contenteditable", "false"), i.setAttribute("data-autodown-link", ""), !0;
      const s = e.ownerDocument.createElement("a");
      return s.setAttribute("href", n), s.setAttribute("contenteditable", "false"), s.setAttribute("data-autodown-link", ""), Zt(o, s), !0;
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
    if (t === N.Link) {
      const a = ue(n, e.startContainer, ["a"]);
      if (a && a === ue(n, e.endContainer, ["a"])) {
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
    return Xt(n, e, l) ? (An(ue(n, e.startContainer, l)), !0) : !1;
  }
};
function Ln() {
  const t = Be, n = typeof window > "u" ? null : window.getSelection();
  if (!t || !n || n.rangeCount === 0) return [];
  const e = n.getRangeAt(0);
  if (!t.contains(e.startContainer)) return [];
  const o = [];
  for (const l of [N.Strong, N.Em, N.Underline, N.Del, N.Code]) {
    const a = Ne[l];
    a != null && ue(t, e.startContainer, lt[a] ?? [a]) && o.push(l);
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
  const a = ue(n, o.startContainer, lt[l] ?? [l]);
  return a ? (An(a), !0) : !1;
}
function Ie(t, n) {
  return t === ee && t.getSelection() == null ? sl(n) : t.isActive(n) ? t.removeMark(n) : t.applyMark(n);
}
const ul = {
  setParagraph: T.Paragraph,
  setMathBlock: T.MathBlock,
  setMermaidBlock: T.Mermaid,
  setHorizontalRule: T.ThematicBreak,
  toggleBulletList: T.ListItem,
  toggleOrderedList: T.ListItem,
  toggleBlockquote: T.Blockquote
}, cl = {
  bold: N.Strong,
  strong: N.Strong,
  italic: N.Em,
  em: N.Em,
  strike: N.Del,
  strikethrough: N.Del,
  underline: N.Underline,
  code: N.Code,
  link: N.Link
}, en = {
  table: T.Table,
  codeBlock: T.Fence,
  fence: T.Fence,
  blockquote: T.Blockquote,
  bulletList: T.ListBlock,
  orderedList: T.ListBlock,
  listItem: T.ListItem,
  heading: T.Heading,
  details: T.Details,
  callout: T.Callout,
  mathBlock: T.MathBlock,
  mermaid: T.Mermaid,
  queryBlock: T.QueryBlock,
  blockEmbed: T.BlockEmbed
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
  const n = w(0), e = /* @__PURE__ */ new Map();
  let o = t.selection;
  const l = (i) => {
    const s = e.get(i);
    if (s)
      for (const u of [...s]) u();
  };
  return t.onChange((i) => {
    n.value++, dl(i.selection, o) || (o = i.selection, l("selectionUpdate"));
  }), {
    storage: { "slash-command": { query: "", range: null, handled: !1 } },
    isEditable: !0,
    on: (i, s) => {
      let u = e.get(i);
      u || (u = /* @__PURE__ */ new Set(), e.set(i, u)), u.add(s);
    },
    off: (i, s) => {
      var u;
      (u = e.get(i)) == null || u.delete(s);
    },
    isActive: (i) => {
      n.value;
      const s = cl[i];
      if (s != null) {
        const f = Ln();
        return f.length > 0 ? f.includes(s) : Jn(Zn(t, t.selection), s);
      }
      const u = en[i];
      if (u == null) return !1;
      const c = /* @__PURE__ */ new Set();
      return St(t.doc, t.selection.anchor.blockId, c), c.has(u);
    },
    getAttributes: (i) => {
      n.value;
      const s = en[i];
      if (s == null) return {};
      const u = j(t.doc, t.selection.anchor.blockId);
      if (u && u.kind === s) return tn(u.attrs);
      if (u) {
        const c = /* @__PURE__ */ new Set();
        if (St(t.doc, u.id, c) && c.has(s)) {
          let f = u;
          for (; f; ) {
            if (f.kind === s) return tn(f.attrs);
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
        const s = document.querySelector(".autodown-editor-content");
        if (!s) return null;
        const u = t.selection.anchor.blockId;
        for (const c of s.querySelectorAll("[data-block-id]"))
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
        const s = t.selection.anchor.blockId, u = En() ?? document.querySelector(`.autodown-block-host[data-block-id="${s}"]`);
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
  const n = t.selection.anchor.blockId, e = j(t.doc, n);
  if (!e) return null;
  if (e.kind === T.Table) return { tableId: n, rowId: null, rowIdx: null, colIdx: null };
  if (e.kind !== T.TableCell) return null;
  const o = et(t.doc, n);
  if (!o || o.kind !== T.TableRow) return null;
  const l = et(t.doc, o.id);
  return !l || l.kind !== T.Table ? null : { tableId: l.id, rowId: o.id, rowIdx: Ut(l, o.id), colIdx: Ut(o, n) };
}
function De(t, n) {
  var e;
  return ((e = j(t.doc, n)) == null ? void 0 : e.children) ?? [];
}
function hl(t) {
  const n = [], e = {
    focus: () => e,
    run: () => (t.applyTree((l) => n.reduce((a, i) => i(a), l)), !j(t.doc, t.selection.anchor.blockId) && t.doc.children[0] && t.select(Ze(t.doc.children[0].id, 0)), !0),
    setHeading: (l) => (n.push((a) => $e(a, t, T.Heading, [{ key: "level", value: Se.Int((l == null ? void 0 : l.level) ?? 1) }])), e),
    insertContent: (l) => {
      const a = String(l ?? "");
      return n.push((i) => {
        if (!a.includes(`
`)) return wt(i, t, a);
        const u = wn(a, !0).children;
        if (u.length === 0) return i;
        const c = Me(t), f = j(i, c);
        if (!f) return i;
        const d = i.children, B = d.findIndex((v) => v.id === c), A = he(f) === "" ? [] : [f], O = [...d.slice(0, B), ...A, ...u, ...d.slice(B + 1)];
        return kn(i, O);
      }), e;
    },
    deleteRange: (l) => (n.push((a) => {
      const i = Me(t), s = j(a, i);
      if (!s) return a;
      const u = he(s), c = Math.max(0, Math.min(l.from, u.length)), f = Math.max(c, Math.min(l.to, u.length));
      return we(a, i, [Je(i, s.kind, u.slice(0, c) + u.slice(f))]);
    }), e),
    insertTable: (l) => (n.push((a) => wt(a, t, `| a | b |
| --- | --- |
|  |  |
|  |  |`)), e),
    setImage: (l) => {
      const a = String((l == null ? void 0 : l.src) ?? ""), i = String((l == null ? void 0 : l.alt) ?? "");
      return n.push((s) => wt(s, t, `![${i}](${a})`)), e;
    },
    // table verbs (plan 026 P0T3): forward to the commands.ts table transforms,
    // resolved against the focused cell (table-level focus takes the
    // table-ends defaults); tree-level so a chain stays ONE undo.
    addRowAfter: () => {
      var s;
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = l.rowId ?? ((s = a[a.length - 1]) == null ? void 0 : s.id) ?? null;
      return n.push((u) => Kt(u, l.tableId, i)), e;
    },
    addRowBefore: () => {
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = l.rowIdx != null && l.rowIdx > 0 ? a[l.rowIdx - 1].id : null;
      return n.push((s) => Kt(s, l.tableId, i)), e;
    },
    deleteRow: () => {
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId);
      if (a.length <= 1) return e;
      const i = l.rowId ?? a[a.length - 1].id;
      return n.push((s) => we(s, i, [])), e;
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
      return n.push((s) => _t(s, l.tableId, l.colIdx != null ? l.colIdx + 1 : a)), e;
    },
    deleteColumn: () => {
      var s;
      const l = Re(t);
      if (!l) return e;
      const a = De(t, l.tableId), i = Math.max(0, (((s = a[0]) == null ? void 0 : s.children.length) ?? 1) - 1);
      return n.push((u) => eo(u, l.tableId, l.colIdx ?? i)), e;
    },
    deleteTable: () => {
      const l = Re(t);
      return l && n.push((a) => we(a, l.tableId, [])), e;
    },
    // code language channel (plan 026 P0T3): setBlockAttrs on the focused
    // Fence (023's IAL ruling); converts the kind when not a Fence yet.
    setCodeBlockLanguage: (l) => (n.push((a) => $e(a, t, T.Fence, [{ key: "language", value: Se.Str(String(l ?? "")) }])), e),
    setCodeBlock: (l) => (l == null ? void 0 : l.language) != null ? e.setCodeBlockLanguage(l.language) : (n.push((a) => $e(a, t, T.Fence)), e),
    // slash manifest's Details template carries { summary } (plan 026 P2T3):
    // kind conversion + summary attr so the mounted node-view shows it.
    // Converting an inline leaf moves its text into a child paragraph — a
    // Details renders children, inlines would serialize away (data loss).
    setDetails: (l) => (n.push((a) => {
      const i = Me(t), s = j(a, i);
      if (!s) return a;
      const u = s.children.length > 0 ? s.children : he(s).length > 0 ? [Je(`${i}-p`, T.Paragraph, he(s))] : [];
      let c = { ...s, kind: T.Details, children: u };
      return (l == null ? void 0 : l.summary) != null && (c = { ...c, attrs: qe(c.attrs, "summary", Se.Str(String(l.summary))) }), we(a, i, [c]);
    }), e),
    // slash Callout template carries { type, title } (plan 030 T7): same
    // conversion shape as setDetails — before this the kind-only KIND_COMMANDS
    // path silently dropped both attrs (the lost-title roundtrip break).
    setCallout: (l) => (n.push((a) => {
      const i = Me(t), s = j(a, i);
      if (!s) return a;
      const u = s.children.length > 0 ? s.children : he(s).length > 0 ? [Je(`${i}-p`, T.Paragraph, he(s))] : [];
      let c = { ...s, kind: T.Callout, children: u };
      return (l == null ? void 0 : l.type) != null && (c = { ...c, attrs: qe(c.attrs, "type", Se.Str(String(l.type))) }), (l == null ? void 0 : l.title) != null && (c = { ...c, attrs: qe(c.attrs, "title", Se.Str(String(l.title))) }), we(a, i, [c]);
    }), e),
    // task list (plan 030 T7): a real verb distinct from toggleBulletList —
    // the focused ListItem (a caret usually sits on its child paragraph, so
    // resolve the ListItem ancestor first — the list-commands 选中定位
    // discipline) gains/loses the `checked` attr (task ⇄ plain bullet);
    // outside a list it converts like the bullet verb.
    toggleTaskList: () => (n.push((l) => {
      const a = Me(t);
      let i = j(l, a);
      for (; i != null && i.kind !== T.ListItem; )
        i = et(l, i.id);
      if (i == null) return $e(l, t, T.ListItem);
      const u = Tt(i.attrs, "checked") != null ? i.attrs.filter((c) => c.key !== "checked") : qe(i.attrs, "checked", Se.Bool(!1));
      return we(l, i.id, [{ ...i, attrs: u }]);
    }), e),
    // inline mark toggles (plan 024 P3T1; adapter-routed plan 036 T3): wrap
    // the FOCUSED host's live DOM through the SelectionAdapter — the model
    // catches up on the blur writeback. No focused host → no-op.
    toggleBold: () => (Ie(ee, N.Strong), e),
    toggleItalic: () => (Ie(ee, N.Em), e),
    toggleStrike: () => (Ie(ee, N.Del), e),
    toggleCode: () => (Ie(ee, N.Code), e),
    // underline (plan 028 P2T2): same DOM-wrap protocol as the others —
    // the model catches up on the blur writeback (u → Mark.Underline)
    toggleUnderline: () => (Ie(ee, N.Underline), e),
    setLink: (l) => {
      const a = String((l == null ? void 0 : l.href) ?? "");
      return a ? ee.applyMark(N.Link, a) : ee.removeMark(N.Link), e;
    },
    unsetLink: () => (ee.removeMark(N.Link), e)
  }, o = e;
  for (const [l, a] of Object.entries(ul))
    o[l] = () => (n.push((i) => $e(i, t, a)), e);
  return e;
}
function Me(t) {
  var n;
  return t.selection.anchor.blockId || ((n = t.doc.children[0]) == null ? void 0 : n.id) || "";
}
function $e(t, n, e, o) {
  const l = Me(n), a = j(t, l);
  if (!a) return t;
  let i = { ...a, kind: e };
  if (o)
    for (const s of o) i = { ...i, attrs: qe(i.attrs, s.key, s.value) };
  return we(t, l, [i]);
}
function wt(t, n, e) {
  const o = Me(n), l = j(t, o);
  if (!l) return t;
  const a = he(l) + e;
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
  (!l || !no(o, l)) && Lt(o), Le(() => {
    at.delete(o), En() === o && At(null);
  });
}
function Lt(t) {
  const n = document.createRange();
  n.selectNodeContents(t), n.collapse(!1);
  const e = window.getSelection();
  e == null || e.removeAllRanges(), e == null || e.addRange(n);
}
function wl(t, n) {
  const e = t.ownerDocument.createTreeWalker(t, NodeFilter.SHOW_TEXT);
  let o = Math.max(0, n), l = null, a = !1;
  for (; l = e.nextNode(); ) {
    const i = l.data.length;
    if (o <= i) {
      const s = document.createRange();
      if (o === i) {
        const c = l.parentElement;
        if (c && c !== t) {
          const f = t.ownerDocument.createTextNode(bl);
          c.after(f), s.setStart(f, 1), s.collapse(!0);
          const d = window.getSelection();
          d == null || d.removeAllRanges(), d == null || d.addRange(s), a = !0;
          break;
        }
      }
      s.setStart(l, o), s.collapse(!0);
      const u = window.getSelection();
      u == null || u.removeAllRanges(), u == null || u.addRange(s), a = !0;
      break;
    }
    o -= i;
  }
  a || Lt(t);
}
const bl = "​";
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
    o == null ? Lt(t) : wl(t, o);
  }
  typeof document < "u" && ml(vl(n.text, He(t)), n.id, He(t));
}
function Il(t, n) {
  if (n.composition.composing) return;
  const e = t.currentTarget ?? t.target;
  if (t.ctrlKey || t.metaKey) {
    const o = t.key.toLowerCase();
    if (o === "b") {
      t.preventDefault(), Ie(ee, N.Strong);
      return;
    }
    if (o === "i") {
      t.preventDefault(), Ie(ee, N.Em);
      return;
    }
    if (o === "u") {
      t.preventDefault(), Ie(ee, N.Underline);
      return;
    }
    if (o === "k") {
      t.preventDefault();
      const l = window.prompt("Enter URL");
      l && ee.applyMark(N.Link, l);
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
const Al = /* @__PURE__ */ J({
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
    const e = t, o = m(() => pl(e.blockKind, e.level)), l = m(() => gl(e.blockKind, e.level)), a = n;
    function i(v) {
      El(v.target, e.controller), a("Blur", v);
    }
    function s(v) {
      a("ClickStop", v);
    }
    function u(v) {
      Rl(v.target, e.controller), a("CompositionEnd", v);
    }
    function c(v) {
      Tl(v.target, e.controller), a("CompositionStart", v);
    }
    function f(v) {
      xl(v, e.controller), a("CompositionUpdate", v);
    }
    function d(v) {
      Ml(v.target, e.controller), a("Focus", v);
    }
    function B(v) {
      Sl(v.target, e.controller), a("Input", v);
    }
    function A(v) {
      Il(v, e.controller), a("Keydown", v);
    }
    function O(v) {
      Bl(v, e.controller), a("Paste", v);
    }
    return me(() => {
      kl(e.initial_html, e.blockId);
    }), (v, k) => (g(), Y(ve(o.value), {
      class: Te(l.value),
      contenteditable: !0,
      "data-block-id": t.blockId,
      "data-node-type": t.blockKind,
      dir: "auto",
      spellcheck: "false",
      onBlur: k[0] || (k[0] = (p) => i(p)),
      onClick: k[1] || (k[1] = Oe((p) => s(p), ["stop"])),
      onCompositionend: k[2] || (k[2] = (p) => u(p)),
      onCompositionstart: k[3] || (k[3] = (p) => c(p)),
      onCompositionupdate: k[4] || (k[4] = (p) => f(p)),
      onFocus: k[5] || (k[5] = (p) => d(p)),
      onInput: k[6] || (k[6] = (p) => B(p)),
      onKeydown: k[7] || (k[7] = (p) => A(p)),
      onPaste: k[8] || (k[8] = (p) => O(p))
    }, null, 40, ["class", "data-block-id", "data-node-type"]));
  }
});
function Ll(t) {
  return t.startsWith("bottom") ? { vertical: "bottom", horizontal: t.endsWith("end") ? "right" : "left" } : { vertical: "top", horizontal: t.endsWith("end") ? "right" : "left" };
}
function be(t, n, e, o, l = "bottom", a = 8, i = "left") {
  const { vertical: s, horizontal: u } = Ll(l);
  let c;
  if (s === "bottom") {
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
function Ae(t) {
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
const Nl = J({
  name: "EngineBubbleMenu",
  props: {
    editor: { type: Object, default: null },
    options: { type: Object, default: null },
    shouldShow: { type: Function, default: null }
  },
  setup(t, { slots: n }) {
    const e = w(!1), o = w("0px"), l = w("0px"), a = w(null);
    let i = null;
    const s = () => {
      var K, G, _, I, C, y;
      const d = (K = t.editor) == null ? void 0 : K.__engine;
      if (!d) {
        e.value = !1;
        return;
      }
      (_ = (G = t.editor) == null ? void 0 : G.__bump) == null || _.call(G);
      const B = typeof window > "u" ? null : window.getSelection(), A = B && B.rangeCount > 0 ? B.getRangeAt(0) : null;
      if (!A) {
        e.value = !1;
        return;
      }
      let O;
      A.collapsed && (O = Ln());
      const v = { editor: t.editor, state: { ...ql(d), marks: O } };
      if (e.value = t.shouldShow ? !!t.shouldShow(v) : !1, !e.value) return;
      const k = (I = A.startContainer.nodeType === 3 ? A.startContainer.parentElement : A.startContainer) == null ? void 0 : I.closest(".autodown-block-host"), p = k == null ? void 0 : k.closest(".autodown-editor");
      if (!k || !p || !p.contains(k)) {
        e.value = !1;
        return;
      }
      const U = A.getBoundingClientRect(), X = p.getBoundingClientRect(), ie = {
        top: U.top - X.top,
        left: U.left - X.left,
        bottom: U.bottom - X.top,
        right: U.right - X.left,
        width: U.width,
        height: U.height
      }, te = be(
        ie,
        ((C = a.value) == null ? void 0 : C.offsetWidth) ?? 0,
        ((y = a.value) == null ? void 0 : y.offsetHeight) ?? 0,
        { width: p.clientWidth, height: p.clientHeight },
        "top"
      );
      l.value = `${te.left}px`, o.value = `${te.top}px`;
    }, u = (d) => {
      var B, A;
      e.value && !((A = (B = d.target) == null ? void 0 : B.closest) != null && A.call(B, ".autodown-bubble-menu")) && (e.value = !1);
    }, c = (d) => {
      d.key === "Escape" && e.value && (e.value = !1);
    }, f = () => s();
    return me(() => {
      var B;
      const d = (B = t.editor) == null ? void 0 : B.__engine;
      d && (d.onChange(f), i = () => {
      }), document.addEventListener("pointerdown", u), document.addEventListener("keydown", c), document.addEventListener("selectionchange", f), s();
    }), Le(() => {
      i == null || i(), document.removeEventListener("pointerdown", u), document.removeEventListener("keydown", c), document.removeEventListener("selectionchange", f);
    }), () => {
      var d;
      return e.value ? le(
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
}), Hl = ["title", "onClick"], Pl = /* @__PURE__ */ J({
  __name: "BubbleMenu",
  props: {
    editor: {},
    linkPrompt: { default: "Enter URL" },
    tooltips: { default: null }
  },
  emits: ["RunButton"],
  setup(t, { emit: n }) {
    const e = t, o = m(() => [{ name: "bold", title: e.tooltips && e.tooltips.bold || "Bold", icon: Ae("bold"), active: e.editor.isActive("bold"), action: () => e.editor.chain().focus().toggleBold().run() }, { name: "italic", title: e.tooltips && e.tooltips.italic || "Italic", icon: Ae("italic"), active: e.editor.isActive("italic"), action: () => e.editor.chain().focus().toggleItalic().run() }, { name: "underline", title: e.tooltips && e.tooltips.underline || "Underline", icon: Ae("underline"), active: e.editor.isActive("underline"), action: () => e.editor.chain().focus().toggleUnderline().run() }, { name: "strike", title: e.tooltips && e.tooltips.strike || "Strikethrough", icon: Ae("strike"), active: e.editor.isActive("strike"), action: () => e.editor.chain().focus().toggleStrike().run() }, { name: "code", title: e.tooltips && e.tooltips.code || "Inline Code", icon: Ae("code"), active: e.editor.isActive("code"), action: () => e.editor.chain().focus().toggleCode().run() }, { name: "link", title: e.tooltips && e.tooltips.link || "Link", icon: Ae("link"), active: e.editor.isActive("link"), action: () => $l(e.editor, e.linkPrompt) }]), l = n;
    function a(i) {
      i.action(), l("RunButton", i);
    }
    return (i, s) => t.editor ? (g(), Y(re(Nl), {
      class: Te("autodown-bubble-menu"),
      editor: t.editor,
      options: { placement: "top" },
      shouldShow: re(Dl),
      key: "TiptapBubbleMenu-1"
    }, {
      default: Ye(() => [
        (g(!0), x(ye, null, Pe(o.value, (u) => (g(), x("button", {
          class: Te(["autodown-bubble-btn", { active: u.active }]),
          key: u.title,
          title: u.title,
          onClick: (c) => a(u)
        }, [
          (g(), Y(ve(u.icon), { size: 14 }))
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
}, Gl = /* @__PURE__ */ J({
  __name: "CodeBlockMenu",
  props: {
    editor: {}
  },
  emits: ["Init", "Destroy", "SearchInput", "MoveDown", "MoveUp", "SelectHighlighted", "SelectItem", "HoverItem", "Close", "OutsideClick"],
  setup(t, { emit: n }) {
    const e = t, o = w(!1), l = w(""), a = w(0), i = w(""), s = w(""), u = w(""), c = w(""), f = w(null), d = w(null), B = w(null), A = w(null), O = w(0), v = w(null), k = w(null), p = w(null), U = w(null), X = w(null), ie = w(null), te = w(null), K = m(() => on().filter((M) => [M.id, M.label].concat(M.aliases).join(" ").toLowerCase().includes(l.value.toLowerCase().trim()))), G = m(() => K.value.length === 0), _ = m(() => Fl()), I = n;
    function C() {
      o.value = !1, l.value = "", a.value = 0, f.value = null, I("Close");
    }
    function y(M) {
      a.value = M, I("HoverItem", M);
    }
    function R() {
      a.value < K.value.length - 1 && (a.value = a.value + 1, ae(() => {
        let M = null;
        if (d.value != null && (M = d.value.querySelector(".autodown-codeblock-menu")), M != null) {
          let F = M.querySelector(".autodown-codeblock-menu-list"), W = M.querySelector(".autodown-codeblock-menu-item.active");
          if (F != null && W != null) {
            let ne = F.getBoundingClientRect(), se = W.getBoundingClientRect(), D = se.top - ne.top - ne.height / 2 + se.height / 2;
            F.scrollTop = F.scrollTop + D;
          }
        }
      })), I("MoveDown");
    }
    function Q() {
      a.value > 0 && (a.value = a.value - 1, ae(() => {
        let M = null;
        if (d.value != null && (M = d.value.querySelector(".autodown-codeblock-menu")), M != null) {
          let F = M.querySelector(".autodown-codeblock-menu-list"), W = M.querySelector(".autodown-codeblock-menu-item.active");
          if (F != null && W != null) {
            let ne = F.getBoundingClientRect(), se = W.getBoundingClientRect(), D = se.top - ne.top - ne.height / 2 + se.height / 2;
            F.scrollTop = F.scrollTop + D;
          }
        }
      })), I("MoveUp");
    }
    function pe(M) {
      if (o.value) {
        let F = null;
        d.value != null && (F = d.value.querySelector(".autodown-codeblock-menu")), F != null && (F.contains(M.target) || (o.value = !1, l.value = "", a.value = 0, f.value = null));
      }
      I("OutsideClick", M);
    }
    function ce(M) {
      l.value = M.target.value, I("SearchInput", M);
    }
    function ge() {
      if (K.value.length == 1) {
        let M = K.value[0];
        e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null;
      }
      if (K.value.length != 1) {
        let M = K.value[a.value];
        M != null && (e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null);
      }
      I("SelectHighlighted");
    }
    function de(M) {
      e.editor.chain().focus().setCodeBlock({ language: M.id }).run(), o.value = !1, l.value = "", a.value = 0, f.value = null, I("SelectItem", M);
    }
    me(() => {
      let M = e.editor.view.dom;
      B.value = M, d.value = M.closest(".autodown-editor"), A.value = M.closest(".autodown-editor-content-wrapper");
      let F = () => {
        O.value != 0 && cancelAnimationFrame(O.value), O.value = requestAnimationFrame(() => {
          if (O.value = 0, o.value && d.value != null) {
            let D = d.value.getBoundingClientRect(), H = f.value;
            if (H == null) {
              let r = e.editor.view, h = r.nodeDOM(r.state.selection.from);
              h != null && h.closest != null && (H = h.closest("pre[data-language]"), H == null && (H = h.closest(".autodown-codeblock-node")));
            }
            if (H == null && (o.value = !1, l.value = "", a.value = 0, f.value = null), H != null) {
              let r = H.querySelector("[data-codeblock-language-badge]"), h = H;
              r != null && (h = r);
              let S = h.getBoundingClientRect(), q = { top: S.top - D.top + 6, left: S.left - D.left, bottom: S.bottom - D.top + 6, right: S.right - D.left, width: S.width, height: S.height }, E = { width: D.width, height: D.height }, L = be(q, 0, 0, E, "bottom-end", 0);
              s.value = L.top + "px", u.value = L.left + "px", c.value = "hidden", ae(() => {
                let P = d.value.querySelector(".autodown-codeblock-menu");
                if (P != null) {
                  let _e = P.getBoundingClientRect(), Z = be(q, _e.width, _e.height, E, "bottom-end", 0);
                  s.value = Z.top + "px", u.value = Z.left + "px", c.value = "visible";
                }
              });
            }
          }
        });
      };
      U.value = F;
      let W = (D) => {
        if (o.value) {
          let H = null;
          if (d.value != null && (H = d.value.querySelector(".autodown-codeblock-menu")), H != null && H.contains(D.target)) {
            D.preventDefault(), D.stopPropagation();
            let r = H.querySelector(".autodown-codeblock-menu-list");
            if (r != null) {
              let h = r.scrollTop + r.clientHeight < r.scrollHeight, S = r.scrollTop > 0;
              D.deltaY > 0 && h && (r.scrollTop = r.scrollTop + D.deltaY), D.deltaY < 0 && S && (r.scrollTop = r.scrollTop + D.deltaY);
            }
          }
          H == null && (D.preventDefault(), D.stopPropagation()), H != null && !H.contains(D.target) && (D.preventDefault(), D.stopPropagation());
        }
      };
      v.value = W, document.addEventListener("wheel", v.value, { passive: !1, capture: !0 });
      let ne = (D) => {
        let H = D.target, r = null, h = null, S = null, q = null;
        H.closest != null && (r = H.closest("[data-codeblock-language-badge]"), h = H.closest("[data-codeblock-copy-btn]"), S = H.closest("[data-codeblock-expand-btn]"), q = H.closest("[data-codeblock-more-btn]")), (r != null || h != null || S != null || q != null) && (D.preventDefault(), D.stopPropagation());
      };
      k.value = ne, M.addEventListener("mousedown", k.value, { capture: !0 });
      let se = (D) => {
        let H = D.target, r = null, h = null, S = null, q = null;
        if (H.closest != null && (r = H.closest("[data-codeblock-copy-btn]"), h = H.closest("[data-codeblock-expand-btn]"), S = H.closest("[data-codeblock-language-badge]"), q = H.closest("[data-codeblock-more-btn]")), r != null) {
          D.preventDefault(), D.stopPropagation();
          let E = r.closest("pre");
          if (E == null) {
            let P = r.closest(".code-block-container");
            P != null && (E = P.querySelector("pre[data-language]"));
          }
          let L = "";
          if (E != null) {
            let P = E.querySelector("code");
            P != null && (L = P.textContent ?? "");
          }
          navigator.clipboard.writeText(L);
        }
        if (r == null && h != null) {
          D.preventDefault(), D.stopPropagation();
          let E = h.closest("pre");
          if (E == null) {
            let L = h.closest(".code-block-container");
            L != null && (E = L.querySelector("pre[data-language]"));
          }
          E != null && E.classList.toggle("is-collapsed");
        }
        if (r == null && h == null) {
          let E = S;
          if (E == null && (E = q), E != null) {
            let L = E.closest("pre");
            L == null && (L = E.closest(".autodown-codeblock-node"));
            let P = !1;
            if (L == null && (L = E.closest(".code-block-container"), L != null && (P = !0)), D.preventDefault(), P == !1 && D.stopPropagation(), L == null && (L = E.closest(".code-block-container")), L == null) {
              let Z = e.editor.view, z = Z.nodeDOM(Z.state.selection.from);
              z != null && z.closest != null && (L = z.closest("pre[data-language]"), L == null && (L = z.closest(".autodown-codeblock-node")));
            }
            if (f.value = L, i.value = "", f.value != null && (i.value = f.value.getAttribute("data-language") ?? "", i.value == "")) {
              let Z = f.value.querySelector("pre[data-language]");
              Z != null && (i.value = Z.getAttribute("data-language") ?? "");
            }
            i.value == "" && (i.value = e.editor.getAttributes("codeBlock").language ?? ""), o.value = !0, l.value = "";
            let _e = on().findIndex((Z) => Z.id == i.value);
            a.value = _e, _e < 0 && (a.value = 0), ae(() => {
              let Z = null;
              if (d.value != null && (Z = d.value.querySelector(".autodown-codeblock-menu")), Z != null) {
                let z = Z.querySelector(".autodown-codeblock-menu-search");
                z != null && z.focus();
              }
              if (d.value != null) {
                let z = d.value.getBoundingClientRect(), oe = f.value;
                if (oe == null) {
                  let Ce = e.editor.view, fe = Ce.nodeDOM(Ce.state.selection.from);
                  fe != null && fe.closest != null && (oe = fe.closest("pre[data-language]"), oe == null && (oe = fe.closest(".autodown-codeblock-node")));
                }
                if (oe == null && (o.value = !1, l.value = "", a.value = 0, f.value = null), oe != null) {
                  let Ce = oe.querySelector("[data-codeblock-language-badge]"), fe = oe;
                  Ce != null && (fe = Ce);
                  let ke = fe.getBoundingClientRect(), We = { top: ke.top - z.top + 6, left: ke.left - z.left, bottom: ke.bottom - z.top + 6, right: ke.right - z.left, width: ke.width, height: ke.height }, $t = { width: z.width, height: z.height }, qt = be(We, 0, 0, $t, "bottom-end", 0);
                  s.value = qt.top + "px", u.value = qt.left + "px", c.value = "hidden", ae(() => {
                    let Nt = d.value.querySelector(".autodown-codeblock-menu");
                    if (Nt != null) {
                      let Ht = Nt.getBoundingClientRect(), Pt = be(We, Ht.width, Ht.height, $t, "bottom-end", 0);
                      s.value = Pt.top + "px", u.value = Pt.left + "px", c.value = "visible";
                    }
                  });
                }
              }
              ae(() => {
                let z = null;
                if (d.value != null && (z = d.value.querySelector(".autodown-codeblock-menu")), z != null) {
                  let oe = z.querySelector(".autodown-codeblock-menu-list"), Ce = z.querySelector(".autodown-codeblock-menu-item.active");
                  if (oe != null && Ce != null) {
                    let fe = oe.getBoundingClientRect(), ke = Ce.getBoundingClientRect(), We = ke.top - fe.top - fe.height / 2 + ke.height / 2;
                    oe.scrollTop = oe.scrollTop + We;
                  }
                }
              });
            });
          }
        }
      };
      p.value = se, M.addEventListener("click", p.value, { capture: !0 }), A.value != null && A.value.addEventListener("scroll", U.value, { passive: !0 });
    }), yt(() => {
      document.removeEventListener("wheel", v.value, { capture: !0 }), B.value != null && (B.value.removeEventListener("mousedown", k.value, { capture: !0 }), B.value.removeEventListener("click", p.value, { capture: !0 })), A.value != null && A.value.removeEventListener("scroll", U.value);
    });
    function Ke(M) {
      pe(M);
    }
    return me(() => {
      document.addEventListener("mousedown", Ke);
    }), yt(() => {
      document.removeEventListener("mousedown", Ke);
    }), (M, F) => o.value ? (g(), x("div", {
      key: 0,
      class: "autodown-codeblock-menu",
      ref_key: "menuEl",
      ref: X,
      style: gn({ top: s.value, left: u.value, visibility: c.value })
    }, [
      b("div", Kl, [
        st(b("input", {
          class: "autodown-codeblock-menu-search",
          placeholder: "Search language…",
          ref_key: "searchEl",
          ref: ie,
          "onUpdate:modelValue": F[0] || (F[0] = (W) => l.value = W),
          onInput: F[1] || (F[1] = (W) => ce(W)),
          onKeydown: [
            Ve(Oe(R, ["prevent"]), ["down"]),
            Ve(Oe(ge, ["prevent"]), ["enter"]),
            Ve(C, ["esc"]),
            Ve(Oe(Q, ["prevent"]), ["up"])
          ]
        }, null, 40, Wl), [
          [Bt, l.value]
        ])
      ]),
      b("div", {
        class: "autodown-codeblock-menu-list",
        ref_key: "listEl",
        ref: te
      }, [
        (g(!0), x(ye, null, Pe(K.value, (W, ne) => (g(), x("button", {
          class: Te(["autodown-codeblock-menu-item", { active: ne == a.value, selected: W.id == i.value }]),
          key: W.id,
          onClick: (se) => de(W),
          onMouseenter: (se) => y(ne)
        }, [
          b("span", Ql, [
            b("span", null, V(W.label), 1)
          ]),
          W.id == i.value ? (g(), Y(ve(_.value), {
            key: 0,
            class: "autodown-codeblock-menu-check",
            size: 13
          })) : $("", !0)
        ], 42, Vl))), 128)),
        G.value ? (g(), x("div", zl, [...F[2] || (F[2] = [
          b("span", null, "No matching languages", -1)
        ])])) : $("", !0)
      ], 512)
    ], 4)) : $("", !0);
  }
}), jl = { class: "autodown-slash-menu-items" }, Yl = ["onClick", "onMouseenter"], Jl = { class: "autodown-slash-menu-info" }, Xl = { class: "autodown-slash-menu-title" }, Zl = { class: "autodown-slash-menu-desc" }, ea = {
  key: 0,
  class: "autodown-slash-menu-empty"
}, ta = /* @__PURE__ */ J({
  __name: "SlashMenu",
  props: {
    editor: {},
    items: {},
    noResultsText: { default: "No results" }
  },
  emits: ["OnOpen", "OnUpdate", "OnClose", "OnKeydown", "SelectItem", "HoverItem"],
  setup(t, { emit: n }) {
    const e = t, o = w(!1), l = w(""), a = w(null), i = w(0), s = w(""), u = w(""), c = w(""), f = w(null), d = m(() => e.items.filter((_) => [_.title, _.description].concat(_.searchTerms).join(" ").toLowerCase().includes(l.value.toLowerCase()))), B = m(() => d.value.length === 0), A = m(() => e.noResultsText ?? "No results"), O = n;
    Ee(d, () => {
      i.value = 0;
    });
    function v(_) {
      i.value = _, O("HoverItem", _);
    }
    function k() {
      o.value = !1, l.value = "", a.value = null, i.value = 0, O("OnClose");
    }
    function p(_) {
      if (o.value) {
        if (_.detail.event.key == "ArrowDown") {
          _.detail.event.preventDefault();
          let I = i.value + 1;
          i.value = I % d.value.length, ae(() => {
            if (f.value) {
              let C = f.value.querySelector(".autodown-slash-menu-item.active");
              C != null && C.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (_.detail.event.key == "ArrowUp") {
          _.detail.event.preventDefault();
          let I = i.value - 1 + d.value.length;
          i.value = I % d.value.length, ae(() => {
            if (f.value) {
              let C = f.value.querySelector(".autodown-slash-menu-item.active");
              C != null && C.scrollIntoView({ block: "nearest", behavior: "auto" });
            }
          }), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        if (_.detail.event.key == "Enter" || _.detail.event.key == "NumpadEnter") {
          _.detail.event.preventDefault();
          let I = d.value[i.value];
          I != null && a.value != null && (I.command({ editor: e.editor, range: a.value }), o.value = !1, l.value = "", a.value = null, i.value = 0), e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0);
        }
        _.detail.event.key == "Escape" && (_.detail.event.preventDefault(), o.value = !1, l.value = "", a.value = null, i.value = 0, e.editor.storage["slash-command"] != null && (e.editor.storage["slash-command"].handled = !0));
      }
      O("OnKeydown", _);
    }
    function U(_) {
      l.value = _.detail.query, a.value = _.detail.range, o.value = !0, i.value = 0, ae(() => {
        if (a.value != null && e.editor.view) {
          let I = e.editor.view.coordsAtPos(a.value.from), C = e.editor.view.dom.closest(".autodown-editor");
          if (C != null) {
            let y = C.getBoundingClientRect(), R = { top: I.top - y.top, left: I.left - y.left, bottom: I.bottom - y.top, right: I.right - y.left, width: I.right - I.left, height: I.bottom - I.top }, Q = { width: y.width, height: y.height }, pe = be(R, 0, 0, Q, "bottom", 8, "left");
            s.value = pe.top + "px", u.value = pe.left + "px", c.value = "hidden", ae(() => {
              let ce = C.querySelector(".autodown-slash-menu");
              if (ce != null) {
                let ge = ce.getBoundingClientRect(), de = be(R, ge.width, ge.height, Q, "bottom", 8, "left");
                s.value = de.top + "px", u.value = de.left + "px", c.value = "visible";
              }
            });
          }
        }
      }), O("OnOpen", _);
    }
    function X(_) {
      l.value = _.detail.query, a.value = _.detail.range, ae(() => {
        if (a.value != null && e.editor.view) {
          let I = e.editor.view.coordsAtPos(a.value.from), C = e.editor.view.dom.closest(".autodown-editor");
          if (C != null) {
            let y = C.getBoundingClientRect(), R = { top: I.top - y.top, left: I.left - y.left, bottom: I.bottom - y.top, right: I.right - y.left, width: I.right - I.left, height: I.bottom - I.top }, Q = { width: y.width, height: y.height }, pe = be(R, 0, 0, Q, "bottom", 8, "left");
            s.value = pe.top + "px", u.value = pe.left + "px", c.value = "hidden", ae(() => {
              let ce = C.querySelector(".autodown-slash-menu");
              if (ce != null) {
                let ge = ce.getBoundingClientRect(), de = be(R, ge.width, ge.height, Q, "bottom", 8, "left");
                s.value = de.top + "px", u.value = de.left + "px", c.value = "visible";
              }
            });
          }
        }
      }), O("OnUpdate", _);
    }
    function ie(_) {
      let I = d.value[_];
      I != null && a.value != null && (I.command({ editor: e.editor, range: a.value }), o.value = !1, l.value = "", a.value = null, i.value = 0), O("SelectItem", _);
    }
    function te(_) {
      p(_);
    }
    function K(_) {
      U(_);
    }
    function G(_) {
      X(_);
    }
    return me(() => {
      document.addEventListener("autodown:slash-close", k), document.addEventListener("autodown:slash-keydown", te), document.addEventListener("autodown:slash-open", K), document.addEventListener("autodown:slash-update", G);
    }), yt(() => {
      document.removeEventListener("autodown:slash-close", k), document.removeEventListener("autodown:slash-keydown", te), document.removeEventListener("autodown:slash-open", K), document.removeEventListener("autodown:slash-update", G);
    }), (_, I) => o.value ? (g(), x("div", {
      key: 0,
      class: "autodown-slash-menu",
      ref_key: "menuEl",
      ref: f,
      style: gn({ top: s.value, left: u.value, visibility: c.value })
    }, [
      b("div", jl, [
        (g(!0), x(ye, null, Pe(d.value, (C, y) => (g(), x("button", {
          class: Te(["autodown-slash-menu-item", { active: y == i.value }]),
          key: C.title,
          onClick: (R) => ie(y),
          onMouseenter: (R) => v(y)
        }, [
          (g(), Y(ve(C.icon), {
            class: "autodown-slash-menu-icon",
            size: 16
          })),
          b("div", Jl, [
            b("div", Xl, [
              b("span", null, V(C.title), 1)
            ]),
            b("div", Zl, [
              b("span", null, V(C.description), 1)
            ])
          ])
        ], 42, Yl))), 128)),
        B.value ? (g(), x("div", ea, [
          b("span", null, V(A.value), 1)
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
J({
  name: "NodeViewWrapper",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(t, { slots: n, attrs: e }) {
    return () => {
      var o;
      return le(t.as, { ...e, "data-node-view-wrapper": "" }, (o = n.default) == null ? void 0 : o.call(n));
    };
  }
});
J({
  name: "NodeViewContent",
  props: {
    as: { type: [String, Object], default: "div" }
  },
  setup(t, { slots: n, attrs: e }) {
    const o = Vn($n, null);
    return () => {
      var a;
      const l = o ? o() : ((a = n.default) == null ? void 0 : a.call(n)) ?? [];
      return le(t.as, { ...e, "data-node-view-content": "" }, l);
    };
  }
});
let qn = {};
function bt(t) {
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
J({
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
    return j(this.engine.doc, this.tableId);
  }
  get rows() {
    var n;
    return ((n = this.table()) == null ? void 0 : n.children) ?? [];
  }
  cellText(n) {
    const e = j(this.engine.doc, n);
    return e ? he(e) : "";
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
    this.engine.applyTree((n) => we(n, this.tableId, [])), !j(this.engine.doc, this.engine.selection.anchor.blockId) && this.engine.doc.children[0] && this.engine.select(Ze(this.engine.doc.children[0].id, 0));
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
    const o = j(this.engine.doc, n);
    if (!o || o.kind !== T.TableCell) return !1;
    const l = he(o);
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
}, pa = { class: "math-block-source" }, ga = /* @__PURE__ */ J({
  __name: "MathBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(t, { emit: n }) {
    const e = t, o = w(""), l = w(""), a = w(tt(e.node)), i = w(Cn(e.ctx)), s = w(null), u = m(() => e.mode === "edit"), c = m(() => tt(e.node)), f = m(() => xt(e.ctx)), d = m(() => u.value ? f.value ? "autodown-math-editor is-readonly" : "autodown-math-editor" : "autodown-math-block"), B = m(() => Sn(e.mode)), A = m(() => nt(e.mode, Rt(e.ctx))), O = m(() => nt(e.mode, "MathBlock")), v = m(() => {
      var C;
      return (C = Ct(a.value, !0)) == null ? void 0 : C.html;
    }), k = m(() => {
      var C;
      return (C = Ct(a.value, !0)) == null ? void 0 : C.error;
    }), p = m(() => !k.value), U = m(() => !!k.value), X = m(() => Nn(a.value)), ie = m(() => !l.value), te = m(() => !!l.value), K = m(() => "code"), G = n;
    Ee(c, () => {
      if (!u.value) {
        let C = ln(c.value);
        o.value = C.html, l.value = C.error;
      }
    });
    function _(C) {
      a.value = C.target.value, G("AreaInput", C);
    }
    function I(C) {
      f.value || i.value.commit(C.target.value), G("Blur", C);
    }
    return me(() => {
      if (u.value && In(s.value, f.value), !u.value) {
        let C = ln(c.value);
        o.value = C.html, l.value = C.error;
      }
    }), (C, y) => (g(), x("div", {
      class: Te(d.value),
      "data-block-id": A.value,
      "data-math-block": B.value,
      "data-node-type": O.value
    }, [
      u.value ? (g(), x(ye, { key: 0 }, [
        f.value ? (g(), x("div", ua, [...y[3] || (y[3] = [
          b("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        b("div", ca, [
          p.value ? (g(), x("div", {
            key: 0,
            class: "autodown-math-preview",
            innerHTML: v.value
          }, null, 8, da)) : $("", !0),
          U.value ? (g(), x("div", fa, [
            b("span", null, V(k.value), 1)
          ])) : $("", !0),
          st(b("textarea", {
            class: "math-editor-textarea",
            disabled: f.value,
            ref_key: "area",
            ref: s,
            rows: X.value,
            spellcheck: "false",
            "onUpdate:modelValue": y[0] || (y[0] = (R) => a.value = R),
            onBlur: y[1] || (y[1] = (R) => I(R)),
            onInput: y[2] || (y[2] = (R) => _(R))
          }, null, 40, ha), [
            [Bt, a.value]
          ])
        ])
      ], 64)) : $("", !0),
      u.value ? $("", !0) : (g(), x(ye, { key: 1 }, [
        ie.value ? (g(), x("div", {
          key: 0,
          class: "autodown-math-preview",
          innerHTML: o.value
        }, null, 8, va)) : $("", !0),
        te.value ? (g(), x("div", ma, [
          b("span", null, V(l.value), 1)
        ])) : $("", !0),
        b("pre", pa, [
          y[4] || (y[4] = Xe("          ", -1)),
          (g(), Y(ve(K.value))),
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
const wa = ["data-block-id", "data-mermaid-block", "data-node-type"], ba = {
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
}, xa = { class: "mermaid-source" }, Ra = /* @__PURE__ */ J({
  __name: "MermaidBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "Blur"],
  setup(t, { emit: n }) {
    const e = t, o = w(""), l = w(""), a = w(tt(e.node)), i = w(""), s = w(""), u = w(!1), c = w(Cn(e.ctx)), f = w(null), d = m(() => e.mode === "edit"), B = m(() => tt(e.node)), A = m(() => xt(e.ctx)), O = m(() => d.value ? A.value ? "autodown-mermaid-editor is-readonly" : "autodown-mermaid-editor" : "autodown-mermaid-block"), v = m(() => Sn(e.mode)), k = m(() => nt(e.mode, Rt(e.ctx))), p = m(() => nt(e.mode, "Mermaid")), U = m(() => u.value === !1 && !s.value && !!i.value), X = m(() => u.value === !1 && !!s.value), ie = m(() => Nn(a.value)), te = m(() => !!o.value), K = m(() => !o.value && !!l.value), G = m(() => "code"), _ = n;
    Ee(B, () => {
      d.value || (B.value.trim() == "" && (o.value = "", l.value = ""), B.value.trim() != "" && an(B.value).then((R) => {
        o.value = R.svg, l.value = R.error;
      }));
    });
    function I(y) {
      a.value = y.target.value, u.value = !0, sn(a.value, (R) => {
        i.value = R.svg, s.value = R.error, u.value = R.loading;
      }), _("AreaInput", y);
    }
    function C(y) {
      A.value || c.value.commit(y.target.value), _("Blur", y);
    }
    return me(() => {
      d.value && (In(f.value, A.value), u.value = !0, sn(B.value, (y) => {
        i.value = y.svg, s.value = y.error, u.value = y.loading;
      })), d.value || (B.value.trim() == "" && (o.value = "", l.value = ""), B.value.trim() != "" && an(B.value).then((R) => {
        o.value = R.svg, l.value = R.error;
      }));
    }), (y, R) => (g(), x("div", {
      class: Te(O.value),
      "data-block-id": k.value,
      "data-mermaid-block": v.value,
      "data-node-type": p.value
    }, [
      d.value ? (g(), x(ye, { key: 0 }, [
        A.value ? (g(), x("div", ba, [...R[3] || (R[3] = [
          b("span", null, "流式生成中", -1)
        ])])) : $("", !0),
        b("div", ya, [
          U.value ? (g(), x("div", {
            key: 0,
            class: "autodown-mermaid-preview",
            innerHTML: i.value
          }, null, 8, _a)) : $("", !0),
          X.value ? (g(), x("div", Ca, [
            b("span", null, V(s.value), 1)
          ])) : $("", !0),
          u.value ? (g(), x("div", Sa, [...R[4] || (R[4] = [
            b("span", null, "渲染中…", -1)
          ])])) : $("", !0),
          st(b("textarea", {
            class: "mermaid-editor-textarea",
            disabled: A.value,
            ref_key: "area",
            ref: f,
            rows: ie.value,
            spellcheck: "false",
            "onUpdate:modelValue": R[0] || (R[0] = (Q) => a.value = Q),
            onBlur: R[1] || (R[1] = (Q) => C(Q)),
            onInput: R[2] || (R[2] = (Q) => I(Q))
          }, null, 40, Ia), [
            [Bt, a.value]
          ])
        ])
      ], 64)) : $("", !0),
      d.value ? $("", !0) : (g(), x(ye, { key: 1 }, [
        te.value ? (g(), x("div", {
          key: 0,
          class: "autodown-mermaid-preview",
          innerHTML: o.value
        }, null, 8, Ba)) : $("", !0),
        K.value ? (g(), x("div", Ta, [
          b("span", null, V(l.value), 1)
        ])) : $("", !0),
        b("pre", xa, [
          R[5] || (R[5] = Xe("          ", -1)),
          (g(), Y(ve(G.value))),
          R[6] || (R[6] = Xe(`
        `, -1))
        ])
      ], 64))
    ], 10, wa));
  }
}), Pn = /* @__PURE__ */ ct(Ra, [["__scopeId", "data-v-08feae74"]]), Ma = ["data-open"], Ea = { class: "autodown-details-summary" }, Aa = { class: "autodown-details-content" }, La = {
  key: 0,
  class: "markdown-renderer"
}, It = /* @__PURE__ */ J({
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
    const e = t, o = m(() => e.mode === "edit"), l = m(() => xt(e.ctx));
    m(() => Rt(e.ctx));
    const a = m(() => so(e.ctx)), i = m(() => uo(e.node, e.ctx)), s = m(() => co(e.node, "open")), u = m(() => s.value ? "▼" : "▶"), c = m(() => fo(e.node, "summary")), f = m(() => c.value ? c.value : "Details"), d = n;
    function B() {
      vo(a.value, i.value, s.value), d("ToggleOpen");
    }
    return (A, O) => (g(), x("div", {
      class: "autodown-details",
      "data-open": s.value
    }, [
      b("div", Ea, [
        b("span", {
          class: "autodown-details-marker",
          "aria-hidden": "true",
          title: "点击展开详细内容",
          onClick: Oe(B, ["stop"])
        }, [
          b("span", null, V(u.value), 1)
        ]),
        o.value ? (g(), Y(re(ho), {
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
        o.value ? $("", !0) : (g(), x("span", {
          key: 1,
          class: "autodown-details-summary-text",
          onClick: Oe(B, ["stop"])
        }, [
          b("span", null, V(f.value), 1)
        ]))
      ]),
      st(b("div", Aa, [
        o.value ? (g(), x("div", La, [
          (g(), Y(re(ot), {
            children_slot: t.children,
            key: "BlockChildren-2"
          }, null, 8, ["children_slot"]))
        ])) : $("", !0),
        o.value ? $("", !0) : (g(), Y(re(ot), {
          children_slot: t.children,
          key: "BlockChildren-3"
        }, null, 8, ["children_slot"]))
      ], 512), [
        [zn, s.value]
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
}, Wa = /* @__PURE__ */ J({
  __name: "QueryBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(t, { emit: n }) {
    const e = t, o = w([]), l = w(!1), a = w(""), i = m(() => Oa(e.node)), s = m(() => "code"), u = m(() => "ul"), c = m(() => "li"), f = m(() => l.value || !e.final), d = m(() => e.final && !l.value && !!a.value), B = m(() => e.final && !l.value && !a.value && o.value.length > 0), A = m(() => e.final && !l.value && !a.value && o.value.length === 0);
    return Ee(i, async () => {
      if (e.final) {
        let O = un();
        if ((O == null || i.value == "") && (a.value = "No query runner configured"), O != null && i.value != "") {
          l.value = !0, a.value = "";
          try {
            let v = await O(i.value);
            o.value = cn(v);
          } catch (v) {
            a.value = dn(v), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), me(async () => {
      if (e.final) {
        let O = un();
        if ((O == null || i.value == "") && (a.value = "No query runner configured"), O != null && i.value != "") {
          l.value = !0, a.value = "";
          try {
            let v = await O(i.value);
            o.value = cn(v);
          } catch (v) {
            a.value = dn(v), o.value = [];
          } finally {
            l.value = !1;
          }
        }
      }
    }), (O, v) => (g(), x("div", Da, [
      b("div", $a, [
        v[0] || (v[0] = b("span", { class: "query-label" }, [
          b("span", null, "Query")
        ], -1)),
        (g(), Y(ve(s.value), { class: "query-code" }, {
          default: Ye(() => [
            b("span", null, V(i.value), 1)
          ]),
          _: 1
        }))
      ]),
      f.value ? (g(), x("div", qa, [...v[1] || (v[1] = [
        b("span", null, "Loading query…", -1)
      ])])) : $("", !0),
      d.value ? (g(), x("div", Na, [
        b("span", null, V(a.value), 1)
      ])) : $("", !0),
      B.value ? (g(), Y(ve(u.value), {
        key: 2,
        class: "query-results"
      }, {
        default: Ye(() => [
          (g(!0), x(ye, null, Pe(o.value, (k, p) => (g(), Y(ve(c.value), {
            class: "query-result",
            key: p
          }, {
            default: Ye(() => [
              b("span", Ha, [
                b("span", null, V(k.marker), 1)
              ]),
              k.priority ? (g(), x("span", Pa, [
                b("span", null, V(k.priority_label), 1)
              ])) : $("", !0),
              b("span", Fa, [
                b("span", null, V(k.content), 1)
              ]),
              b("span", Ua, [
                b("span", null, V(k.source), 1)
              ])
            ]),
            _: 2
          }, 1024))), 128))
        ]),
        _: 1
      })) : $("", !0),
      A.value ? (g(), x("div", Ka, [...v[2] || (v[2] = [
        b("span", null, "No results", -1)
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
}, ei = /* @__PURE__ */ J({
  __name: "EmbedBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init"],
  setup(t, { emit: n }) {
    const e = t, o = w(null), l = w(!1), a = w(""), i = m(() => Qa(e.node)), s = m(() => za(e.node)), u = m(() => s.value != null ? i.value ? i.value + "#" + s.value : s.value : i.value), c = m(() => "Loading " + u.value + "…" || "Loading…"), f = m(() => o.value && o.value.content || ""), d = m(() => l.value || !e.final), B = m(() => e.final && !l.value && !!a.value), A = m(() => e.final && !l.value && !a.value && o.value), O = m(() => e.final && !l.value && !a.value);
    return Ee(s, async () => {
      if (e.final && s.value != null) {
        let v = fn();
        if (v == null && (a.value = "No block loader configured"), v != null) {
          l.value = !0, a.value = "";
          try {
            let k = await v(s.value);
            o.value = k, k || (a.value = "Block not found");
          } catch (k) {
            a.value = hn(k), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), me(async () => {
      if (e.final && s.value != null) {
        let v = fn();
        if (v == null && (a.value = "No block loader configured"), v != null) {
          l.value = !0, a.value = "";
          try {
            let k = await v(s.value);
            o.value = k, k || (a.value = "Block not found");
          } catch (k) {
            a.value = hn(k), o.value = null;
          } finally {
            l.value = !1;
          }
        }
      }
    }), (v, k) => (g(), x("div", {
      class: "autodown-block-embed",
      "data-title": i.value
    }, [
      d.value ? (g(), x("div", ja, [
        b("span", null, V(c.value), 1)
      ])) : $("", !0),
      B.value ? (g(), x("div", Ya, [
        b("span", null, V(a.value), 1)
      ])) : $("", !0),
      O.value ? (g(), x("div", Ja, [
        b("span", Xa, [
          b("span", null, V(u.value), 1)
        ])
      ])) : $("", !0),
      A.value ? (g(), x("div", Za, [
        b("span", null, V(f.value), 1)
      ])) : $("", !0)
    ], 8, Ga));
  }
}), ti = /* @__PURE__ */ ct(ei, [["__scopeId", "data-v-5badd3b4"]]), ni = {
  class: "blockquote",
  dir: "auto"
}, oi = {
  key: 0,
  class: "markdown-renderer"
}, vn = /* @__PURE__ */ J({
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
    const n = t, e = m(() => n.mode === "edit");
    return (o, l) => (g(), x("blockquote", ni, [
      e.value ? (g(), x("div", oi, [
        (g(), Y(re(ot), {
          children_slot: t.children,
          key: "BlockChildren-1"
        }, null, 8, ["children_slot"]))
      ])) : $("", !0),
      e.value ? $("", !0) : (g(), Y(re(ot), {
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
      checked: bn(n.attrs, "checked", !1),
      cls: "list-item" + (e ? " task-item" : ""),
      children_slot: () => Ue(ft(n.children), !0)
    };
  });
}
function ht(t, n) {
  return (e, o) => le(t, {
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
  return { id: t.id, text: he(t), cls: Kn(ut(t.attrs, "align", "left")) };
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
    return le(gt, {
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
    return le(gt, {
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
    const n = ia(), e = mo(t.node) ?? ii(T.Details, t.node);
    return le(It, {
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
const di = /* @__PURE__ */ J({
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
    const o = t, l = ["indigo", "coral", "ocean", "sage", "amber"], a = m(
      () => l.includes(o.accent) ? o.accent : "indigo"
    ), i = e;
    let s = null, u = null;
    const c = 250;
    function f(r) {
      const h = r ?? "\0null";
      h !== s && (s = h, ae(() => {
        const S = r ? W().find((q) => q.id === r) : void 0;
        i("focusblock", r && S ? { id: r, height: S.height } : null);
      }));
    }
    function d(r) {
      if (K) {
        K = !1, s = null;
        return;
      }
      if (u != null && (clearTimeout(u), u = null), !r) {
        f(null);
        return;
      }
      const h = p.selection.anchor.blockId || null;
      h && (u = setTimeout(() => {
        u = null, f(h);
      }, c));
    }
    const B = (r, h) => i("open-wiki-link", r, h);
    Gt(B), Le(() => {
      Bo() === B && Gt(null);
    });
    let A = !1;
    Ee(
      () => [o.runQuery, o.loadBlock],
      () => {
        o.runQuery != null || o.loadBlock != null ? (bt({ runQuery: o.runQuery, loadBlock: o.loadBlock }), A = !0) : A && (bt({}), A = !1);
      },
      { immediate: !0 }
    ), Le(() => {
      if (!A) return;
      const r = Ot();
      r.runQuery === o.runQuery && r.loadBlock === o.loadBlock && bt({});
    });
    const O = w(null), v = w(null);
    function k(r) {
      return wn(r ?? "", !0);
    }
    const p = new nl(k(o.modelValue ?? o.content ?? "")), U = fl(p), X = oa({ extraSlashItems: o.extraSlashItems });
    let ie = Qe(p.doc, !0);
    p.onChange(() => {
      _.value++;
      const r = te();
      d(r);
    });
    function te() {
      const r = Qe(p.doc, !0);
      return r === ie ? !1 : (ie = r, i("update", r), i("update:modelValue", r), !0);
    }
    let K = !1;
    Ee(
      () => o.modelValue ?? o.content,
      (r) => {
        r != null && r !== Qe(p.doc, !0) && (K = !0, p.replaceDoc(k(r)));
      }
    );
    function G() {
      var _e;
      const r = typeof window > "u" ? null : window.getSelection();
      if (!r || r.rangeCount === 0) return;
      const h = r.getRangeAt(0);
      if (h.collapsed) return;
      const S = h.startContainer, q = S.nodeType === 3 ? S.parentElement : S, E = q == null ? void 0 : q.closest(".autodown-block-host");
      if (!E || !((_e = O.value) != null && _e.contains(E))) return;
      const L = E.dataset.blockId;
      if (!L) return;
      const P = il(E, L);
      !P || P.lo === P.hi || p.selection.anchor.blockId === L && p.selection.anchor.offset === P.lo && p.selection.head.offset === P.hi || p.select(new mt(new ze(L, P.lo), new ze(L, P.hi)));
    }
    me(() => document.addEventListener("selectionchange", G)), Le(() => document.removeEventListener("selectionchange", G)), Le(() => {
      u != null && clearTimeout(u);
    });
    const _ = w(0), I = w(0), C = J({
      name: "AssemblyView",
      props: { render: { type: Function, required: !0 } },
      setup(r) {
        return () => r.render();
      }
    }), y = m(() => {
      _.value;
      const r = p.selection.anchor.blockId, h = { path: go(p.doc, r), focusedId: r, counter: { n: 0 } };
      return p.doc.children.map((S) => de(S, h, !0));
    });
    function R(r) {
      la({ engine: p, adapter: U });
      try {
        return Ue(ft([r]), !0)[0] ?? le("div", { class: "unknown-node" }, "");
      } finally {
        aa();
      }
    }
    function Q(r, h, S, q, E = !0) {
      return le(
        "div",
        {
          class: "node-slot",
          "data-node-index": String(q.n++),
          "data-node-type": T[r.kind],
          "data-block-id": r.id,
          // the innermost addressable slot wins — an expanded container's outer
          // chrome must not re-handle the bubbled click (it would resolve the
          // whole container back to its first leaf)
          onClick: (L) => {
            L.stopPropagation();
            const P = bo(r, L.currentTarget, L);
            P.anchor && yo(L, P.targetId, P.anchor, P.cellId), ne(r.id, P.targetId);
          }
        },
        [
          le("div", { class: "node-content" }, [h]),
          ...S && E ? [le("div", { class: "autodown-block-boundary", "data-boundary-for": r.id })] : []
        ]
      );
    }
    function pe(r, h) {
      return r.children.map((S) => {
        const q = Tt(S.attrs, "checked") != null;
        return {
          id: S.id,
          task: q,
          checked: bn(S.attrs, "checked", !1),
          cls: "list-item" + (q ? " task-item" : ""),
          children_slot: () => S.children.map((E) => ce(E, h))
        };
      });
    }
    function ce(r, h) {
      return r.id === h.focusedId || h.path.has(r.id) ? ge(de(r, h, !1)) : Q(r, R(r), !1, h.counter);
    }
    function ge(r) {
      return le(r.view, r.props);
    }
    function de(r, h, S) {
      if (r.id === h.focusedId) {
        const q = Vt(T[r.kind]);
        if (q)
          return {
            id: r.id,
            view: C,
            props: {
              render: () => Q(r, q(r, { engine: p, blockId: r.id, readonly: o.streaming === !0 }), S, h.counter, !1),
              key: `edit:${r.id}:${I.value}`
            }
          };
        if (ko(r)) {
          const E = F(r.id), L = r.kind === T.Heading ? Xn(r.attrs, "level", 1) : void 0;
          return {
            id: r.id,
            view: C,
            props: {
              render: () => Q(
                r,
                le(Al, {
                  controller: E,
                  // flat chrome data (plan 034 D4): the widget derives tag/cls from
                  // blockKind/level itself (the host-face computation is absorbed);
                  // initial_html is the mount-once rich snapshot, evaluated here —
                  // the engine is not Vue-reactive, the snapshot never invalidates.
                  blockId: E.id,
                  blockKind: T[r.kind],
                  level: L ?? 0,
                  initial_html: yn(E.inlines),
                  // The face lives in the key: a kind/level flip mid-typing (input
                  // rules) must REMOUNT the host. <component :is> would swap the
                  // DOM element under the caret without re-running onMounted —
                  // focus lands nowhere and every post-flip keystroke is lost.
                  // The remount re-focuses at end (plan 029; rules match only a
                  // whole-block marker, so the caret IS at end on every flip).
                  key: `host:${r.id}:${T[r.kind]}:${L ?? ""}:${I.value}`
                }),
                S,
                h.counter,
                !1
              )
            }
          };
        }
      }
      if (h.path.has(r.id) && Ke(r)) {
        const q = Vt(T[r.kind]);
        if (q)
          return {
            id: r.id,
            view: C,
            props: {
              render: () => Q(r, q(r, {
                engine: p,
                blockId: r.id,
                readonly: o.streaming === !0,
                children: () => r.children.map((E) => ce(E, h)),
                items: () => pe(r, h),
                version: _.value
              }), S, h.counter)
            }
          };
      }
      return {
        id: r.id,
        view: C,
        props: { render: () => Q(r, R(r), S, h.counter) }
      };
    }
    function Ke(r) {
      return r.children.length > 0 && (r.kind === T.ListBlock || r.kind === T.Blockquote || r.kind === T.Callout || r.kind === T.Details);
    }
    const M = /* @__PURE__ */ new Map();
    function F(r) {
      let h = M.get(r);
      return h || (h = new wo(p, r), M.set(r, h)), h;
    }
    function W() {
      const r = [], h = v.value;
      return h && Array.from(h.querySelectorAll("[data-block-id]")).forEach((q, E) => {
        r.push({
          id: q.dataset.blockId ?? "",
          index: E,
          pos: E,
          el: q,
          top: q.offsetTop,
          height: q.offsetHeight
        });
      }), r;
    }
    function ne(r, h) {
      const S = j(p.doc, r);
      if (!S) return;
      const q = h && h !== r ? j(p.doc, h) : null, E = (q && pt(q) === q ? q : null) ?? pt(S) ?? S, L = new ze(E.id, 0);
      p.select(new mt(L, L)), _.value++;
    }
    function se(r) {
      const h = ol(r);
      if (h) {
        if (Array.from(M.values()).some((S) => S.composition.composing)) return;
        r.preventDefault(), ll(p, M.values(), h) && I.value++;
        return;
      }
      if (r.ctrlKey && r.key === "End") {
        r.preventDefault();
        const S = _o(p.doc);
        S && ne(S.id);
      }
    }
    function D() {
      const r = pt(p.doc);
      if (r) {
        const h = new ze(r.id, 0);
        p.select(new mt(h, h));
      }
    }
    D();
    function H() {
      i("save", Qe(p.doc, !0));
    }
    return n({ getBlockMap: W, handleSave: H }), (r, h) => (g(), x("div", {
      ref_key: "root",
      ref: O,
      class: Te(["autodown-editor", { "is-dark": t.darkMode }]),
      "data-accent": a.value
    }, [
      b("div", {
        ref_key: "wrapper",
        ref: v,
        class: "autodown-editor-content-wrapper"
      }, [
        b("div", {
          class: "autodown-editor-content",
          "data-engine-editor": "",
          tabindex: "-1",
          onKeydown: se
        }, [
          vt(re(ta), {
            editor: re(U),
            items: re(X)
          }, null, 8, ["editor", "items"]),
          vt(Pl, { editor: re(U) }, null, 8, ["editor"]),
          vt(Gl, { editor: re(U) }, null, 8, ["editor"]),
          (g(!0), x(ye, null, Pe(y.value, (S) => (g(), Y(ve(S.view), Gn({
            key: S.id
          }, { ref_for: !0 }, S.props), null, 16))), 128))
        ], 32)
      ], 512),
      b("div", { class: "autodown-editor-actions" }, [
        b("button", {
          type: "button",
          class: "autodown-editor-save",
          onClick: H
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
  bt as s
};
