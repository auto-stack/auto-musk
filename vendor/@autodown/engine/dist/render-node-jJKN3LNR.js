import { h as v, defineComponent as at, ref as ut, computed as k, onMounted as Ft, openBlock as m, createElementBlock as b, normalizeClass as D, createElementVNode as h, createCommentVNode as O, toDisplayString as wt, createStaticVNode as zt, withDirectives as Ke, vModelText as Ue, Fragment as H, watch as Ve, withKeys as Xt, withModifiers as Rt, createBlock as K, unref as z, resolveDynamicComponent as ae, withCtx as ce, renderList as F, getCurrentInstance as Qe } from "vue";
import je from "katex";
import ue from "mermaid";
import { createLowlight as Ge, common as ze } from "lowlight";
import { toHtml as Xe } from "hast-util-to-html";
import { f as y, r as S, w as Q, a as de, b as q, M as _, c as Ye, B as c, W as Z, d as A, e as nt, t as Je, g as Ze, h as V, i as $t, j as tn, l as en, k as Nt, q as nn, m as ln, n as rn, o as on, u as P, v as sn, x as an, y as cn, z as un, A as dn, C as fe, D as fn, E as hn, F as mn, G as kn, H as pn, I as gn, O as U, J as he, K as R, S as j, R as xt, L as bn, N as me, P as vn, Q as ke, V as L, T as C, U as lt, X, Y as Ht, Z as yn, _ as pe, $ as ct, a0 as mt, a1 as $, a2 as kt, a3 as Yt, a4 as bt, a5 as wn, a6 as _n, a7 as Cn, a8 as Jt, a9 as xn, aa as In } from "./parser-BfX0E-c9.js";
function N(t, e, n, l, r) {
  return { kind: t, tag: e, class_token: n, registry: l, extension: r };
}
function ge(t) {
  let e = t;
  return e < 1 && (e = 1), e > 6 && (e = 6), N("H" + String(e), "h" + String(e), "heading-node", "Heading", !1);
}
function Bn(t) {
  return t == "paragraph" ? N("Text", "p", "paragraph-node", "Text", !1) : t == "text" ? N("Text", "span", "text-node", "Text", !1) : t == "heading" ? ge(1) : t == "thematic_break" ? N("Separator", "hr", "hr-node", "Separator", !1) : t == "code_block" ? N("Codeblock", "div", "code-block-container", "Codeblock", !1) : t == "blockquote" ? N("Quote", "blockquote", "blockquote", "Quote", !1) : t == "list" ? N("List", "ul", "list-node", "List", !1) : t == "table" ? N("Table", "table", "table-node", "Table", !1) : t == "callout" ? N("Callout", "div", "callout-node", "Callout", !0) : t == "details" ? N("Details", "div", "details-node", "Details", !0) : t == "math_block" ? N("MathBlock", "div", "math-block", "MathBlock", !0) : t == "mermaid" ? N("Mermaid", "div", "mermaid-block-container", "Mermaid", !0) : t == "query" ? N("Query", "div", "query-block", "Query", !0) : t == "embed" ? N("Embed", "div", "embed-block", "Embed", !0) : N("Unknown", "div", "unknown-node", "", !1);
}
function Sn({ node: t, final: e, budget: n, renderInlineChildren: l }) {
  return t.type === "text" ? v("span", { class: "whitespace-pre-wrap break-words text-node" }, [v("span", t.content)]) : v("p", { class: "paragraph-node", dir: "auto" }, l(t.children, e, n));
}
function Tn({ node: t, final: e, budget: n, renderInlineChildren: l }) {
  const r = Math.min(6, Math.max(1, t.level));
  return v(`h${r}`, { class: `heading-node heading-${r}`, dir: "auto" }, [
    ...l(t.children, e, n)
  ]);
}
function An() {
  return v("hr", { class: "hr-node" });
}
function Mn({ node: t, final: e, budget: n, renderEmbedded: l }) {
  return v("blockquote", { class: "blockquote", dir: "auto" }, [
    l(t.children, e, n)
  ]);
}
const On = ["note", "info", "tip", "warning", "caution", "danger", "error"], tt = Tn, Ln = {
  Text: Sn,
  H1: tt,
  H2: tt,
  H3: tt,
  H4: tt,
  H5: tt,
  H6: tt,
  Separator: An,
  Quote: Mn
  // List and Callout are NOT here anymore (plan 035 T6): the container
  // families' widgets own those panel faces, registered on the custom slot
  // by block-widget-panels.ts (same channel Codeblock took in 033).
}, ht = {};
function It(t, e) {
  ht[t] = e;
}
function qo(t) {
  delete ht[t];
}
function Fo() {
  for (const t of Object.keys(ht)) delete ht[t];
}
function Rn(t) {
  return (t == null ? void 0 : t.type) === "heading" ? ge(t.level) : Bn((t == null ? void 0 : t.type) ?? "");
}
function $n(t) {
  return ht[t.kind] ?? Ln[t.kind];
}
let Nn = null;
function Wt() {
  return Nn;
}
let dt = null;
function Wo(t) {
  dt = t;
}
function Ko() {
  return dt;
}
function Hn(t, e) {
  dt == null || dt(t, e);
}
function Pn() {
  return 2166136261;
}
function Dn() {
  return 16777619;
}
function Ot() {
  return 65536 * 65536;
}
function Zt(t, e) {
  const n = t % e;
  return (t - n) / e % 2;
}
function En(t, e) {
  let n = 0, l = 1, r = 0;
  for (; r < 32; ) {
    const i = Zt(t, l), o = Zt(e, l);
    i != o && (n = n + l), l = l * 2, r = r + 1;
  }
  return n;
}
function qn(t, e) {
  const n = t % 65536, i = (t - n) / 65536 * e, o = n * e, s = i % Ot(), a = o % Ot();
  return (s * 65536 + a) % Ot();
}
function Fn(t, e) {
  const n = En(t, e);
  return qn(n, Dn());
}
function Wn(t) {
  return "0123456789abcdef".slice(t, t + 1);
}
function Kn(t) {
  let e = "", n = t, l = 0;
  for (; l < 8; ) {
    const r = n % 16;
    e = Wn(r) + e, n = (n - r) / 16, l = l + 1;
  }
  return e;
}
function Un(t, e, n) {
  let l = Pn();
  for (let r = 0; r < Number(n.length); r++)
    l = Fn(l, n[r]);
  return t + ":" + String(e) + ":" + Kn(l);
}
function Vn(t, e) {
  const n = t + "\0" + e, l = new Array(n.length);
  for (let r = 0; r < n.length; r++) l[r] = n.charCodeAt(r);
  return Un(t, e.length, l);
}
let be = null;
function ve(t) {
  be = t;
}
function ye() {
  return be;
}
const _t = {};
function Kt(t, e, n) {
  _t[t] = { enabled: e, factory: n };
}
function Uo(t) {
  Kt("katex", !0, t);
}
function Vo(t) {
  Kt("mermaid", !0, t);
}
function Qo(t) {
  Kt("highlight", !0), ve(t ?? null);
}
function Qn(t) {
  var e;
  return ((e = _t[t]) == null ? void 0 : e.enabled) === !0;
}
let we = null;
function jn() {
  return we;
}
function jo() {
  for (const t of Object.keys(_t))
    delete _t[t];
  ve(null), we = null;
}
ue.initialize({ startOnLoad: !1, theme: "default" });
function Gn(t, e) {
  try {
    return {
      html: je.renderToString(t, { throwOnError: !0, displayMode: e }),
      error: ""
    };
  } catch (n) {
    return { html: "", error: n.message || String(n) };
  }
}
async function Go(t) {
  try {
    const e = `mermaid-${Math.random().toString(36).slice(2)}`;
    return { svg: (await ue.render(e, t)).svg, error: "" };
  } catch (e) {
    return { svg: "", error: e.message || String(e) };
  }
}
function zo(t, e, n) {
  if (n.error !== "") return;
  const l = jn();
  l && l.put(Vn(t, e), n);
}
const te = Ge(ze), _e = (t, e) => {
  if (!(!t || !e || e === "text" || e === "plaintext"))
    try {
      return te.registered(e) ? Xe(te.highlight(e, t)) : void 0;
    } catch {
      return;
    }
};
class zn {
  constructor(e, n) {
    this.engine = e, this.blockId = n, this.knownCode = this.readModel();
  }
  get id() {
    return this.blockId;
  }
  get code() {
    return this.knownCode;
  }
  /** The live block (attrs included) — the SFC reads the language from it. */
  node() {
    return y(this.engine.doc, this.blockId);
  }
  /** The engine repaints after history changes / external edits — re-sync. */
  syncFromModel() {
    return this.knownCode = this.readModel(), this.knownCode;
  }
  /** Write the edited code text back; false = no change or block gone. */
  commit(e) {
    if (e === this.knownCode) return !1;
    const n = y(this.engine.doc, this.blockId);
    return n ? (this.engine.applyTree((l) => S(l, this.blockId, [Q(n, [de(e)])])), this.syncFromModel(), !0) : !1;
  }
  /** The code widget's commit: the edit face drafts the COLLAPSED text
   *  (draftCodeOf strips the closed-fence representation newline), so the
   *  model representation is restored here before the compare+write — an
   *  untouched blur stays a no-op, a real edit always wins, and a trailing
   *  newline the user actually typed becomes model content on top of the
   *  representation. Math/mermaid draft the model text verbatim and keep
   *  using commit. */
  commitDraft(e) {
    return this.knownCode.endsWith(`
`) && (e = e + `
`), this.commit(e);
  }
  readModel() {
    const e = y(this.engine.doc, this.blockId);
    return e ? q(e) : "";
  }
}
const Ce = /* @__PURE__ */ new WeakMap();
function xe(t) {
  return Ce.get(t);
}
function Xo(t) {
  return (t ?? []).map(G);
}
function G(t) {
  const e = Xn(t);
  return Ce.set(e, t), e;
}
function Xn(t) {
  switch (t.kind) {
    case c.Heading:
      return rn(Nt(t.attrs, "level", 1), Pt(t.inlines));
    case c.Fence:
      return ln(
        A(t.attrs, "language", ""),
        nt(t.inlines),
        V(t.attrs, "loading", !1)
      );
    case c.Blockquote:
      return nn(t.children.map(G));
    case c.ListBlock:
      return en(
        V(t.attrs, "ordered", !1),
        Nt(t.attrs, "start", 1),
        t.children.map(G)
      );
    case c.ListItem: {
      const n = $t(t.attrs, "checked") == null ? null : V(t.attrs, "checked", !1);
      return tn(t.children.map(G), n);
    }
    case c.Table: {
      const e = t.children.map(Yn);
      return Ze(e.length > 0 ? [e[0]] : [], e.slice(1), V(t.attrs, "loading", !1));
    }
    case c.ThematicBreak:
      return Je();
    case c.Callout: {
      const e = new Z(
        "callout",
        null,
        null,
        null,
        null,
        null,
        t.children.map(G),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null
      );
      return e.language = A(t.attrs, "type", ""), e.title = A(t.attrs, "title", ""), e;
    }
    case c.Details:
      return new Z(
        "details",
        null,
        null,
        null,
        null,
        null,
        t.children.map(G),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null
      );
    case c.MathBlock:
      return new Z(
        "math_block",
        null,
        null,
        null,
        nt(t.inlines),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null
      );
    case c.Mermaid:
      return new Z(
        "mermaid",
        null,
        null,
        null,
        nt(t.inlines),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null
      );
    case c.QueryBlock:
      return new Z(
        "query",
        A(t.attrs, "query", ""),
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null
      );
    case c.BlockEmbed:
      return new Z(
        "embed",
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        A(t.attrs, "src", ""),
        null,
        null
      );
    default:
      return Ye(Pt(t.inlines));
  }
}
function Yn(t) {
  return on(t.children.map(Jn));
}
function Jn(t) {
  return sn(
    V(t.attrs, "header", !1),
    Pt(t.inlines),
    A(t.attrs, "align", "left")
  );
}
const Zn = [_.Strong, _.Em, _.Underline, _.Del, _.Link];
function Pt(t) {
  return Dt(t, 0);
}
function Dt(t, e) {
  const n = Zn[e];
  if (n === void 0) return el(t);
  const l = [];
  let r = [];
  const i = () => {
    r.length !== 0 && (l.push(tl(n, r, Dt(r, e + 1))), r = []);
  };
  for (const o of t)
    P(o.marks, n) ? r.push(o) : (i(), l.push(...Dt([o], e + 1)));
  return i(), l;
}
function tl(t, e, n) {
  switch (t) {
    case _.Strong:
      return gn(n);
    case _.Em:
      return pn(n);
    case _.Underline:
      return kn(n);
    case _.Del:
      return mn(n);
    case _.Link: {
      const l = A(e[0].attrs, "href", ""), r = A(e[0].attrs, "title", "");
      return fe(l, r.length > 0 ? r : null, nt(e), n, !1);
    }
    default:
      return n[0];
  }
}
function el(t) {
  const e = [];
  for (const n of t) {
    if (n.text === `
`) {
      e.push(an());
      continue;
    }
    if (A(n.attrs, "wikilink", "") !== "") {
      e.push(cn(n.text));
      continue;
    }
    if (A(n.attrs, "math_inline", "") !== "") {
      e.push(un(n.text));
      continue;
    }
    if (P(n.marks, _.Image)) {
      const l = dn(A(n.attrs, "src", ""), n.text), r = A(n.attrs, "title", "");
      if (r.length > 0 && (l.title = r), P(n.marks, _.Link)) {
        const i = A(n.attrs, "href", "");
        e.push(fe(i, null, n.text, [l], !1));
      } else
        e.push(l);
      continue;
    }
    if (P(n.marks, _.Code)) {
      e.push(fn(n.text));
      continue;
    }
    e.push(hn(n.text));
  }
  return e;
}
function Ut(t) {
  const e = (t ?? "").split(/[_-]+/).filter((n) => n.length > 0);
  return e.length === 0 ? "" : e.map((n) => n.charAt(0).toUpperCase() + n.slice(1)).join("");
}
const rt = {};
function nl(t, e) {
  const n = Ut(t);
  rt[n] = { ...rt[n], ...e };
}
function ll(t) {
  delete rt[Ut(t)];
}
function Yo() {
  for (const t of Object.keys(rt)) delete rt[t];
}
function ee(t, e) {
  return To([G(t)], e)[0];
}
function rl(t) {
  const e = rt[Ut(t)];
  return e ? {
    view: e.view ?? ee,
    stream: e.stream,
    edit: e.edit
  } : { view: ee };
}
function Ie(t) {
  return rl(t).edit;
}
function Jo(t) {
  return (e, n) => v(t, { node: e, ctx: n });
}
class ol {
  constructor() {
    this.active = !1, this.baseline = "", this.blockId = "", this.baselineOffset = 0;
  }
  get composing() {
    return this.active;
  }
  /** compositionstart — record the pre-edit state of the focused block. */
  begin(e, n, l) {
    this.active = !0, this.blockId = e, this.baseline = n, this.baselineOffset = l;
  }
  /** compositionupdate — staged preedit; produces NO op by contract. */
  update(e) {
    return this.active, null;
  }
  /** compositionend — diff baseline → final text into one op. */
  commit(e) {
    if (!this.active) return null;
    if (this.active = !1, this.baseline.length === 0 && e.length > 0)
      return U.InsertText(new he(new R(this.blockId, this.baselineOffset), e));
    if (e === this.baseline) return null;
    const n = new j(
      new R(this.blockId, this.baselineOffset),
      new R(this.blockId, this.baselineOffset + this.baseline.length)
    );
    return U.ReplaceRange(new xt(n, e));
  }
  /** composition cancelled — nothing happened, by contract. */
  cancel() {
    return this.active = !1, null;
  }
}
function il(t, e, n) {
  if (e === n) return null;
  let l = 0;
  for (; l < e.length && l < n.length && e[l] === n[l]; ) l++;
  let r = 0;
  for (; r < e.length - l && r < n.length - l && e[e.length - 1 - r] === n[n.length - 1 - r]; )
    r++;
  const i = e.slice(l, e.length - r), o = n.slice(l, n.length - r);
  if (i.length === 0)
    return U.InsertText(new he(new R(t, l), o));
  const s = new j(new R(t, l), new R(t, l + i.length));
  return U.ReplaceRange(new xt(s, o));
}
const sl = [
  { marker: "# ", kind: c.Heading, level: 1 },
  { marker: "## ", kind: c.Heading, level: 2 },
  { marker: "### ", kind: c.Heading, level: 3 },
  { marker: "- ", kind: c.ListItem, wrap: c.ListBlock },
  { marker: "* ", kind: c.ListItem, wrap: c.ListBlock },
  { marker: "+ ", kind: c.ListItem, wrap: c.ListBlock },
  { marker: "> ", kind: c.Blockquote, wrap: c.Blockquote },
  { marker: "``` ", kind: c.Fence },
  { marker: "---", kind: c.ThematicBreak },
  { marker: "***", kind: c.ThematicBreak }
];
function al(t) {
  for (const e of sl)
    if (t === e.marker) return e;
  return null;
}
function cl(t, e, n) {
  const l = y(t, e);
  if (!l) return null;
  const r = q(l);
  if (r !== n.marker) return null;
  const i = new j(new R(e, 0), new R(e, r.length)), o = [U.ReplaceRange(new xt(i, ""))];
  return n.wrap || o.push(U.SetBlockType(new bn(e, n.kind))), { ops: o, rule: n };
}
function ul(t, e, n) {
  if (n.level == null) return t;
  const l = y(t, e);
  return l ? S(t, e, [vn({ ...l, attrs: ke(l.attrs, "level", L.Int(n.level)) }, l.kind)]) : t;
}
function dl(t, e, n) {
  const l = [];
  for (; l.length < e; ) {
    const r = `${n}-${Math.random().toString(36).slice(2, 8)}`;
    !me(t, r) && !l.includes(r) && l.push(r);
  }
  return l;
}
function fl(t, e, n, l) {
  const r = y(t, e);
  if (!r || !n.wrap) return t;
  if (n.wrap === c.ListBlock) {
    const [i, o] = l, s = C(lt(o ?? "li-x", c.ListItem), [r]);
    return S(t, e, [C(lt(i ?? "lb-x", c.ListBlock), [s])]);
  }
  return S(t, e, [C(lt(l[0] ?? "bq-x", c.Blockquote), [r])]);
}
function hl(t, e) {
  const n = y(t.doc, e);
  if (!n) return !1;
  const l = al(q(n));
  if (!l) return !1;
  const r = cl(t.doc, e, l);
  if (!r) return !1;
  const i = l.wrap ? dl(t.doc, l.wrap === c.ListBlock ? 2 : 1, "b") : [];
  return t.applyGroup(r.ops, (o) => ul(fl(o, e, l, i), e, l)), !0;
}
function ml(t, e) {
  if (t.length !== e.length) return !1;
  for (const n of t) if (!P(e, n)) return !1;
  return !0;
}
function kl(t, e) {
  return t._tag !== e._tag ? !1 : !("value" in t) || !("value" in e) ? !0 : JSON.stringify(t.value) === JSON.stringify(e.value);
}
function pl(t, e) {
  if (t.length !== e.length) return !1;
  for (const n of t) {
    const l = e.find((r) => r.key === n.key);
    if (!l || !kl(n.value, l.value)) return !1;
  }
  return !0;
}
function Vt(t) {
  const e = [];
  for (const n of t) {
    if (n.text === "") continue;
    const l = e[e.length - 1];
    l && ml(l.marks, n.marks) && pl(l.attrs, n.attrs) ? e[e.length - 1] = new X(l.text + n.text, l.marks, l.attrs) : e.push(n);
  }
  return e;
}
function Et(t, e, n) {
  if (t.length === 0) return [];
  const l = nt(t).length;
  e = Math.max(0, Math.min(e, l)), n = Math.max(e, Math.min(n, l));
  let r = [], i = 0, o = null;
  for (const a of t) {
    const d = i + a.text.length;
    e < d && n > i && r.push(a), i <= e && d > e && (o = a), d <= e && (o = a), i = d;
  }
  r.length === 0 && (r = o ? [o] : [t[t.length - 1]]);
  let s = [...r[0].marks];
  for (const a of r.slice(1)) s = s.filter((d) => P(a.marks, d));
  return s;
}
function gl(t, e, n, l, r) {
  const i = [];
  let o = 0;
  for (const s of t) {
    const a = o, d = o + s.text.length, u = Math.max(e, a), f = Math.min(n, d);
    if (u < f) {
      const p = u - a, g = f - a;
      p > 0 && i.push(new X(s.text.slice(0, p), s.marks, s.attrs)), i.push(new X(s.text.slice(p, g), r === "add" ? Ht(s.marks, l) : yn(s.marks, l), s.attrs)), g < s.text.length && i.push(new X(s.text.slice(g), s.marks, s.attrs));
    } else
      i.push(s);
    o = d;
  }
  return Vt(i);
}
function bl(t, e, n, l) {
  const r = nt(t).length;
  if (e = Math.max(0, e), n = Math.min(n, r), e >= n) return t;
  const i = Et(t, e, n);
  return gl(t, e, n, l, P(i, l) ? "remove" : "add");
}
const vl = [
  { marker: "~~", mark: _.Del },
  { marker: "**", mark: _.Strong },
  { marker: "`", mark: _.Code },
  { marker: "*", mark: _.Em }
];
function ne(t, e) {
  return e > 0 && t[e - 1] === "\\";
}
function yl(t, e, n) {
  const l = n[0];
  return e > 0 && t[e - 1] === l ? !0 : e + n.length < t.length && t[e + n.length] === l;
}
function wl(t, e) {
  if (e <= 0 || e > t.length) return null;
  for (const n of vl) {
    const l = n.marker, r = e - l.length;
    if (r < 0 || t.slice(r, e) !== l || ne(t, r)) continue;
    let i = -1;
    for (let s = r - l.length; s >= 0; s--)
      if (t[s] === l[0] && t.slice(s, s + l.length) === l && !ne(t, s) && !yl(t, s, l) && !t.slice(s + l.length, r).includes(l)) {
        i = s;
        break;
      }
    if (i < 0) continue;
    const o = r - (i + l.length);
    if (!(o <= 0) && !t.slice(i, e).includes(`
`))
      return { start: i, end: e, innerStart: i + l.length, innerLen: o, mark: n.mark, marker: l };
  }
  return null;
}
function _l(t, e, n) {
  const l = y(t, e);
  return l ? S(t, e, [Q(l, n(l.inlines))]) : t;
}
function Cl(t, e) {
  const n = y(t.doc, e);
  if (!n) return !1;
  const l = q(n), r = t.selection, i = r.anchor.blockId === e ? r.anchor.offset : l.length, o = wl(l, i);
  if (!o) return !1;
  {
    let u = 0;
    for (const f of n.inlines) {
      const p = u + f.text.length;
      if (u < o.end && p > o.start && f.marks.includes(_.Code)) return !1;
      u = p;
    }
  }
  const s = l.slice(o.innerStart, o.innerStart + o.innerLen), a = new j(new R(e, o.start), new R(e, o.end));
  t.applyGroup(
    [U.ReplaceRange(new xt(a, s))],
    (u) => _l(
      u,
      e,
      (f) => Vt(bl(f, o.start, o.start + o.innerLen, o.mark))
    )
  );
  const d = new R(e, o.start + o.innerLen);
  return t.select(new j(d, d)), !0;
}
function Ct(t, e) {
  for (; ; ) {
    const n = `${e}-${Math.random().toString(36).slice(2, 8)}`;
    if (!me(t, n)) return n;
  }
}
function W(t, e) {
  const n = y(t, e);
  if (!n) return null;
  const l = $(t, e);
  if (!l || l.kind !== c.ListItem) return null;
  const r = $(t, l.id);
  return !r || r.kind !== c.ListBlock ? null : { para: n, item: l, list: r, itemIndex: kt(r, l.id) };
}
function xl(t) {
  for (let e = t.children.length - 1; e >= 0; e--) {
    const n = t.children[e];
    if (n.kind === c.Paragraph) return { node: n, index: e };
  }
  return null;
}
function Il(t) {
  const e = [], n = $t(t.attrs, "ordered");
  n != null && e.push(Yt("ordered", n));
  const l = $t(t.attrs, "start");
  return l != null && e.push(Yt("start", l)), e;
}
function Be(t, e, n) {
  W(t.doc, e) && (t.applyTree((l) => {
    const r = W(l, e);
    if (!r) return l;
    const i = r.item.children.filter((a) => a.id !== e), o = [...r.list.children];
    i.length > 0 ? o[r.itemIndex] = C(r.item, i) : o.splice(r.itemIndex, 1);
    let s;
    if (o.length === 0) s = [r.para];
    else {
      const a = C(r.list, o);
      s = n === "after" ? [a, r.para] : [r.para, a];
    }
    return S(l, r.list.id, s);
  }), t.select(mt(e, 0)));
}
function Bl(t, e, n) {
  const l = W(t.doc, e);
  if (!l) return;
  if (q(l.para) === "") {
    Be(t, e, "after");
    return;
  }
  const r = Ct(t.doc, "b"), i = Ct(t.doc, "li");
  t.applyTree((o) => {
    const s = W(o, e);
    if (!s) return o;
    const a = pe(s.para.inlines, n), d = C(
      s.item,
      s.item.children.map((p) => p.id === e ? Q(s.para, a.before) : p)
    ), u = C(lt(i, c.ListItem), [
      Q(ct(r, c.Paragraph, ""), a.after)
    ]), f = [...s.list.children];
    return f.splice(s.itemIndex, 1, d, u), S(o, s.list.id, [C(s.list, f)]);
  }), t.select(mt(r, 0));
}
function Sl(t, e) {
  const n = W(t.doc, e);
  if (!n) return;
  if (n.itemIndex === 0) {
    Be(t, e, "before");
    return;
  }
  let l = "", r = 0;
  t.applyTree((i) => {
    const o = W(i, e);
    if (!o || o.itemIndex === 0) return i;
    const s = o.list.children[o.itemIndex - 1], a = o.item.children.filter((g) => g.id !== e), d = xl(s);
    let u;
    if (d) {
      l = d.node.id, r = q(d.node).length;
      const g = Q(d.node, [...d.node.inlines, ...o.para.inlines]);
      u = [...s.children.map((T, M) => M === d.index ? g : T), ...a];
    } else
      l = e, r = 0, u = [...s.children, o.para, ...a];
    const f = C(s, u), p = o.list.children.filter((g, T) => T !== o.itemIndex && T !== o.itemIndex - 1);
    return p.splice(o.itemIndex - 1, 0, f), S(i, o.list.id, [C(o.list, p)]);
  }), l && t.select(mt(l, r));
}
function Tl(t, e) {
  const n = W(t.doc, e);
  if (!n || n.itemIndex <= 0) return;
  const l = Ct(t.doc, "lb");
  t.applyTree((r) => {
    const i = W(r, e);
    if (!i || i.itemIndex <= 0) return r;
    const o = i.list.children[i.itemIndex - 1], a = [...o.children].reverse().find((p) => p.kind === c.ListBlock) ?? {
      ...lt(l, c.ListBlock),
      attrs: Il(i.list)
    }, d = o.children.filter((p) => p.id !== a.id), u = C(o, [...d, C(a, [...a.children, i.item])]), f = i.list.children.filter((p, g) => g !== i.itemIndex);
    return f.splice(i.itemIndex - 1, 1, u), S(r, i.list.id, [C(i.list, f)]);
  });
}
function Al(t, e) {
  const n = W(t.doc, e);
  if (!n) return;
  const l = $(t.doc, n.list.id);
  !l || l.kind !== c.ListItem || t.applyTree((r) => {
    const i = W(r, e);
    if (!i) return r;
    const o = $(r, i.list.id);
    if (!o || o.kind !== c.ListItem) return r;
    const s = $(r, o.id);
    if (!s) return r;
    const a = i.list.children.filter((p) => p.id !== i.item.id), d = o.children.filter((p) => p.id !== i.list.id), u = kt(s, o.id);
    let f;
    if (d.length > 0) {
      const p = a.length > 0 ? [...d, C(i.list, a)] : d, g = C(o, p);
      f = s.children.map((T, M) => M === u ? g : T), f.splice(u + 1, 0, i.item);
    } else
      f = s.children.filter((p, g) => g !== u), f.splice(u, 0, i.item);
    return S(r, s.id, [C(s, f)]);
  });
}
function Ml(t, e, n) {
  const l = y(t.doc, e), r = l ? $(t.doc, e) : null;
  if (!l || !r || r.kind !== c.Blockquote) return;
  if (q(l) === "") {
    Ol(t, e);
    return;
  }
  const i = Ct(t.doc, "b");
  t.applyTree((o) => {
    const s = y(o, e), a = s ? $(o, e) : null;
    if (!s || !a || a.kind !== c.Blockquote) return o;
    const d = pe(s.inlines, n), u = kt(a, e), f = [...a.children];
    return f[u] = Q(s, d.before), f.splice(u + 1, 0, Q(ct(i, c.Paragraph, ""), d.after)), S(o, a.id, [C(a, f)]);
  }), t.select(mt(i, 0));
}
function Ol(t, e) {
  const n = y(t.doc, e), l = n ? $(t.doc, e) : null;
  !n || !l || l.kind !== c.Blockquote || (t.applyTree((r) => {
    const i = y(r, e), o = i ? $(r, e) : null;
    if (!i || !o || o.kind !== c.Blockquote) return r;
    const s = o.children.filter((d) => d.id !== e), a = s.length === 0 ? [i] : [C(o, s), i];
    return S(r, o.id, a);
  }), t.select(mt(e, 0)));
}
function yt(t) {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function vt(t) {
  return yt(t).replace(/'/g, "&#39;");
}
function Zo(t) {
  let e = "";
  for (const n of t) {
    let l = yt(n.text);
    const r = A(n.attrs, "wikilink", "");
    if (r !== "") {
      const o = r.indexOf("#"), s = (o >= 0 ? r.slice(0, o) : r).trim();
      l = `<span class="autodown-wikilink-label" data-wikilink-title="${vt(s)}" contenteditable="false">${yt(r)}</span>`;
    }
    const i = A(n.attrs, "math_inline", "");
    if (i !== "" && (l = `<span class="autodown-math-inline" data-math-src="${vt(i)}" contenteditable="false">${yt(i)}</span>`), P(n.marks, _.Code) && (l = `<code>${l}</code>`), P(n.marks, _.Strong) && (l = `<strong>${l}</strong>`), P(n.marks, _.Em) && (l = `<em>${l}</em>`), P(n.marks, _.Underline) && (l = `<u>${l}</u>`), P(n.marks, _.Del) && (l = `<del>${l}</del>`), P(n.marks, _.Link)) {
      const o = vt(A(n.attrs, "href", "")), s = A(n.attrs, "title", ""), a = s ? ` title="${vt(s)}"` : "";
      l = `<a href="${o}"${a} contenteditable="false" data-autodown-link>${l}</a>`;
    }
    e += l;
  }
  return e.replace(/ +$/, (n) => "&nbsp;".repeat(n.length));
}
function Ll(t) {
  const e = t.toUpperCase();
  return e === "STRONG" || e === "B" ? _.Strong : e === "EM" || e === "I" ? _.Em : e === "DEL" || e === "S" ? _.Del : e === "U" ? _.Underline : e === "CODE" ? _.Code : null;
}
function qt(t) {
  let e = "";
  for (const n of t ?? [])
    n.text !== void 0 ? e += n.text.replace(/\u00A0/g, " ") : e += qt(n.children);
  return e;
}
function Rl(t) {
  const e = [], n = (l, r, i) => {
    var u, f, p;
    if (l.text !== void 0) {
      const g = l.text.replace(/\u00A0/g, " ");
      g !== "" && e.push(new X(g, r, i));
      return;
    }
    const o = ((u = l.attrs) == null ? void 0 : u.class) ?? "";
    if (o.includes("autodown-wikilink-label")) {
      const g = qt(l.children);
      g !== "" && e.push(new X(g, r, [new bt("wikilink", L.Str(g))]));
      return;
    }
    if (o.includes("autodown-math-inline")) {
      const g = ((f = l.attrs) == null ? void 0 : f["data-math-src"]) ?? qt(l.children);
      g !== "" && e.push(new X(g, r, [new bt("math_inline", L.Str(g))]));
      return;
    }
    let s = r, a = i;
    const d = Ll(l.tag ?? "");
    d !== null && (s = Ht(s, d)), (l.tag ?? "").toUpperCase() === "A" && ((p = l.attrs) == null ? void 0 : p.href) !== void 0 && (s = Ht(s, _.Link), a = [new bt("href", L.Str(l.attrs.href))], l.attrs.title !== void 0 && a.push(new bt("title", L.Str(l.attrs.title))));
    for (const g of l.children ?? []) n(g, s, a);
  };
  return n(t, [], []), Vt(e);
}
function $l(t) {
  const e = (n) => {
    var u, f, p, g, T;
    if (n.nodeType === 3) return { text: n.textContent ?? "" };
    const l = n, r = {}, i = (u = l.getAttribute) == null ? void 0 : u.call(l, "href");
    i != null && (r.href = i);
    const o = (f = l.getAttribute) == null ? void 0 : f.call(l, "title");
    o != null && (r.title = o);
    const s = (p = l.getAttribute) == null ? void 0 : p.call(l, "class");
    s != null && (r.class = s);
    const a = (g = l.getAttribute) == null ? void 0 : g.call(l, "data-wikilink-title");
    a != null && (r["data-wikilink-title"] = a);
    const d = (T = l.getAttribute) == null ? void 0 : T.call(l, "data-math-src");
    return d != null && (r["data-math-src"] = d), { tag: l.tagName ?? "", children: Array.from(n.childNodes).map(e), attrs: r };
  };
  return Rl(e(t));
}
class ti {
  constructor(e, n) {
    this.composition = new ol(), this.engine = e, this.blockId = n;
    const l = y(e.doc, n);
    this.knownText = l ? q(l) : "";
  }
  get id() {
    return this.blockId;
  }
  get text() {
    return this.knownText;
  }
  /** The block's inline spans (rich host mount render — plan 024 P2T1). */
  get inlines() {
    const e = y(this.engine.doc, this.blockId);
    return e ? e.inlines : [];
  }
  /** Where the DOM caret should sit after a model-side rewrite: the engine
   *  selection when it targets this block (inline input rules park it after
   *  the mark), else null → the resync falls back to end-of-text. */
  desiredCaretOffset() {
    const e = this.engine.selection;
    return e.anchor.blockId === this.blockId ? e.anchor.offset : null;
  }
  /** The host was (re)rendered from the engine — re-sync the known text
   *  (history changes repaint the host). */
  syncFromModel() {
    const e = y(this.engine.doc, this.blockId);
    return this.knownText = e ? q(e) : "", this.knownText;
  }
  /** `input` DOM event outside composition: old→new text becomes one op. */
  onInput(e) {
    if (this.composition.composing) return null;
    const n = il(this.blockId, this.knownText, e);
    return this.knownText = e, n ? (this.engine.apply(n), hl(this.engine, this.blockId) || Cl(this.engine, this.blockId), this.syncFromModel(), n) : null;
  }
  /** Enter key at caret offset → split the block. Nested paragraphs dispatch
   *  on the parent kind first (plan 025 P1T3): a ListItem parent splits the
   *  ITEM, a Blockquote parent continues the quote; only top-level leaves
   *  take the bare SplitBlock path. */
  onEnter(e, n) {
    const l = $(this.engine.doc, this.blockId);
    if ((l == null ? void 0 : l.kind) === c.ListItem) {
      Bl(this.engine, this.blockId, e), this.syncFromModel();
      return;
    }
    if ((l == null ? void 0 : l.kind) === c.Blockquote) {
      Ml(this.engine, this.blockId, e), this.syncFromModel();
      return;
    }
    this.engine.apply(U.SplitBlock(new wn(new R(this.blockId, e), n))), this.knownText = "";
  }
  /** Backspace at offset 0 → merge with the previous sibling (if any). In a
   *  list item the structural command owns the semantics (merge into the
   *  previous ITEM / lift the first item out); elsewhere the merge target
   *  must be an editable leaf of the same container — a container sibling
   *  (nested list subtree) never merges. */
  onBackspaceAtStart(e) {
    const n = $(this.engine.doc, this.blockId);
    if ((n == null ? void 0 : n.kind) === c.ListItem)
      return Sl(this.engine, this.blockId), this.syncFromModel(), !0;
    if (!e) return !1;
    const l = y(this.engine.doc, e);
    return !l || !ft(l) ? !1 : (this.engine.apply(U.MergeBlocks(new _n(e, this.blockId))), this.syncFromModel(), !0);
  }
  /** Previous mergeable sibling of this block, resolved MODEL-side. The
   *  deployed host DOM nests the contenteditable inside per-block slot
   *  wrappers (node-slot > node-content), so a previousElementSibling lookup
   *  is always null and the DOM route never fired the merge. Returns null at
   *  the first child position or when the previous sibling is not an
   *  editable leaf (containers and attr-only blocks never merge) — the same
   *  guard onBackspaceAtStart applies, so a non-null result guarantees the
   *  op will land. */
  prevSiblingId() {
    const e = $(this.engine.doc, this.blockId);
    if (!e) return null;
    const n = e.children.findIndex((r) => r.id === this.blockId);
    if (n <= 0) return null;
    const l = e.children[n - 1];
    return ft(l) ? l.id : null;
  }
  /** Next mergeable sibling of this block, mirror of prevSiblingId. */
  nextSiblingId() {
    const e = $(this.engine.doc, this.blockId);
    if (!e) return null;
    const n = e.children.findIndex((r) => r.id === this.blockId);
    if (n < 0 || n + 1 >= e.children.length) return null;
    const l = e.children[n + 1];
    return ft(l) ? l.id : null;
  }
  /** Cross-block vertical navigation, ↑ at the host's first visual line:
   *  select the previous editable-leaf sibling with the caret at its END
   *  (the reactive remount then mounts that block's host, whose mount
   *  focus lands the caret at the block end — exactly the contract).
   *  Returns the target block id, or null when there is nothing to
   *  navigate to (first block / container siblings stay out of v1). */
  navigateUp() {
    const e = this.prevSiblingId();
    if (!e) return null;
    const n = y(this.engine.doc, e);
    if (!n) return null;
    const l = q(n).length, r = new j(new R(e, l), new R(e, l));
    return this.engine.select(r), e;
  }
  /** ↓ at the last visual line: select the next editable-leaf sibling with
   *  the caret at its START. mountHost defaults the caret to the block end,
   *  so the ext re-places the caret at offset 0 once the remount has run. */
  navigateDown() {
    const e = this.nextSiblingId();
    if (!e) return null;
    const n = new j(new R(e, 0), new R(e, 0));
    return this.engine.select(n), e;
  }
  /** Tab / Shift+Tab inside a list item → indent / outdent (plan 025 P1T3).
   *  Returns false (browser default) when the block is not in a list. */
  onTab(e) {
    const n = $(this.engine.doc, this.blockId);
    return (n == null ? void 0 : n.kind) !== c.ListItem ? !1 : (e ? Al(this.engine, this.blockId) : Tl(this.engine, this.blockId), this.syncFromModel(), !0);
  }
  // -- composition delegates ------------------------------------------------------
  compositionBegin(e, n) {
    this.composition.begin(this.blockId, e, n);
  }
  compositionUpdate(e) {
    this.composition.update(e);
  }
  compositionCommit(e) {
    const n = this.composition.commit(e);
    return n && (this.engine.apply(n, { coalesce: !1 }), this.syncFromModel()), n;
  }
  compositionCancel() {
    this.composition.cancel();
  }
  /** Markdown / multiline paste: parse to blocks and insert after this one
   *  (plan 018 目标 5 — paste is v1-mandatory; HTML paste degrades to
   *  text/plain per 待澄清 5). */
  onPasteMarkdown(e) {
    const n = Cn(e, !0), l = n.children.length > 0 ? n.children : [];
    if (l.length === 0) return;
    const r = this.engine.doc, i = r.children, o = i.findIndex((a) => a.id === this.blockId), s = [...i.slice(0, o + 1), ...l, ...i.slice(o + 1)];
    this.engine.applyTree(() => C(r, s)), this.syncFromModel();
  }
  // -- rich blur writeback (plan 024 P2T2) ----------------------------------------
  /** Focus-leave writeback of the rich host: DOM walk → spans → whole-block
   *  withInlines through applyTree — ONE undo step, CodeEditorBlock protocol.
   *  Returns true when a rewrite landed. */
  onRichBlur(e) {
    return this.commitRichSpans($l(e));
  }
  /** Headless core of onRichBlur (the walk itself is e2e-pinned). Blocks
   *  carrying Image marks are skipped: their marks are not rendered in the
   *  rich host, so a rewrite would silently drop them (v1 no-data-loss). */
  commitRichSpans(e) {
    const n = y(this.engine.doc, this.blockId);
    return !n || this.inlines.some((l) => P(l.marks, _.Image)) || Jt(n.inlines) === Jt(e) ? !1 : (this.engine.applyTree((l) => {
      const r = y(l, this.blockId);
      return r ? S(l, this.blockId, [Q(r, e)]) : l;
    }), this.syncFromModel(), !0);
  }
}
function ft(t) {
  return !(t.children.length !== 0 || t.kind === c.ThematicBreak || t.kind === c.Details || t.kind === c.Callout || t.kind === c.QueryBlock || t.kind === c.BlockEmbed);
}
function ei(t, e) {
  const n = /* @__PURE__ */ new Set();
  if (!e) return n;
  let l = e;
  for (; ; ) {
    const r = $(t, l);
    if (!r || r === t) break;
    n.add(r.id), l = r.id;
  }
  return n;
}
const Se = /* @__PURE__ */ new Set([
  c.Callout,
  c.Details,
  c.Blockquote,
  c.ListBlock
]);
function Te(t) {
  return Se.has(t.kind) ? !1 : Ie(c[t.kind]) != null || ft(t);
}
function Nl(t) {
  if (Te(t)) return t;
  for (const e of t.children) {
    const n = Nl(e);
    if (n) return n;
  }
  return null;
}
function Hl(t) {
  if (Te(t)) return t;
  for (let e = t.children.length - 1; e >= 0; e--) {
    const n = Hl(t.children[e]);
    if (n) return n;
  }
  return null;
}
const Pl = 1e3;
let ot = null;
function Dl(t) {
  ot = t;
}
function El() {
  ot = null;
}
function Ae(t) {
  const e = ot;
  return ot = null, !e || e.blockId !== t || Date.now() - e.at > Pl ? null : e;
}
function ni(t, e, n, l) {
  if (ot = null, t.button !== 0 || e === "" || !n) return;
  const r = n.getBoundingClientRect();
  ot = { blockId: e, at: Date.now(), dx: t.clientX - r.left, dy: t.clientY - r.top, cellId: l };
}
function ql(t) {
  return t ? t.querySelector(".node-content > *") ?? t : null;
}
function Fl(t, e) {
  var u;
  if (e.dx == null || e.dy == null) return !1;
  const n = t.ownerDocument, l = n.caretRangeFromPoint, r = n.caretPositionFromPoint, i = t.getBoundingClientRect(), o = i.left + e.dx, s = i.top + e.dy;
  let a = null;
  if (typeof l == "function")
    a = l.call(n, o, s);
  else if (typeof r == "function") {
    const f = r.call(n, o, s);
    if (f) {
      a = n.createRange();
      try {
        a.setStart(f.offsetNode, f.offset), a.collapse(!0);
      } catch {
        return !1;
      }
    }
  }
  if (!a || !t.contains(a.startContainer)) return !1;
  a.collapse(!0);
  const d = ((u = n.defaultView) == null ? void 0 : u.getSelection()) ?? null;
  return d ? (d.removeAllRanges(), d.addRange(a), !0) : !1;
}
function le(t) {
  return ft(t) && Ie(c[t.kind]) == null;
}
function Me(t, e = []) {
  for (const n of t.children)
    n.kind === c.ListItem || Se.has(n.kind) ? Me(n, e) : e.push(n);
  return e;
}
function Wl(t) {
  return Array.from(t.querySelectorAll(".node-slot")).filter(
    (e) => e.querySelector(".node-slot") == null
  );
}
function re(t, e, n) {
  var s, a;
  const l = t.ownerDocument, r = l.caretRangeFromPoint, i = l.caretPositionFromPoint;
  let o = null;
  return typeof r == "function" ? o = ((s = r.call(l, e, n)) == null ? void 0 : s.startContainer) ?? null : typeof i == "function" && (o = ((a = i.call(l, e, n)) == null ? void 0 : a.offsetNode) ?? null), o && t.contains(o) ? o : null;
}
function oe(t) {
  return t ? t.nodeType === Node.TEXT_NODE ? t.parentElement : t : null;
}
function Kl(t, e, n) {
  var o, s;
  const l = n.parentElement;
  if (!l) return "";
  const r = Array.from(t.querySelectorAll("tr")).indexOf(l), i = Array.from(l.children).indexOf(n);
  return ((s = (o = e.children[r]) == null ? void 0 : o.children[i]) == null ? void 0 : s.id) ?? "";
}
function li(t, e, n) {
  var u, f;
  const l = ql(e);
  if (!l) return { targetId: t.id, anchor: null };
  if (t.kind === c.Table) {
    const p = ((u = oe(re(l, n.clientX, n.clientY))) == null ? void 0 : u.closest("th,td")) ?? null, g = p ? Kl(l, t, p) : "";
    return g ? { targetId: t.id, anchor: p, cellId: g } : { targetId: t.id, anchor: null };
  }
  if (le(t)) return { targetId: t.id, anchor: l };
  const r = ((f = oe(re(l, n.clientX, n.clientY))) == null ? void 0 : f.closest(".node-slot")) ?? null;
  if (!r || r === l) return { targetId: "", anchor: null };
  const i = Wl(l), o = Me(t);
  if (i.length !== o.length) return { targetId: "", anchor: null };
  const s = i.indexOf(r);
  if (s < 0) return { targetId: "", anchor: null };
  const a = o[s], d = le(a) ? r.querySelector(".node-content > *") ?? r : null;
  return { targetId: a.id, anchor: d };
}
function Ul(t, e) {
  return Qn("highlight") ? (ye() ?? _e)(t, e) ?? "" : "";
}
function Bt(t) {
  return it(t);
}
function Vl(t, e) {
  const n = Ul(t, e);
  return n !== "" ? `<code translate="no" data-highlighted="${Oe(e)}">${n}</code>` : `<code translate="no">${it(t)}</code>`;
}
function Ql(t, e) {
  return t === "edit" ? e : void 0;
}
function Oe(t) {
  return it(t).replace(/"/g, "&quot;");
}
function it(t) {
  return t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function jl(t, e) {
  const l = (ye() ?? _e)(t, e);
  return l !== void 0 ? l : it(t);
}
function Gl(t, e) {
  const n = jl(t, e);
  return n !== it(t) ? `<code translate="no" data-highlighted="${Oe(e)}">${n}</code>` : `<code translate="no">${it(t)}</code>`;
}
function zl(t) {
  const e = Re(t);
  return e.endsWith(`
`) ? e.slice(0, -1) : e;
}
function Xl(t, e, n) {
  const l = t.ownerDocument;
  let r = null, i = 0;
  const o = l.caretRangeFromPoint, s = l.caretPositionFromPoint;
  if (typeof o == "function") {
    const u = o.call(l, e, n);
    if (!u) return null;
    r = u.startContainer, i = u.startOffset;
  } else if (typeof s == "function") {
    const u = s.call(l, e, n);
    if (!u) return null;
    r = u.offsetNode, i = u.offset;
  } else
    return null;
  const a = t.querySelector("code") ?? t, d = l.createRange();
  d.selectNodeContents(a);
  try {
    d.setEnd(r, i);
  } catch {
    return null;
  }
  return d.toString().length;
}
function Yl(t, e) {
  if (El(), t.button !== 0 || e === "") return;
  const n = t.currentTarget;
  if (!n) return;
  const l = Xl(n, t.clientX, t.clientY);
  l != null && Dl({ blockId: e, at: Date.now(), offset: l });
}
function Jl(t) {
  var e;
  return ((e = Ae(t)) == null ? void 0 : e.offset) ?? null;
}
function Zl(t, e, n) {
  if (!t || e) return;
  const l = t;
  l.focus();
  const r = l.value.length, i = typeof n == "number" && n >= 0 && n <= r ? Math.round(n) : r;
  l.setSelectionRange(i, i), Le(l);
}
function Le(t) {
  const e = t;
  e && (e.style.height = "auto", e.style.height = `${e.scrollHeight}px`);
}
function Lt(t, e) {
  const n = t;
  !n || !e || (e.style.height = n.style.height || `${n.offsetHeight}px`, e.scrollTop = n.scrollTop, e.scrollLeft = n.scrollLeft);
}
function tr(t) {
  return A((t == null ? void 0 : t.attrs) ?? [], "language", "");
}
function Re(t) {
  return q(t ?? { inlines: [] });
}
function er(t) {
  return V((t == null ? void 0 : t.attrs) ?? [], "loading", !1);
}
function $e(t) {
  return (t == null ? void 0 : t.readonly) === !0;
}
function Qt(t) {
  const e = t == null ? void 0 : t.blockId;
  return typeof e == "string" ? e : "";
}
function nr(t) {
  return (t == null ? void 0 : t.id) ?? "";
}
function lr(t) {
  if (t == null) return null;
  const e = t;
  return new zn(e.engine, e.blockId);
}
function ri(t, e) {
  return t === "edit" ? e : void 0;
}
function oi(t) {
  return t === "edit" ? void 0 : "";
}
const rr = ["data-language"], or = ["data-block-id"], ir = {
  key: 0,
  class: "autodown-stream-banner"
}, sr = { class: "code-block-header flex justify-between items-center" }, ar = {
  class: "code-header-trigger",
  "data-codeblock-language-badge": "",
  title: "切换语言",
  type: "button"
}, cr = { class: "code-header-title" }, ur = { class: "code-editor-stack" }, dr = ["innerHTML"], fr = ["disabled"], hr = { class: "code-block-header flex justify-between items-center" }, mr = {
  class: "code-header-trigger",
  "data-codeblock-language-badge": "",
  title: "切换语言",
  type: "button"
}, kr = { class: "code-header-title" }, pr = ["aria-busy", "data-language", "innerHTML"], gr = /* @__PURE__ */ at({
  __name: "CodeBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "AreaScroll", "Blur", "ViewClick"],
  setup(t, { emit: e }) {
    const n = t, l = ut(zl(n.node)), r = ut(lr(n.ctx)), i = ut(null), o = ut(null), s = k(() => n.mode === "edit"), a = k(() => tr(n.node)), d = k(() => Re(n.node)), u = k(() => $e(n.ctx)), f = k(() => Qt(n.ctx)), p = k(() => er(n.node)), g = k(() => s.value ? "code-block-container rounded-lg border autodown-codeblock-node" : p.value ? "code-block-container rounded-lg border autodown-block-placeholder is-loading" : "code-block-container rounded-lg border"), T = k(() => Ql(n.mode, a.value)), M = k(() => a.value ? a.value : "text"), Y = k(() => "language-" + M.value + " code-pre-fallback is-wrap"), pt = k(() => p.value ? "true" : "false"), gt = k(() => Vl(d.value, a.value)), St = k(() => Gl(l.value, a.value)), J = e;
    function Tt(w) {
      Le(o.value), Lt(o.value, i.value), J("AreaInput", w);
    }
    function At(w) {
      Lt(w.target, i.value), J("AreaScroll", w);
    }
    function E(w) {
      u.value || r.value.commitDraft(w.target.value), J("Blur", w);
    }
    function I(w) {
      Yl(w, nr(n.node)), J("ViewClick", w);
    }
    return Ft(() => {
      s.value && (Zl(o.value, u.value, Jl(f.value)), Lt(o.value, i.value));
    }), (w, x) => (m(), b("div", {
      class: D(g.value),
      "data-language": T.value
    }, [
      s.value ? (m(), b("div", {
        key: 0,
        class: D(["autodown-code-editor", { "is-readonly": u.value }]),
        "data-block-id": f.value,
        "data-node-type": "Fence"
      }, [
        u.value ? (m(), b("div", ir, [...x[5] || (x[5] = [
          h("span", null, "流式生成中", -1)
        ])])) : O("", !0),
        h("div", sr, [
          h("button", ar, [
            h("span", cr, [
              h("span", null, wt(a.value), 1)
            ]),
            x[6] || (x[6] = h("span", { class: "code-header-caret" }, [
              h("span", null, "▾")
            ], -1))
          ]),
          x[7] || (x[7] = zt('<div class="flex items-center gap-0.5" data-v-352e562d><button class="code-action-btn" data-codeblock-copy-btn="" title="复制" type="button" data-v-352e562d><span class="codeblock-copy-icon" data-v-352e562d></span></button><button class="code-action-btn" data-codeblock-expand-btn="" title="折叠" type="button" data-v-352e562d><span class="codeblock-expand-icon" data-v-352e562d></span></button></div>', 1))
        ]),
        h("div", ur, [
          h("pre", {
            class: "code-editor-highlight",
            "aria-hidden": "true",
            innerHTML: St.value,
            ref_key: "hl",
            ref: i
          }, null, 8, dr),
          Ke(h("textarea", {
            class: "code-editor-textarea",
            disabled: u.value,
            ref_key: "area",
            ref: o,
            spellcheck: "false",
            "onUpdate:modelValue": x[0] || (x[0] = (B) => l.value = B),
            onBlur: x[1] || (x[1] = (B) => E(B)),
            onInput: x[2] || (x[2] = (B) => Tt(B.target.value)),
            onScroll: x[3] || (x[3] = (B) => At(B))
          }, null, 40, fr), [
            [Ue, l.value]
          ])
        ])
      ], 10, or)) : O("", !0),
      s.value ? O("", !0) : (m(), b(H, { key: 1 }, [
        h("div", hr, [
          h("button", mr, [
            h("span", kr, [
              h("span", null, wt(a.value), 1)
            ]),
            x[8] || (x[8] = h("span", { class: "code-header-caret" }, [
              h("span", null, "▾")
            ], -1))
          ]),
          x[9] || (x[9] = zt('<div class="flex items-center gap-0.5" data-v-352e562d><button class="code-action-btn" data-codeblock-copy-btn="" title="复制" type="button" data-v-352e562d><span class="codeblock-copy-icon" data-v-352e562d></span></button><button class="code-action-btn" data-codeblock-expand-btn="" title="折叠" type="button" data-v-352e562d><span class="codeblock-expand-icon" data-v-352e562d></span></button></div>', 1))
        ]),
        h("pre", {
          class: D(Y.value),
          "aria-busy": pt.value,
          "data-language": a.value,
          innerHTML: gt.value,
          tabindex: "0",
          onClick: x[4] || (x[4] = (B) => I(B))
        }, null, 10, pr)
      ], 64))
    ], 10, rr));
  }
}), Ne = (t, e) => {
  const n = t.__vccOpts || t;
  for (const [l, r] of e)
    n[l] = r;
  return n;
}, br = /* @__PURE__ */ Ne(gr, [["__scopeId", "data-v-352e562d"]]);
function ii(t, e, n) {
  y(t.doc, e) && t.applyTree((l) => S(l, e, n.length > 0 ? n : [ct(e, c.Paragraph, "")]));
}
function si(t, e) {
  const n = t.selection.anchor.blockId;
  !n || !y(t.doc, n) || t.applyTree((l) => S(l, n, e));
}
function ai(t, e, n = 0) {
  y(t.doc, e) && t.select(new j(new R(e, n), new R(e, n)));
}
function ci(t, e, n) {
  t.applyTree((l) => vr(l, e, n));
}
function vr(t, e, n) {
  var u;
  const l = y(t, e);
  if (!l) return t;
  const r = ((u = l.children[0]) == null ? void 0 : u.children.length) ?? 1, i = `row-${Math.random().toString(36).slice(2, 8)}`, o = [];
  for (let f = 0; f < r; f++) o.push(ct(`${i}-c${f}`, c.TableCell, ""));
  const s = C(lt(i, c.TableRow), o), a = [...l.children], d = n == null ? 0 : kt(l, n) + 1;
  return a.splice(d < 0 ? a.length : d, 0, s), S(t, e, [C(l, a)]);
}
function ui(t, e) {
  t.applyTree((n) => S(n, e, []));
}
function di(t, e) {
  t.applyTree((n) => yr(n, e));
}
function yr(t, e) {
  const n = y(t, e);
  if (!n) return t;
  const l = n.children.map(
    (r) => C(r, [...r.children, ct(`${r.id}-nc`, c.TableCell, "")])
  );
  return S(t, e, [C(n, l)]);
}
function fi(t, e, n) {
  const l = y(t, e);
  if (!l) return t;
  const r = l.children.map((i) => {
    const o = [...i.children], s = Math.max(0, Math.min(n, o.length));
    return o.splice(s, 0, ct(`${i.id}-nc${s}`, c.TableCell, "")), C(i, o);
  });
  return S(t, e, [C(l, r)]);
}
function hi(t, e, n) {
  var i;
  const l = y(t, e);
  if (!l || (((i = l.children[0]) == null ? void 0 : i.children.length) ?? 0) <= 1) return t;
  const r = l.children.map((o) => C(o, o.children.filter((s, a) => a !== n)));
  return S(t, e, [C(l, r)]);
}
function mi(t, e) {
  t.applyTree((n) => {
    const l = y(n, e);
    if (!l) return n;
    const r = l.children.map((i) => C(i, i.children.slice(0, -1)));
    return S(n, e, [C(l, r)]);
  });
}
function ki(t, e, n) {
  t.applyTree((l) => {
    const r = $(l, e);
    if (!r) return l;
    const i = kt(r, e), o = i + n;
    if (o < 0 || o >= r.children.length) return l;
    const s = [...r.children], [a] = s.splice(i, 1);
    return s.splice(o, 0, a), S(l, r.id, [C(r, s)]);
  });
}
function jt(t, e, n) {
  t.applyTree((l) => {
    const r = y(l, e);
    if (!r) return l;
    let i = r;
    for (const o of n) i = { ...i, attrs: ke(i.attrs, o.key, o.value) };
    return S(l, e, [i]);
  });
}
function pi(t, e) {
  const n = y(t.doc, e.anchor.blockId);
  if (!n) return [];
  if (e.anchor.blockId !== e.head.blockId) return Et(n.inlines, e.anchor.offset, e.anchor.offset);
  const l = Math.min(e.anchor.offset, e.head.offset), r = Math.max(e.anchor.offset, e.head.offset);
  return Et(n.inlines, l, r);
}
function wr(t) {
  const e = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", n = typeof crypto < "u" ? crypto : void 0, l = () => {
    if (n != null && n.getRandomValues) {
      const r = new Uint32Array(1);
      return n.getRandomValues(r), r[0] % e.length;
    }
    return Math.floor(Math.random() * e.length);
  };
  for (let r = 0; r < 64; r++) {
    let i = "";
    for (let o = 0; o < 7; o++) i += e[l()];
    if (!t.has(i)) return i;
  }
  return `a${Date.now().toString(36).slice(-6)}`;
}
function gi(t, e) {
  const n = y(t.doc, e);
  if (!n) return null;
  const l = xn(n);
  if (l) return l;
  const r = /* @__PURE__ */ new Set(), i = (s) => {
    r.add(s.id), s.children.forEach(i);
  };
  i(t.doc);
  const o = wr(r);
  return t.applyTree((s) => In(s, e, o)), o;
}
const st = at({
  name: "BlockChildren",
  props: { children_slot: { type: Function, required: !0 } },
  setup(t) {
    return () => t.children_slot();
  }
});
function He(t, e, n) {
  const r = y(t.doc, e);
  return r ? A(r.attrs, n, "") : "";
}
function _r(t, e) {
  const n = t;
  n && (n.textContent = e);
}
function Cr(t) {
  return typeof document < "u" && document.activeElement === t;
}
function xr(t, e, n, l) {
  const r = t;
  r && !Cr(r) && (r.textContent = He(e, n, l));
}
function Ir(t, e, n, l, r) {
  if (r) return;
  const i = t, o = ((i == null ? void 0 : i.textContent) ?? "").replace(/\u00a0/g, " ").trim();
  o !== He(e, n, l) && jt(e, n, [{ key: l, value: L.Str(o) }]);
}
function ie(t) {
  const e = t;
  e && e.blur();
}
const Br = ["contenteditable", "data-placeholder", "onKeydown"], Sr = /* @__PURE__ */ at({
  __name: "AttrHost",
  props: {
    controller: {},
    blockId: {},
    attr_key: {},
    value: {},
    placeholder: {},
    host_class: {},
    readonly: { type: Boolean },
    version: {}
  },
  emits: ["Init", "KeyEnter", "KeyEscape", "Blur"],
  setup(t, { emit: e }) {
    const n = t, l = ut(null), r = k(() => !n.readonly), i = k(() => "autodown-attr-host " + n.host_class), o = e;
    Ve(() => n.version, () => {
      xr(l.value, n.controller, n.blockId, n.attr_key);
    });
    function s(u) {
      Ir(u.target, n.controller, n.blockId, n.attr_key, n.readonly), o("Blur", u);
    }
    function a() {
      ie(l.value), o("KeyEnter");
    }
    function d() {
      ie(l.value), o("KeyEscape");
    }
    return Ft(() => {
      _r(l.value, n.value);
    }), (u, f) => (m(), b("span", {
      class: D(i.value),
      contenteditable: r.value,
      "data-placeholder": t.placeholder,
      ref_key: "host",
      ref: l,
      spellcheck: "false",
      onBlur: f[0] || (f[0] = (p) => s(p)),
      onKeydown: [
        Xt(Rt(a, ["prevent"]), ["enter"]),
        Xt(Rt(d, ["prevent"]), ["esc"])
      ]
    }, null, 42, Br));
  }
});
function Pe(t) {
  return (t == null ? void 0 : t.engine) ?? null;
}
function se(t, e) {
  return t ? A(t.attrs, e, "") : "";
}
function Tr(t) {
  return On.includes(t);
}
function Ar(t, e) {
  return t ? V(t.attrs, e, !1) : !1;
}
function Mr(t, e) {
  return t ? Nt(t.attrs, e, 1) : 1;
}
function Or(t, e, n) {
  return t === "edit" && e ? n : void 0;
}
function Lr(t, e) {
  const n = t;
  if (!n || !e) return;
  const l = y(n.doc, e);
  if (!l) return;
  const r = V(l.attrs, "checked", !1);
  jt(n, e, [{ key: "checked", value: L.Bool(!r) }]);
}
function bi(t, e) {
  const n = Qt(e);
  return n || ((t == null ? void 0 : t.id) ?? "");
}
function vi(t, e, n) {
  const l = t;
  !l || !e || jt(l, e, [{ key: "open", value: L.Bool(!n) }]);
}
const Rr = ["data-callout-type"], $r = {
  key: 0,
  class: "autodown-stream-banner"
}, Nr = { class: "autodown-callout-header" }, Hr = ["innerHTML"], Pr = { class: "autodown-callout-content" }, Dr = {
  key: 0,
  class: "markdown-renderer"
}, Er = /* @__PURE__ */ at({
  __name: "CalloutBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    children: {},
    version: {}
  },
  setup(t) {
    const e = t, n = k(() => e.mode === "edit"), l = k(() => $e(e.ctx)), r = k(() => Qt(e.ctx)), i = k(() => Pe(e.ctx)), o = k(() => se(e.node, "type")), s = k(() => Tr(o.value)), a = k(() => se(e.node, "title")), d = k(() => a.value ? a.value : o.value), u = k(() => Bt(d.value)), f = k(() => o.value ? o.value : "标题"), p = k(() => "callout-node autodown-callout autodown-callout-" + o.value), g = k(() => "autodown-callout-icon autodown-callout-icon-" + o.value);
    return (T, M) => (m(), b("div", {
      class: D(p.value),
      "data-callout-type": o.value
    }, [
      n.value ? (m(), b(H, { key: 0 }, [
        l.value ? (m(), b("div", $r, [...M[0] || (M[0] = [
          h("span", null, "流式生成中", -1)
        ])])) : O("", !0)
      ], 64)) : O("", !0),
      h("div", Nr, [
        s.value ? (m(), b("span", {
          key: 0,
          class: D(g.value),
          "aria-hidden": "true"
        }, null, 2)) : O("", !0),
        n.value ? (m(), K(z(Sr), {
          attr_key: "title",
          blockId: r.value,
          controller: i.value,
          host_class: "autodown-callout-title",
          placeholder: f.value,
          readonly: l.value,
          value: a.value,
          version: t.version,
          key: "AttrHost-1"
        }, null, 8, ["blockId", "controller", "placeholder", "readonly", "value", "version"])) : O("", !0),
        n.value ? O("", !0) : (m(), b("div", {
          key: 2,
          class: "autodown-callout-title",
          dir: "auto",
          innerHTML: u.value
        }, null, 8, Hr))
      ]),
      h("div", Pr, [
        n.value ? (m(), b("div", Dr, [
          (m(), K(z(st), {
            children_slot: t.children,
            key: "BlockChildren-2"
          }, null, 8, ["children_slot"]))
        ])) : O("", !0),
        n.value ? O("", !0) : (m(), K(z(st), {
          children_slot: t.children,
          key: "BlockChildren-3"
        }, null, 8, ["children_slot"]))
      ])
    ], 10, Rr));
  }
}), qr = ["aria-label", "checked", "disabled", "onClick"], Fr = {
  key: 1,
  class: "markdown-renderer"
}, Wr = /* @__PURE__ */ at({
  __name: "ListBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    items: {},
    version: {}
  },
  emits: ["TaskClick"],
  setup(t, { emit: e }) {
    const n = t, l = k(() => n.mode === "edit"), r = k(() => Pe(n.ctx)), i = k(() => Ar(n.node, "ordered")), o = k(() => i.value ? "ol" : "ul"), s = k(() => i.value ? "list-node list-decimal" : "list-node list-disc"), a = k(() => Or(n.mode, i.value, Mr(n.node, "start"))), d = k(() => !l.value), u = k(() => l.value ? "toggle task" : "task checkbox"), f = e;
    function p(g) {
      Lr(r.value, g.id), f("TaskClick", g);
    }
    return (g, T) => (m(), K(ae(o.value), {
      class: D(s.value),
      start: a.value
    }, {
      default: ce(() => [
        (m(!0), b(H, null, F(t.items, (M, Y) => (m(), b("li", {
          class: D(M.cls),
          dir: "auto",
          key: M.id
        }, [
          M.task ? (m(), b("input", {
            key: 0,
            class: "task-checkbox",
            "aria-label": u.value,
            checked: M.checked,
            disabled: d.value,
            type: "checkbox",
            onClick: Rt((pt) => p(M), ["stop"])
          }, null, 8, qr)) : O("", !0),
          l.value ? (m(), b("div", Fr, [
            (m(), K(z(st), {
              children_slot: M.children_slot,
              key: "BlockChildren-1-" + Y
            }, null, 8, ["children_slot"]))
          ])) : O("", !0),
          l.value ? O("", !0) : (m(), K(z(st), {
            children_slot: M.children_slot,
            key: "BlockChildren-2-" + Y
          }, null, 8, ["children_slot"]))
        ], 2))), 128))
      ]),
      _: 1
    }, 8, ["class", "start"]));
  }
});
function Kr(t, e) {
  var r;
  const n = e == null ? void 0 : e.target, l = (r = n == null ? void 0 : n.dataset) == null ? void 0 : r.cellId;
  l && t.commitCell(l, n.innerText.replace(/\n+$/, ""));
}
function Ur(t) {
  var a, d;
  const e = Qe(), n = ((a = e == null ? void 0 : e.proxy) == null ? void 0 : a.$el) ?? null;
  if (!n || t === "") return;
  const l = Ae(t);
  if (!l || !l.cellId) return;
  const r = Array.from(n.querySelectorAll("[data-cell-id]")).find(
    (u) => u.dataset.cellId === l.cellId
  );
  if (!r || (r.focus(), Fl(r, l))) return;
  const i = r.ownerDocument, o = i.createRange();
  o.selectNodeContents(r), o.collapse(!1);
  const s = ((d = i.defaultView) == null ? void 0 : d.getSelection()) ?? null;
  s == null || s.removeAllRanges(), s == null || s.addRange(o);
}
function Vr(t) {
  return t === "view" ? "table" : "div";
}
function Qr(t, e, n) {
  return t === "view" ? "table-node" : t === "stream" ? e ? "streaming-table final" : "streaming-table" : n ? "autodown-table-editor is-readonly" : "autodown-table-editor";
}
function jr(t) {
  return t === "view" ? "false" : void 0;
}
function Gr(t, e) {
  return t === "edit" ? e : void 0;
}
function zr(t) {
  return t === "edit" ? "Table" : void 0;
}
function Xr(t) {
  return (t ?? []).map((n) => ({ col: String(n), html: Bt(String(n)) }));
}
function Yr(t, e) {
  const n = t ?? [];
  return (e ?? []).map(
    (r) => n.map((i) => ({ col: String(i), html: Bt(String((r == null ? void 0 : r[i]) ?? "")) }))
  );
}
function Jr(t) {
  const e = (t ?? []).length;
  return Math.max(1, e);
}
const Zr = {
  key: 0,
  class: "autodown-stream-banner"
}, to = {
  class: "te-toolbar",
  "aria-label": "表格工具栏",
  role: "toolbar"
}, eo = ["disabled"], no = ["disabled"], lo = ["disabled"], ro = ["disabled"], oo = ["disabled"], io = ["disabled"], so = ["disabled"], ao = {
  class: "table-node",
  "aria-busy": "false"
}, co = ["contenteditable", "data-cell-id"], uo = ["contenteditable", "data-cell-id"], fo = { key: 2 }, ho = ["innerHTML"], mo = ["innerHTML"], ko = {
  key: 0,
  class: "loading-row"
}, po = ["colspan"], go = ["innerHTML"], bo = /* @__PURE__ */ at({
  __name: "TableBlockWidget",
  props: {
    mode: {},
    controller: {},
    blockId: {},
    readonly: { type: Boolean },
    final: { type: Boolean },
    header_cells: {},
    body_rows: {},
    columns: {},
    rows: {}
  },
  emits: ["Init", "AddRowAbove", "AddRow", "DeleteRow", "AddColumnBefore", "AddColumn", "DeleteColumn", "DeleteTable", "CellBlur"],
  setup(t, { emit: e }) {
    const n = t, l = k(() => n.mode === "edit"), r = k(() => n.mode === "view"), i = k(() => Vr(n.mode)), o = k(() => Qr(n.mode, n.final, n.readonly)), s = k(() => jr(n.mode)), a = k(() => Gr(n.mode, n.blockId)), d = k(() => zr(n.mode)), u = k(() => Xr(n.columns)), f = k(() => Yr(n.columns, n.rows)), p = k(() => Jr(n.columns)), g = k(() => Bt("Loading")), T = e;
    function M() {
      n.controller.addColumn(), T("AddColumn");
    }
    function Y() {
      n.controller.addRow(), T("AddRow");
    }
    function pt() {
      n.controller.addRowAbove(), T("AddRowAbove");
    }
    function gt(E) {
      if (!n.readonly) {
        let I = n.controller;
        Kr(I, E);
      }
      T("CellBlur", E);
    }
    function St() {
      n.controller.deleteColumn(), T("DeleteColumn");
    }
    function J() {
      n.controller.deleteRow(), T("DeleteRow");
    }
    function Tt() {
      n.controller.deleteTable(), T("DeleteTable");
    }
    function At() {
      T("AddColumnBefore");
    }
    return Ft(() => {
      Ur(n.blockId);
    }), (E, I) => (m(), K(ae(i.value), {
      class: D(o.value),
      "aria-busy": s.value,
      "data-block-id": a.value,
      "data-node-type": d.value
    }, {
      default: ce(() => [
        l.value ? (m(), b(H, { key: 0 }, [
          t.readonly ? (m(), b("div", Zr, [...I[2] || (I[2] = [
            h("span", null, "流式生成中", -1)
          ])])) : O("", !0),
          h("div", to, [
            h("button", {
              class: "te-btn",
              "data-te-action": "add-row-above",
              disabled: t.readonly,
              title: "在上方插入一行",
              type: "button",
              onClick: pt
            }, [...I[3] || (I[3] = [
              h("span", null, "行↑", -1)
            ])], 8, eo),
            h("button", {
              class: "te-btn",
              "data-te-action": "add-row",
              disabled: t.readonly,
              title: "在末尾后插入一行",
              type: "button",
              onClick: Y
            }, [...I[4] || (I[4] = [
              h("span", null, "行↓", -1)
            ])], 8, no),
            h("button", {
              class: "te-btn",
              "data-te-action": "delete-row",
              disabled: t.readonly,
              title: "删除最后一行",
              type: "button",
              onClick: J
            }, [...I[5] || (I[5] = [
              h("span", null, "删行", -1)
            ])], 8, lo),
            h("button", {
              class: "te-btn",
              "data-te-action": "add-col-before",
              disabled: t.readonly,
              title: "在左侧插入一列",
              type: "button",
              onClick: At
            }, [...I[6] || (I[6] = [
              h("span", null, "列←", -1)
            ])], 8, ro),
            h("button", {
              class: "te-btn",
              "data-te-action": "add-col",
              disabled: t.readonly,
              title: "追加一列",
              type: "button",
              onClick: M
            }, [...I[7] || (I[7] = [
              h("span", null, "列→", -1)
            ])], 8, oo),
            h("button", {
              class: "te-btn",
              "data-te-action": "delete-col",
              disabled: t.readonly,
              title: "删除最后一列",
              type: "button",
              onClick: St
            }, [...I[8] || (I[8] = [
              h("span", null, "删列", -1)
            ])], 8, io),
            h("button", {
              class: "te-btn te-btn-danger",
              "data-te-action": "delete-table",
              disabled: t.readonly,
              title: "删除整个表格",
              type: "button",
              onClick: Tt
            }, [...I[9] || (I[9] = [
              h("span", null, "删表", -1)
            ])], 8, so)
          ]),
          h("table", ao, [
            h("thead", null, [
              h("tr", null, [
                (m(!0), b(H, null, F(t.header_cells, (w, x) => (m(), b("th", {
                  class: D(w.cls),
                  contenteditable: t.readonly == !1,
                  "data-cell-id": w.id,
                  dir: "auto",
                  key: w.id,
                  spellcheck: "false",
                  onBlur: I[0] || (I[0] = (B) => gt(B))
                }, [
                  h("span", null, wt(w.text), 1)
                ], 42, co))), 128))
              ])
            ]),
            h("tbody", null, [
              (m(!0), b(H, null, F(t.body_rows, (w, x) => (m(), b("tr", {
                key: w.id
              }, [
                (m(!0), b(H, null, F(w.cells, (B, Mt) => (m(), b("td", {
                  class: D(B.cls),
                  contenteditable: t.readonly == !1,
                  "data-cell-id": B.id,
                  dir: "auto",
                  key: B.id,
                  spellcheck: "false",
                  onBlur: I[1] || (I[1] = (We) => gt(We))
                }, [
                  h("span", null, wt(B.text), 1)
                ], 42, uo))), 128))
              ]))), 128))
            ])
          ])
        ], 64)) : O("", !0),
        r.value ? (m(), b(H, { key: 1 }, [
          h("thead", null, [
            h("tr", null, [
              (m(!0), b(H, null, F(t.header_cells, (w, x) => (m(), b("th", {
                class: D(w.cls),
                dir: "auto",
                key: w.id
              }, [
                (m(), K(z(st), {
                  children_slot: w.children_slot,
                  key: "BlockChildren-1-" + x
                }, null, 8, ["children_slot"])),
                I[10] || (I[10] = h("button", {
                  class: "table-node__resize-handle",
                  type: "button"
                }, null, -1))
              ], 2))), 128))
            ])
          ]),
          h("tbody", null, [
            (m(!0), b(H, null, F(t.body_rows, (w, x) => (m(), b("tr", {
              key: w.id
            }, [
              (m(!0), b(H, null, F(w.cells, (B, Mt) => (m(), b("td", {
                class: D(B.cls),
                dir: "auto",
                key: B.id
              }, [
                (m(), K(z(st), {
                  children_slot: B.children_slot,
                  key: "BlockChildren-2-" + Mt
                }, null, 8, ["children_slot"]))
              ], 2))), 128))
            ]))), 128))
          ])
        ], 64)) : O("", !0),
        t.mode == "stream" ? (m(), b("table", fo, [
          h("thead", null, [
            h("tr", null, [
              (m(!0), b(H, null, F(u.value, (w, x) => (m(), b("th", {
                innerHTML: w.html,
                key: w.col
              }, null, 8, ho))), 128))
            ])
          ]),
          h("tbody", null, [
            (m(!0), b(H, null, F(f.value, (w, x) => (m(), b("tr", { key: x }, [
              (m(!0), b(H, null, F(w, (B, Mt) => (m(), b("td", {
                innerHTML: B.html,
                key: B.col
              }, null, 8, mo))), 128))
            ]))), 128)),
            t.final ? O("", !0) : (m(), b("tr", ko, [
              h("td", { colspan: p.value }, [
                h("span", {
                  class: "loading-dots",
                  innerHTML: g.value
                }, null, 8, go)
              ], 8, po)
            ]))
          ])
        ])) : O("", !0)
      ]),
      _: 1
    }, 8, ["class", "aria-busy", "data-block-id", "data-node-type"]));
  }
}), De = /* @__PURE__ */ Ne(bo, [["__scopeId", "data-v-4712a9a5"]]);
function yi(t, e, n = {}) {
  nl(t, {
    view: n.view ?? ((l, r) => v(e, { mode: "view", node: l, final: r, ctx: null })),
    stream: n.stream ?? ((l, r) => v(e, { mode: "stream", node: l, final: r, ctx: null })),
    edit: n.edit ?? ((l, r) => v(e, { mode: "edit", node: l, ctx: r }))
  });
}
function wi(t) {
  ll(t);
}
function vo(t) {
  return (e) => {
    const n = xe(e.node) ?? yo(e.node);
    return v(t, { mode: "view", node: n, final: e.final ?? !0, ctx: null });
  };
}
function yo(t) {
  const e = [], n = typeof (t == null ? void 0 : t.code) == "string" ? t.code : "";
  return (t == null ? void 0 : t.type) === "code_block" && (e.push({ key: "language", value: L.Str(String(t.language ?? "")) }), (t == null ? void 0 : t.loading) === !0 && e.push({ key: "loading", value: L.Bool(!0) })), (t == null ? void 0 : t.type) === "query" && e.push({ key: "query", value: L.Str(String(t.content ?? "")) }), (t == null ? void 0 : t.type) === "embed" && e.push({ key: "src", value: L.Str(String(t.src ?? "")) }), {
    id: "nv",
    kind: wo((t == null ? void 0 : t.type) ?? ""),
    attrs: e,
    children: [],
    inlines: n.length > 0 ? [de(n)] : [],
    source: { start: 0, end: 0 }
  };
}
function wo(t) {
  return t === "code_block" ? c.Fence : t === "mermaid" ? c.Mermaid : t === "query" ? c.QueryBlock : t === "embed" ? c.BlockEmbed : c.MathBlock;
}
function _o(t) {
  const e = [];
  return (t == null ? void 0 : t.type) === "callout" && (e.push({ key: "type", value: L.Str(String(t.language ?? "")) }), e.push({ key: "title", value: L.Str(String(t.title ?? "")) })), (t == null ? void 0 : t.type) === "details" && (e.push({ key: "summary", value: L.Str(String(t.text ?? "")) }), (t == null ? void 0 : t.loading) === !0 && e.push({ key: "open", value: L.Bool(!0) })), (t == null ? void 0 : t.type) === "list" && (e.push({ key: "ordered", value: L.Bool(t.ordered === !0) }), e.push({ key: "start", value: L.Int(typeof t.start == "number" ? t.start : 1) })), {
    id: "nv",
    kind: c.Paragraph,
    attrs: e,
    children: [],
    inlines: [],
    source: { start: 0, end: 0 }
  };
}
function Ee(t) {
  return xe(t) ?? _o(t);
}
function Co(t) {
  return (e) => {
    const n = e.final ?? !0, l = Wt();
    return v(t, {
      mode: "view",
      node: Ee(e.node),
      final: n,
      ctx: null,
      children: Gt(l, () => e.renderEmbedded(e.node.children ?? [], n, e.budget)),
      version: 0
    });
  };
}
function xo(t) {
  const e = t.final ?? !0, n = Wt();
  return (t.node.items ?? []).map((l, r) => ({
    id: `li-${r}`,
    task: l.checked != null,
    checked: l.checked === !0,
    cls: "list-item" + (l.checked != null ? " task-item" : ""),
    children_slot: Gt(n, () => t.renderEmbedded(l.children ?? [], e, t.budget))
  }));
}
function Io(t) {
  return (t == null ? void 0 : t.align) === "center" ? "text-center" : (t == null ? void 0 : t.align) === "right" ? "text-right" : "text-left";
}
function qe(t, e) {
  const n = t.final ?? !0, l = Wt();
  return (e ?? []).map((r, i) => ({
    id: `cell-${i}`,
    cls: Io(r),
    children_slot: Gt(l, () => t.renderInlineChildren(r.children ?? [], n, t.budget))
  }));
}
function Bo(t) {
  var e, n;
  return qe(t, ((n = (e = t.node.header) == null ? void 0 : e[0]) == null ? void 0 : n.cells) ?? []);
}
function So(t) {
  return (t.node.rows ?? []).map((e, n) => ({
    id: `tr-${n}`,
    cells: qe(t, e.cells)
  }));
}
function Gt(t, e) {
  return () => {
    const n = e();
    return Array.isArray(n) ? n : [n];
  };
}
function _i(t, e) {
  return v(De, {
    mode: "stream",
    controller: null,
    blockId: "",
    readonly: !0,
    final: e,
    header_cells: [],
    body_rows: [],
    columns: (t == null ? void 0 : t.columns) ?? [],
    rows: (t == null ? void 0 : t.rows) ?? []
  });
}
It("Codeblock", vo(br));
It("Callout", Co(Er));
It(
  "List",
  (t) => v(Wr, {
    mode: "view",
    node: Ee(t.node),
    final: t.final ?? !0,
    ctx: null,
    items: xo(t),
    version: 0
  })
);
It(
  "Table",
  (t) => v(De, {
    mode: "view",
    final: t.final ?? !0,
    ctx: null,
    // filler values for the generated required-prop checks (the 033
    // ctx:null idiom): the view face reads none of these
    controller: null,
    blockId: "",
    readonly: !1,
    columns: [],
    rows: [],
    header_cells: Bo(t),
    body_rows: So(t)
  })
);
function To(t, e, n) {
  const l = n !== void 0 && Number.isFinite(n) ? { remaining: n } : void 0;
  return (t ?? []).map((r, i) => {
    const o = i === t.length - 1;
    return Fe(r, i, e, o ? l : void 0);
  });
}
function Fe(t, e, n, l) {
  return v("div", { class: "node-slot", "data-node-index": String(e), "data-node-type": t.type }, [
    v("div", { class: "node-content" }, [Ro(t, n, l)])
  ]);
}
function Ao(t, e, n) {
  const l = (t ?? []).map((r, i) => {
    const o = i === ((t == null ? void 0 : t.length) ?? 0) - 1;
    return Fe(r, i, e, o ? n : void 0);
  });
  return v("div", { class: "markdown-renderer" }, l);
}
function et(t, e, n) {
  return (t ?? []).map((l) => Lo(l, e, n));
}
function Mo(t) {
  return t.content ?? t.code ?? "";
}
function Oo(t, e) {
  if (!e) return t;
  if (e.remaining <= 0) return "";
  const n = e.remaining >= t.length ? t : t.slice(0, e.remaining);
  return e.remaining -= n.length, n;
}
function Lo(t, e, n) {
  switch (t.type) {
    case "text":
      return v("span", { class: "whitespace-pre-wrap break-words text-node" }, [v("span", Oo(t.content, n))]);
    case "strong":
      return v("strong", { class: "strong-node" }, et(t.children, e, n));
    case "emphasis":
      return v("em", { class: "emphasis-node" }, et(t.children, e, n));
    case "underline":
      return v("u", { class: "underline-node" }, et(t.children, e, n));
    case "strikethrough":
      return v("del", { class: "strikethrough-node" }, et(t.children, e, n));
    case "inline_code":
      return v("code", { class: "inline-code" }, [v("span", t.code)]);
    case "link":
      return v(
        "a",
        {
          class: "link-node",
          href: t.href,
          title: t.title ?? void 0,
          target: "_blank",
          rel: "noopener noreferrer"
        },
        et(t.children, e, n)
      );
    case "image":
      return v("span", { class: "image-node-container" }, [
        v("img", {
          src: t.src,
          alt: t.alt,
          title: t.alt,
          class: "image-node__img",
          loading: "lazy"
        })
      ]);
    case "hardbreak":
      return v("br");
    case "math_inline": {
      const l = t.code ?? "", r = Gn(l, !1);
      return r.error === "" ? v("span", { class: "autodown-math-inline", "data-math-src": l }, [
        v("span", { class: "math-inline-render", innerHTML: r.html })
      ]) : v(
        "span",
        {
          class: "autodown-math-inline autodown-math-error",
          "data-math-src": l,
          title: r.error
        },
        [l]
      );
    }
    case "wikilink": {
      const l = t.title ?? "", r = l.indexOf("#"), i = (r >= 0 ? l.slice(0, r) : l).trim(), o = r >= 0 ? l.slice(r + 1).trim() : void 0, s = o ? `${i}#${o}` : i;
      return v(
        "span",
        {
          class: "autodown-wikilink-label",
          "data-wikilink-title": i,
          onClick: (a) => {
            a.stopPropagation(), Hn(i, o);
          }
        },
        s
      );
    }
    default:
      return v("span", { class: "whitespace-pre-wrap break-words text-node" }, [
        v("span", Mo(t))
      ]);
  }
}
function Ro(t, e, n) {
  const l = Rn(t), r = $n(l);
  return r ? r({ node: t, final: e, budget: n, spec: l, renderEmbedded: Ao, renderInlineChildren: et }) : v("div", { class: "unknown-node" }, String(t.type));
}
export {
  Pe as $,
  ve as A,
  ti as B,
  Jo as C,
  ll as D,
  wi as E,
  To as F,
  _i as G,
  pi as H,
  hi as I,
  fi as J,
  vr as K,
  Ae as L,
  Fl as M,
  Zo as N,
  gi as O,
  il as P,
  Gn as Q,
  zo as R,
  Re as S,
  lr as T,
  $e as U,
  oi as V,
  ri as W,
  Qt as X,
  Zl as Y,
  Go as Z,
  Ne as _,
  ci as a,
  bi as a0,
  Ar as a1,
  se as a2,
  Sr as a3,
  st as a4,
  vi as a5,
  xe as a6,
  Gt as a7,
  ei as a8,
  ft as a9,
  Xo as aa,
  li as ab,
  ni as ac,
  Nl as ad,
  Hl as ae,
  br as af,
  Er as ag,
  Wr as ah,
  De as ai,
  Wt as aj,
  Wo as ak,
  Ko as al,
  mi as b,
  ui as c,
  jo as d,
  Fo as e,
  ai as f,
  Qo as g,
  Uo as h,
  ii as i,
  Vo as j,
  Qn as k,
  It as l,
  ki as m,
  Ut as n,
  Yo as o,
  Ie as p,
  ye as q,
  si as r,
  jt as s,
  di as t,
  qo as u,
  _e as v,
  vo as w,
  nl as x,
  yi as y,
  rl as z
};
