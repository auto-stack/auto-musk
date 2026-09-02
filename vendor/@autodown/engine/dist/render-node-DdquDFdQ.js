import { h, defineComponent as J, ref as X, computed as u, onMounted as Re, openBlock as i, createElementBlock as d, normalizeClass as M, createElementVNode as s, createCommentVNode as v, toDisplayString as oe, createStaticVNode as xe, withDirectives as st, vModelText as it, Fragment as T, watch as ut, withKeys as Te, withModifiers as pe, createBlock as D, unref as q, resolveDynamicComponent as Oe, withCtx as Ee, renderList as O } from "vue";
import ct from "katex";
import De from "mermaid";
import { createLowlight as dt, common as ft } from "lowlight";
import { toHtml as ht } from "hast-util-to-html";
import { f as _, r as L, w as mt, a as Pe, b as We, c as y, d as P, e as Z, h as U, I as pt, g as kt, S as bt, B as Ae, l as ae, i as k, j as gt, k as qe, m as I, n as vt, o as yt, q as _t, V as S, t as ke, M as $, u as Ct, W as K, v as wt, x as Bt, y as xt, z as Tt, A as At, C as St, D as Mt, E as $t, F as Nt, G as Ht, H as It, J as Lt, K as Rt, L as Ot, N as Fe, O as Et, P as Dt, Q as Pt, R as Wt, T as qt, U as Ft } from "./markdown-parser-0FkmfLuR.js";
function w(e, t, n, l, o) {
  return { kind: e, tag: t, class_token: n, registry: l, extension: o };
}
function Ke(e) {
  let t = e;
  return t < 1 && (t = 1), t > 6 && (t = 6), w("H" + String(t), "h" + String(t), "heading-node", "Heading", !1);
}
function Kt(e) {
  return e == "paragraph" ? w("Text", "p", "paragraph-node", "Text", !1) : e == "text" ? w("Text", "span", "text-node", "Text", !1) : e == "heading" ? Ke(1) : e == "thematic_break" ? w("Separator", "hr", "hr-node", "Separator", !1) : e == "code_block" ? w("Codeblock", "div", "code-block-container", "Codeblock", !1) : e == "blockquote" ? w("Quote", "blockquote", "blockquote", "Quote", !1) : e == "list" ? w("List", "ul", "list-node", "List", !1) : e == "table" ? w("Table", "table", "table-node", "Table", !1) : e == "callout" ? w("Callout", "div", "callout-node", "Callout", !0) : e == "details" ? w("Details", "div", "details-node", "Details", !0) : e == "math_block" ? w("MathBlock", "div", "math-block", "MathBlock", !0) : e == "mermaid" ? w("Mermaid", "div", "mermaid-block-container", "Mermaid", !0) : e == "query" ? w("Query", "div", "query-block", "Query", !0) : e == "embed" ? w("Embed", "div", "embed-block", "Embed", !0) : w("Unknown", "div", "unknown-node", "", !1);
}
function Vt({ node: e, final: t, budget: n, renderInlineChildren: l }) {
  return e.type === "text" ? h("span", { class: "whitespace-pre-wrap break-words text-node" }, [h("span", e.content)]) : h("p", { class: "paragraph-node", dir: "auto" }, l(e.children, t, n));
}
function Qt({ node: e, final: t, budget: n, renderInlineChildren: l }) {
  const o = Math.min(6, Math.max(1, e.level));
  return h(`h${o}`, { class: `heading-node heading-${o}`, dir: "auto" }, [
    ...l(e.children, t, n)
  ]);
}
function Ut() {
  return h("hr", { class: "hr-node" });
}
function zt({ node: e, final: t, budget: n, renderEmbedded: l }) {
  return h("blockquote", { class: "blockquote", dir: "auto" }, [
    l(e.children, t, n)
  ]);
}
const jt = ["note", "info", "tip", "warning", "caution", "danger", "error"], V = Qt, Gt = {
  Text: Vt,
  H1: V,
  H2: V,
  H3: V,
  H4: V,
  H5: V,
  H6: V,
  Separator: Ut,
  Quote: zt
  // List and Callout are NOT here anymore (plan 035 T6): the container
  // families' widgets own those panel faces, registered on the custom slot
  // by block-widget-panels.ts (same channel Codeblock took in 033).
}, te = {};
function se(e, t) {
  te[e] = t;
}
function bo(e) {
  delete te[e];
}
function go() {
  for (const e of Object.keys(te)) delete te[e];
}
function Jt(e) {
  return (e == null ? void 0 : e.type) === "heading" ? Ke(e.level) : Kt((e == null ? void 0 : e.type) ?? "");
}
function Yt(e) {
  return te[e.kind] ?? Gt[e.kind];
}
let Xt = null;
function ve() {
  return Xt;
}
let ee = null;
function vo(e) {
  ee = e;
}
function yo() {
  return ee;
}
function Zt(e, t) {
  ee == null || ee(e, t);
}
function en() {
  return 2166136261;
}
function tn() {
  return 16777619;
}
function he() {
  return 65536 * 65536;
}
function Se(e, t) {
  const n = e % t;
  return (e - n) / t % 2;
}
function nn(e, t) {
  let n = 0, l = 1, o = 0;
  for (; o < 32; ) {
    const r = Se(e, l), a = Se(t, l);
    r != a && (n = n + l), l = l * 2, o = o + 1;
  }
  return n;
}
function ln(e, t) {
  const n = e % 65536, r = (e - n) / 65536 * t, a = n * t, m = r % he(), f = a % he();
  return (m * 65536 + f) % he();
}
function on(e, t) {
  const n = nn(e, t);
  return ln(n, tn());
}
function rn(e) {
  return "0123456789abcdef".slice(e, e + 1);
}
function an(e) {
  let t = "", n = e, l = 0;
  for (; l < 8; ) {
    const o = n % 16;
    t = rn(o) + t, n = (n - o) / 16, l = l + 1;
  }
  return t;
}
function sn(e, t, n) {
  let l = en();
  for (let o = 0; o < Number(n.length); o++)
    l = on(l, n[o]);
  return e + ":" + String(t) + ":" + an(l);
}
function un(e, t) {
  const n = e + "\0" + t, l = new Array(n.length);
  for (let o = 0; o < n.length; o++) l[o] = n.charCodeAt(o);
  return sn(e, t.length, l);
}
let Ve = null;
function Qe(e) {
  Ve = e;
}
function Ue() {
  return Ve;
}
const re = {};
function ye(e, t, n) {
  re[e] = { enabled: t, factory: n };
}
function _o(e) {
  ye("katex", !0, e);
}
function Co(e) {
  ye("mermaid", !0, e);
}
function wo(e) {
  ye("highlight", !0), Qe(e ?? null);
}
function cn(e) {
  var t;
  return ((t = re[e]) == null ? void 0 : t.enabled) === !0;
}
let ze = null;
function dn() {
  return ze;
}
function Bo() {
  for (const e of Object.keys(re))
    delete re[e];
  Qe(null), ze = null;
}
De.initialize({ startOnLoad: !1, theme: "default" });
function fn(e, t) {
  try {
    return {
      html: ct.renderToString(e, { throwOnError: !0, displayMode: t }),
      error: ""
    };
  } catch (n) {
    return { html: "", error: n.message || String(n) };
  }
}
async function xo(e) {
  try {
    const t = `mermaid-${Math.random().toString(36).slice(2)}`;
    return { svg: (await De.render(t, e)).svg, error: "" };
  } catch (t) {
    return { svg: "", error: t.message || String(t) };
  }
}
function To(e, t, n) {
  if (n.error !== "") return;
  const l = dn();
  l && l.put(un(e, t), n);
}
const Me = dt(ft), je = (e, t) => {
  if (!(!e || !t || t === "text" || t === "plaintext"))
    try {
      return Me.registered(t) ? ht(Me.highlight(t, e)) : void 0;
    } catch {
      return;
    }
};
class hn {
  constructor(t, n) {
    this.engine = t, this.blockId = n, this.knownCode = this.readModel();
  }
  get id() {
    return this.blockId;
  }
  get code() {
    return this.knownCode;
  }
  /** The live block (attrs included) — the SFC reads the language from it. */
  node() {
    return _(this.engine.doc, this.blockId);
  }
  /** The engine repaints after history changes / external edits — re-sync. */
  syncFromModel() {
    return this.knownCode = this.readModel(), this.knownCode;
  }
  /** Write the edited code text back; false = no change or block gone. */
  commit(t) {
    if (t === this.knownCode) return !1;
    const n = _(this.engine.doc, this.blockId);
    return n ? (this.engine.applyTree((l) => L(l, this.blockId, [mt(n, [Pe(t)])])), this.syncFromModel(), !0) : !1;
  }
  readModel() {
    const t = _(this.engine.doc, this.blockId);
    return t ? We(t) : "";
  }
}
function mn(e, t) {
  return cn("highlight") ? (Ue() ?? je)(e, t) ?? "" : "";
}
function ie(e) {
  return z(e);
}
function pn(e, t) {
  const n = mn(e, t);
  return n !== "" ? `<code translate="no" data-highlighted="${Ge(t)}">${n}</code>` : `<code translate="no">${z(e)}</code>`;
}
function kn(e, t) {
  return e === "edit" ? t : void 0;
}
function Ge(e) {
  return z(e).replace(/"/g, "&quot;");
}
function z(e) {
  return e.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function bn(e, t) {
  const l = (Ue() ?? je)(e, t);
  return l !== void 0 ? l : z(e);
}
function gn(e, t) {
  const n = bn(e, t);
  return n !== z(e) ? `<code translate="no" data-highlighted="${Ge(t)}">${n}</code>` : `<code translate="no">${z(e)}</code>`;
}
function vn(e, t) {
  if (!e || t) return;
  const n = e;
  n.focus();
  const l = n.value.length;
  n.setSelectionRange(l, l), Je(n);
}
function Je(e) {
  const t = e;
  t && (t.style.height = "auto", t.style.height = `${t.scrollHeight}px`);
}
function me(e, t) {
  const n = e;
  !n || !t || (t.style.height = n.style.height || `${n.offsetHeight}px`, t.scrollTop = n.scrollTop, t.scrollLeft = n.scrollLeft);
}
function yn(e) {
  return y((e == null ? void 0 : e.attrs) ?? [], "language", "");
}
function $e(e) {
  return We(e ?? { inlines: [] });
}
function _n(e) {
  return P((e == null ? void 0 : e.attrs) ?? [], "loading", !1);
}
function Ye(e) {
  return (e == null ? void 0 : e.readonly) === !0;
}
function _e(e) {
  const t = e == null ? void 0 : e.blockId;
  return typeof t == "string" ? t : "";
}
function Cn(e) {
  if (e == null) return null;
  const t = e;
  return new hn(t.engine, t.blockId);
}
function Ao(e, t) {
  return e === "edit" ? t : void 0;
}
function So(e) {
  return e === "edit" ? void 0 : "";
}
const wn = ["data-language"], Bn = ["data-block-id"], xn = {
  key: 0,
  class: "autodown-stream-banner"
}, Tn = { class: "code-block-header flex justify-between items-center" }, An = {
  class: "code-header-trigger",
  "data-codeblock-language-badge": "",
  title: "切换语言",
  type: "button"
}, Sn = { class: "code-header-title" }, Mn = { class: "code-editor-stack" }, $n = ["innerHTML"], Nn = ["disabled"], Hn = { class: "code-block-header flex justify-between items-center" }, In = {
  class: "code-header-trigger",
  "data-codeblock-language-badge": "",
  title: "切换语言",
  type: "button"
}, Ln = { class: "code-header-title" }, Rn = ["aria-busy", "data-language", "innerHTML"], On = /* @__PURE__ */ J({
  __name: "CodeBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean }
  },
  emits: ["Init", "AreaInput", "AreaScroll", "Blur"],
  setup(e, { emit: t }) {
    const n = e, l = X($e(n.node)), o = X(Cn(n.ctx)), r = X(null), a = X(null), m = u(() => n.mode === "edit"), f = u(() => yn(n.node)), b = u(() => $e(n.node)), g = u(() => Ye(n.ctx)), B = u(() => _e(n.ctx)), R = u(() => _n(n.node)), E = u(() => m.value ? "code-block-container rounded-lg border autodown-codeblock-node" : R.value ? "code-block-container rounded-lg border autodown-block-placeholder is-loading" : "code-block-container rounded-lg border"), N = u(() => kn(n.mode, f.value)), x = u(() => f.value ? f.value : "text"), F = u(() => "language-" + x.value + " code-pre-fallback is-wrap"), ne = u(() => R.value ? "true" : "false"), le = u(() => pn(b.value, f.value)), ue = u(() => gn(l.value, f.value)), Y = t;
    function ce(p) {
      Je(p.target), me(p.target, r.value), Y("AreaInput", p);
    }
    function de(p) {
      me(p.target, r.value), Y("AreaScroll", p);
    }
    function H(p) {
      g.value || o.value.commit(p.target.value), Y("Blur", p);
    }
    return Re(() => {
      m.value && (vn(a.value, g.value), me(a.value, r.value));
    }), (p, c) => (i(), d("div", {
      class: M(E.value),
      "data-language": N.value
    }, [
      m.value ? (i(), d("div", {
        key: 0,
        class: M(["autodown-code-editor", { "is-readonly": g.value }]),
        "data-block-id": B.value,
        "data-node-type": "Fence"
      }, [
        g.value ? (i(), d("div", xn, [...c[4] || (c[4] = [
          s("span", null, "流式生成中", -1)
        ])])) : v("", !0),
        s("div", Tn, [
          s("button", An, [
            s("span", Sn, [
              s("span", null, oe(f.value), 1)
            ]),
            c[5] || (c[5] = s("span", { class: "code-header-caret" }, [
              s("span", null, "▾")
            ], -1))
          ]),
          c[6] || (c[6] = xe('<div class="flex items-center gap-0.5" data-v-b676acc5><button class="code-action-btn" data-codeblock-copy-btn="" title="复制" type="button" data-v-b676acc5><span class="codeblock-copy-icon" data-v-b676acc5></span></button><button class="code-action-btn" data-codeblock-expand-btn="" title="折叠" type="button" data-v-b676acc5><span class="codeblock-expand-icon" data-v-b676acc5></span></button></div>', 1))
        ]),
        s("div", Mn, [
          s("pre", {
            class: "code-editor-highlight",
            "aria-hidden": "true",
            innerHTML: ue.value,
            ref_key: "hl",
            ref: r
          }, null, 8, $n),
          st(s("textarea", {
            class: "code-editor-textarea",
            disabled: g.value,
            ref_key: "area",
            ref: a,
            spellcheck: "false",
            "onUpdate:modelValue": c[0] || (c[0] = (C) => l.value = C),
            onBlur: c[1] || (c[1] = (C) => H(C)),
            onInput: c[2] || (c[2] = (C) => ce(C)),
            onScroll: c[3] || (c[3] = (C) => de(C))
          }, null, 40, Nn), [
            [it, l.value]
          ])
        ])
      ], 10, Bn)) : v("", !0),
      m.value ? v("", !0) : (i(), d(T, { key: 1 }, [
        s("div", Hn, [
          s("button", In, [
            s("span", Ln, [
              s("span", null, oe(f.value), 1)
            ]),
            c[7] || (c[7] = s("span", { class: "code-header-caret" }, [
              s("span", null, "▾")
            ], -1))
          ]),
          c[8] || (c[8] = xe('<div class="flex items-center gap-0.5" data-v-b676acc5><button class="code-action-btn" data-codeblock-copy-btn="" title="复制" type="button" data-v-b676acc5><span class="codeblock-copy-icon" data-v-b676acc5></span></button><button class="code-action-btn" data-codeblock-expand-btn="" title="折叠" type="button" data-v-b676acc5><span class="codeblock-expand-icon" data-v-b676acc5></span></button></div>', 1))
        ]),
        s("pre", {
          class: M(F.value),
          "aria-busy": ne.value,
          "data-language": f.value,
          innerHTML: le.value,
          tabindex: "0"
        }, null, 10, Rn)
      ], 64))
    ], 10, wn));
  }
}), Xe = (e, t) => {
  const n = e.__vccOpts || e;
  for (const [l, o] of t)
    n[l] = o;
  return n;
}, En = /* @__PURE__ */ Xe(On, [["__scopeId", "data-v-b676acc5"]]);
function Dn(e, t) {
  if (e.length !== t.length) return !1;
  for (const n of e) if (!U(t, n)) return !1;
  return !0;
}
function Pn(e, t) {
  return e._tag !== t._tag ? !1 : !("value" in e) || !("value" in t) ? !0 : JSON.stringify(e.value) === JSON.stringify(t.value);
}
function Wn(e, t) {
  if (e.length !== t.length) return !1;
  for (const n of e) {
    const l = t.find((o) => o.key === n.key);
    if (!l || !Pn(n.value, l.value)) return !1;
  }
  return !0;
}
function Mo(e) {
  const t = [];
  for (const n of e) {
    if (n.text === "") continue;
    const l = t[t.length - 1];
    l && Dn(l.marks, n.marks) && Wn(l.attrs, n.attrs) ? t[t.length - 1] = new pt(l.text + n.text, l.marks, l.attrs) : t.push(n);
  }
  return t;
}
function Ne(e, t, n) {
  if (e.length === 0) return [];
  const l = Z(e).length;
  t = Math.max(0, Math.min(t, l)), n = Math.max(t, Math.min(n, l));
  let o = [], r = 0, a = null;
  for (const f of e) {
    const b = r + f.text.length;
    t < b && n > r && o.push(f), r <= t && b > t && (a = f), b <= t && (a = f), r = b;
  }
  o.length === 0 && (o = a ? [a] : [e[e.length - 1]]);
  let m = [...o[0].marks];
  for (const f of o.slice(1)) m = m.filter((b) => U(f.marks, b));
  return m;
}
function $o(e, t, n) {
  _(e.doc, t) && e.applyTree((l) => L(l, t, n.length > 0 ? n : [ae(t, k.Paragraph, "")]));
}
function No(e, t) {
  const n = e.selection.anchor.blockId;
  !n || !_(e.doc, n) || e.applyTree((l) => L(l, n, t));
}
function Ho(e, t, n = 0) {
  _(e.doc, t) && e.select(new bt(new Ae(t, n), new Ae(t, n)));
}
function Io(e, t, n) {
  e.applyTree((l) => qn(l, t, n));
}
function qn(e, t, n) {
  var g;
  const l = _(e, t);
  if (!l) return e;
  const o = ((g = l.children[0]) == null ? void 0 : g.children.length) ?? 1, r = `row-${Math.random().toString(36).slice(2, 8)}`, a = [];
  for (let B = 0; B < o; B++) a.push(ae(`${r}-c${B}`, k.TableCell, ""));
  const m = I(vt(r, k.TableRow), a), f = [...l.children], b = n == null ? 0 : qe(l, n) + 1;
  return f.splice(b < 0 ? f.length : b, 0, m), L(e, t, [I(l, f)]);
}
function Lo(e, t) {
  e.applyTree((n) => L(n, t, []));
}
function Ro(e, t) {
  e.applyTree((n) => Fn(n, t));
}
function Fn(e, t) {
  const n = _(e, t);
  if (!n) return e;
  const l = n.children.map(
    (o) => I(o, [...o.children, ae(`${o.id}-nc`, k.TableCell, "")])
  );
  return L(e, t, [I(n, l)]);
}
function Oo(e, t, n) {
  const l = _(e, t);
  if (!l) return e;
  const o = l.children.map((r) => {
    const a = [...r.children], m = Math.max(0, Math.min(n, a.length));
    return a.splice(m, 0, ae(`${r.id}-nc${m}`, k.TableCell, "")), I(r, a);
  });
  return L(e, t, [I(l, o)]);
}
function Eo(e, t, n) {
  var r;
  const l = _(e, t);
  if (!l || (((r = l.children[0]) == null ? void 0 : r.children.length) ?? 0) <= 1) return e;
  const o = l.children.map((a) => I(a, a.children.filter((m, f) => f !== n)));
  return L(e, t, [I(l, o)]);
}
function Do(e, t) {
  e.applyTree((n) => {
    const l = _(n, t);
    if (!l) return n;
    const o = l.children.map((r) => I(r, r.children.slice(0, -1)));
    return L(n, t, [I(l, o)]);
  });
}
function Po(e, t, n) {
  e.applyTree((l) => {
    const o = gt(l, t);
    if (!o) return l;
    const r = qe(o, t), a = r + n;
    if (a < 0 || a >= o.children.length) return l;
    const m = [...o.children], [f] = m.splice(r, 1);
    return m.splice(a, 0, f), L(l, o.id, [I(o, m)]);
  });
}
function Ce(e, t, n) {
  e.applyTree((l) => {
    const o = _(l, t);
    if (!o) return l;
    let r = o;
    for (const a of n) r = { ...r, attrs: kt(r.attrs, a.key, a.value) };
    return L(l, t, [r]);
  });
}
function Wo(e, t) {
  const n = _(e.doc, t.anchor.blockId);
  if (!n) return [];
  if (t.anchor.blockId !== t.head.blockId) return Ne(n.inlines, t.anchor.offset, t.anchor.offset);
  const l = Math.min(t.anchor.offset, t.head.offset), o = Math.max(t.anchor.offset, t.head.offset);
  return Ne(n.inlines, l, o);
}
function Kn(e) {
  const t = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789", n = typeof crypto < "u" ? crypto : void 0, l = () => {
    if (n != null && n.getRandomValues) {
      const o = new Uint32Array(1);
      return n.getRandomValues(o), o[0] % t.length;
    }
    return Math.floor(Math.random() * t.length);
  };
  for (let o = 0; o < 64; o++) {
    let r = "";
    for (let a = 0; a < 7; a++) r += t[l()];
    if (!e.has(r)) return r;
  }
  return `a${Date.now().toString(36).slice(-6)}`;
}
function qo(e, t) {
  const n = _(e.doc, t);
  if (!n) return null;
  const l = yt(n);
  if (l) return l;
  const o = /* @__PURE__ */ new Set(), r = (m) => {
    o.add(m.id), m.children.forEach(r);
  };
  r(e.doc);
  const a = Kn(o);
  return e.applyTree((m) => _t(m, t, a)), a;
}
const j = J({
  name: "BlockChildren",
  props: { children_slot: { type: Function, required: !0 } },
  setup(e) {
    return () => e.children_slot();
  }
});
function Ze(e, t, n) {
  const o = _(e.doc, t);
  return o ? y(o.attrs, n, "") : "";
}
function Vn(e, t) {
  const n = e;
  n && (n.textContent = t);
}
function Qn(e) {
  return typeof document < "u" && document.activeElement === e;
}
function Un(e, t, n, l) {
  const o = e;
  o && !Qn(o) && (o.textContent = Ze(t, n, l));
}
function zn(e, t, n, l, o) {
  if (o) return;
  const r = e, a = ((r == null ? void 0 : r.textContent) ?? "").replace(/\u00a0/g, " ").trim();
  a !== Ze(t, n, l) && Ce(t, n, [{ key: l, value: S.Str(a) }]);
}
function He(e) {
  const t = e;
  t && t.blur();
}
const jn = ["contenteditable", "data-placeholder", "onKeydown"], Gn = /* @__PURE__ */ J({
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
  setup(e, { emit: t }) {
    const n = e, l = X(null), o = u(() => !n.readonly), r = u(() => "autodown-attr-host " + n.host_class), a = t;
    ut(() => n.version, () => {
      Un(l.value, n.controller, n.blockId, n.attr_key);
    });
    function m(g) {
      zn(g.target, n.controller, n.blockId, n.attr_key, n.readonly), a("Blur", g);
    }
    function f() {
      He(l.value), a("KeyEnter");
    }
    function b() {
      He(l.value), a("KeyEscape");
    }
    return Re(() => {
      Vn(l.value, n.value);
    }), (g, B) => (i(), d("span", {
      class: M(r.value),
      contenteditable: o.value,
      "data-placeholder": e.placeholder,
      ref_key: "host",
      ref: l,
      spellcheck: "false",
      onBlur: B[0] || (B[0] = (R) => m(R)),
      onKeydown: [
        Te(pe(f, ["prevent"]), ["enter"]),
        Te(pe(b, ["prevent"]), ["esc"])
      ]
    }, null, 42, jn));
  }
});
function et(e) {
  return (e == null ? void 0 : e.engine) ?? null;
}
function Ie(e, t) {
  return e ? y(e.attrs, t, "") : "";
}
function Jn(e) {
  return jt.includes(e);
}
function Yn(e, t) {
  return e ? P(e.attrs, t, !1) : !1;
}
function Xn(e, t) {
  return e ? ke(e.attrs, t, 1) : 1;
}
function Zn(e, t, n) {
  return e === "edit" && t ? n : void 0;
}
function el(e, t) {
  const n = e;
  if (!n || !t) return;
  const l = _(n.doc, t);
  if (!l) return;
  const o = P(l.attrs, "checked", !1);
  Ce(n, t, [{ key: "checked", value: S.Bool(!o) }]);
}
function Fo(e, t) {
  const n = _e(t);
  return n || ((e == null ? void 0 : e.id) ?? "");
}
function Ko(e, t, n) {
  const l = e;
  !l || !t || Ce(l, t, [{ key: "open", value: S.Bool(!n) }]);
}
const tl = ["data-callout-type"], nl = {
  key: 0,
  class: "autodown-stream-banner"
}, ll = { class: "autodown-callout-header" }, ol = ["innerHTML"], rl = { class: "autodown-callout-content" }, al = {
  key: 0,
  class: "markdown-renderer"
}, sl = /* @__PURE__ */ J({
  __name: "CalloutBlockWidget",
  props: {
    mode: {},
    node: {},
    ctx: {},
    final: { type: Boolean },
    children: {},
    version: {}
  },
  setup(e) {
    const t = e, n = u(() => t.mode === "edit"), l = u(() => Ye(t.ctx)), o = u(() => _e(t.ctx)), r = u(() => et(t.ctx)), a = u(() => Ie(t.node, "type")), m = u(() => Jn(a.value)), f = u(() => Ie(t.node, "title")), b = u(() => f.value ? f.value : a.value), g = u(() => ie(b.value)), B = u(() => a.value ? a.value : "标题"), R = u(() => "callout-node autodown-callout autodown-callout-" + a.value), E = u(() => "autodown-callout-icon autodown-callout-icon-" + a.value);
    return (N, x) => (i(), d("div", {
      class: M(R.value),
      "data-callout-type": a.value
    }, [
      n.value ? (i(), d(T, { key: 0 }, [
        l.value ? (i(), d("div", nl, [...x[0] || (x[0] = [
          s("span", null, "流式生成中", -1)
        ])])) : v("", !0)
      ], 64)) : v("", !0),
      s("div", ll, [
        m.value ? (i(), d("span", {
          key: 0,
          class: M(E.value),
          "aria-hidden": "true"
        }, null, 2)) : v("", !0),
        n.value ? (i(), D(q(Gn), {
          attr_key: "title",
          blockId: o.value,
          controller: r.value,
          host_class: "autodown-callout-title",
          placeholder: B.value,
          readonly: l.value,
          value: f.value,
          version: e.version,
          key: "AttrHost-1"
        }, null, 8, ["blockId", "controller", "placeholder", "readonly", "value", "version"])) : v("", !0),
        n.value ? v("", !0) : (i(), d("div", {
          key: 2,
          class: "autodown-callout-title",
          dir: "auto",
          innerHTML: g.value
        }, null, 8, ol))
      ]),
      s("div", rl, [
        n.value ? (i(), d("div", al, [
          (i(), D(q(j), {
            children_slot: e.children,
            key: "BlockChildren-2"
          }, null, 8, ["children_slot"]))
        ])) : v("", !0),
        n.value ? v("", !0) : (i(), D(q(j), {
          children_slot: e.children,
          key: "BlockChildren-3"
        }, null, 8, ["children_slot"]))
      ])
    ], 10, tl));
  }
}), il = ["aria-label", "checked", "disabled", "onClick"], ul = {
  key: 1,
  class: "markdown-renderer"
}, cl = /* @__PURE__ */ J({
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
  setup(e, { emit: t }) {
    const n = e, l = u(() => n.mode === "edit"), o = u(() => et(n.ctx)), r = u(() => Yn(n.node, "ordered")), a = u(() => r.value ? "ol" : "ul"), m = u(() => r.value ? "list-node list-decimal" : "list-node list-disc"), f = u(() => Zn(n.mode, r.value, Xn(n.node, "start"))), b = u(() => !l.value), g = u(() => l.value ? "toggle task" : "task checkbox"), B = t;
    function R(E) {
      el(o.value, E.id), B("TaskClick", E);
    }
    return (E, N) => (i(), D(Oe(a.value), {
      class: M(m.value),
      start: f.value
    }, {
      default: Ee(() => [
        (i(!0), d(T, null, O(e.items, (x, F) => (i(), d("li", {
          class: M(x.cls),
          dir: "auto",
          key: x.id
        }, [
          x.task ? (i(), d("input", {
            key: 0,
            class: "task-checkbox",
            "aria-label": g.value,
            checked: x.checked,
            disabled: b.value,
            type: "checkbox",
            onClick: pe((ne) => R(x), ["stop"])
          }, null, 8, il)) : v("", !0),
          l.value ? (i(), d("div", ul, [
            (i(), D(q(j), {
              children_slot: x.children_slot,
              key: "BlockChildren-1-" + F
            }, null, 8, ["children_slot"]))
          ])) : v("", !0),
          l.value ? v("", !0) : (i(), D(q(j), {
            children_slot: x.children_slot,
            key: "BlockChildren-2-" + F
          }, null, 8, ["children_slot"]))
        ], 2))), 128))
      ]),
      _: 1
    }, 8, ["class", "start"]));
  }
});
function dl(e, t) {
  var o;
  const n = t == null ? void 0 : t.target, l = (o = n == null ? void 0 : n.dataset) == null ? void 0 : o.cellId;
  l && e.commitCell(l, n.innerText.replace(/\n+$/, ""));
}
function fl(e) {
  return e === "view" ? "table" : "div";
}
function hl(e, t, n) {
  return e === "view" ? "table-node" : e === "stream" ? t ? "streaming-table final" : "streaming-table" : n ? "autodown-table-editor is-readonly" : "autodown-table-editor";
}
function ml(e) {
  return e === "view" ? "false" : void 0;
}
function pl(e, t) {
  return e === "edit" ? t : void 0;
}
function kl(e) {
  return e === "edit" ? "Table" : void 0;
}
function bl(e) {
  return (e ?? []).map((n) => ({ col: String(n), html: ie(String(n)) }));
}
function gl(e, t) {
  const n = e ?? [];
  return (t ?? []).map(
    (o) => n.map((r) => ({ col: String(r), html: ie(String((o == null ? void 0 : o[r]) ?? "")) }))
  );
}
function vl(e) {
  const t = (e ?? []).length;
  return Math.max(1, t);
}
const yl = {
  key: 0,
  class: "autodown-stream-banner"
}, _l = {
  class: "te-toolbar",
  "aria-label": "表格工具栏",
  role: "toolbar"
}, Cl = ["disabled"], wl = ["disabled"], Bl = ["disabled"], xl = ["disabled"], Tl = ["disabled"], Al = ["disabled"], Sl = ["disabled"], Ml = {
  class: "table-node",
  "aria-busy": "false"
}, $l = ["contenteditable", "data-cell-id"], Nl = ["contenteditable", "data-cell-id"], Hl = { key: 2 }, Il = ["innerHTML"], Ll = ["innerHTML"], Rl = {
  key: 0,
  class: "loading-row"
}, Ol = ["colspan"], El = ["innerHTML"], Dl = /* @__PURE__ */ J({
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
  emits: ["AddRowAbove", "AddRow", "DeleteRow", "AddColumnBefore", "AddColumn", "DeleteColumn", "DeleteTable", "CellBlur"],
  setup(e, { emit: t }) {
    const n = e, l = u(() => n.mode === "edit"), o = u(() => n.mode === "view"), r = u(() => fl(n.mode)), a = u(() => hl(n.mode, n.final, n.readonly)), m = u(() => ml(n.mode)), f = u(() => pl(n.mode, n.blockId)), b = u(() => kl(n.mode)), g = u(() => bl(n.columns)), B = u(() => gl(n.columns, n.rows)), R = u(() => vl(n.columns)), E = u(() => ie("Loading")), N = t;
    function x() {
      n.controller.addColumn(), N("AddColumn");
    }
    function F() {
      n.controller.addRow(), N("AddRow");
    }
    function ne() {
      n.controller.addRowAbove(), N("AddRowAbove");
    }
    function le(H) {
      if (!n.readonly) {
        let p = n.controller;
        dl(p, H);
      }
      N("CellBlur", H);
    }
    function ue() {
      n.controller.deleteColumn(), N("DeleteColumn");
    }
    function Y() {
      n.controller.deleteRow(), N("DeleteRow");
    }
    function ce() {
      n.controller.deleteTable(), N("DeleteTable");
    }
    function de() {
      N("AddColumnBefore");
    }
    return (H, p) => (i(), D(Oe(r.value), {
      class: M(a.value),
      "aria-busy": m.value,
      "data-block-id": f.value,
      "data-node-type": b.value
    }, {
      default: Ee(() => [
        l.value ? (i(), d(T, { key: 0 }, [
          e.readonly ? (i(), d("div", yl, [...p[2] || (p[2] = [
            s("span", null, "流式生成中", -1)
          ])])) : v("", !0),
          s("div", _l, [
            s("button", {
              class: "te-btn",
              "data-te-action": "add-row-above",
              disabled: e.readonly,
              title: "在上方插入一行",
              type: "button",
              onClick: ne
            }, [...p[3] || (p[3] = [
              s("span", null, "行↑", -1)
            ])], 8, Cl),
            s("button", {
              class: "te-btn",
              "data-te-action": "add-row",
              disabled: e.readonly,
              title: "在末尾后插入一行",
              type: "button",
              onClick: F
            }, [...p[4] || (p[4] = [
              s("span", null, "行↓", -1)
            ])], 8, wl),
            s("button", {
              class: "te-btn",
              "data-te-action": "delete-row",
              disabled: e.readonly,
              title: "删除最后一行",
              type: "button",
              onClick: Y
            }, [...p[5] || (p[5] = [
              s("span", null, "删行", -1)
            ])], 8, Bl),
            s("button", {
              class: "te-btn",
              "data-te-action": "add-col-before",
              disabled: e.readonly,
              title: "在左侧插入一列",
              type: "button",
              onClick: de
            }, [...p[6] || (p[6] = [
              s("span", null, "列←", -1)
            ])], 8, xl),
            s("button", {
              class: "te-btn",
              "data-te-action": "add-col",
              disabled: e.readonly,
              title: "追加一列",
              type: "button",
              onClick: x
            }, [...p[7] || (p[7] = [
              s("span", null, "列→", -1)
            ])], 8, Tl),
            s("button", {
              class: "te-btn",
              "data-te-action": "delete-col",
              disabled: e.readonly,
              title: "删除最后一列",
              type: "button",
              onClick: ue
            }, [...p[8] || (p[8] = [
              s("span", null, "删列", -1)
            ])], 8, Al),
            s("button", {
              class: "te-btn te-btn-danger",
              "data-te-action": "delete-table",
              disabled: e.readonly,
              title: "删除整个表格",
              type: "button",
              onClick: ce
            }, [...p[9] || (p[9] = [
              s("span", null, "删表", -1)
            ])], 8, Sl)
          ]),
          s("table", Ml, [
            s("thead", null, [
              s("tr", null, [
                (i(!0), d(T, null, O(e.header_cells, (c, C) => (i(), d("th", {
                  class: M(c.cls),
                  contenteditable: e.readonly == !1,
                  "data-cell-id": c.id,
                  dir: "auto",
                  key: c.id,
                  spellcheck: "false",
                  onBlur: p[0] || (p[0] = (A) => le(A))
                }, [
                  s("span", null, oe(c.text), 1)
                ], 42, $l))), 128))
              ])
            ]),
            s("tbody", null, [
              (i(!0), d(T, null, O(e.body_rows, (c, C) => (i(), d("tr", {
                key: c.id
              }, [
                (i(!0), d(T, null, O(c.cells, (A, fe) => (i(), d("td", {
                  class: M(A.cls),
                  contenteditable: e.readonly == !1,
                  "data-cell-id": A.id,
                  dir: "auto",
                  key: A.id,
                  spellcheck: "false",
                  onBlur: p[1] || (p[1] = (at) => le(at))
                }, [
                  s("span", null, oe(A.text), 1)
                ], 42, Nl))), 128))
              ]))), 128))
            ])
          ])
        ], 64)) : v("", !0),
        o.value ? (i(), d(T, { key: 1 }, [
          s("thead", null, [
            s("tr", null, [
              (i(!0), d(T, null, O(e.header_cells, (c, C) => (i(), d("th", {
                class: M(c.cls),
                dir: "auto",
                key: c.id
              }, [
                (i(), D(q(j), {
                  children_slot: c.children_slot,
                  key: "BlockChildren-1-" + C
                }, null, 8, ["children_slot"])),
                p[10] || (p[10] = s("button", {
                  class: "table-node__resize-handle",
                  type: "button"
                }, null, -1))
              ], 2))), 128))
            ])
          ]),
          s("tbody", null, [
            (i(!0), d(T, null, O(e.body_rows, (c, C) => (i(), d("tr", {
              key: c.id
            }, [
              (i(!0), d(T, null, O(c.cells, (A, fe) => (i(), d("td", {
                class: M(A.cls),
                dir: "auto",
                key: A.id
              }, [
                (i(), D(q(j), {
                  children_slot: A.children_slot,
                  key: "BlockChildren-2-" + fe
                }, null, 8, ["children_slot"]))
              ], 2))), 128))
            ]))), 128))
          ])
        ], 64)) : v("", !0),
        e.mode == "stream" ? (i(), d("table", Hl, [
          s("thead", null, [
            s("tr", null, [
              (i(!0), d(T, null, O(g.value, (c, C) => (i(), d("th", {
                innerHTML: c.html,
                key: c.col
              }, null, 8, Il))), 128))
            ])
          ]),
          s("tbody", null, [
            (i(!0), d(T, null, O(B.value, (c, C) => (i(), d("tr", { key: C }, [
              (i(!0), d(T, null, O(c, (A, fe) => (i(), d("td", {
                innerHTML: A.html,
                key: A.col
              }, null, 8, Ll))), 128))
            ]))), 128)),
            e.final ? v("", !0) : (i(), d("tr", Rl, [
              s("td", { colspan: R.value }, [
                s("span", {
                  class: "loading-dots",
                  innerHTML: E.value
                }, null, 8, El)
              ], 8, Ol)
            ]))
          ])
        ])) : v("", !0)
      ]),
      _: 1
    }, 8, ["class", "aria-busy", "data-block-id", "data-node-type"]));
  }
}), Pl = /* @__PURE__ */ Xe(Dl, [["__scopeId", "data-v-08b13a68"]]), tt = /* @__PURE__ */ new WeakMap();
function nt(e) {
  return tt.get(e);
}
function Vo(e) {
  return (e ?? []).map(W);
}
function W(e) {
  const t = Wl(e);
  return tt.set(t, e), t;
}
function Wl(e) {
  switch (e.kind) {
    case k.Heading:
      return $t(ke(e.attrs, "level", 1), be(e.inlines));
    case k.Fence:
      return Mt(
        y(e.attrs, "language", ""),
        Z(e.inlines),
        P(e.attrs, "loading", !1)
      );
    case k.Blockquote:
      return St(e.children.map(W));
    case k.ListBlock:
      return At(
        P(e.attrs, "ordered", !1),
        ke(e.attrs, "start", 1),
        e.children.map(W)
      );
    case k.ListItem: {
      const n = xt(e.attrs, "checked") == null ? null : P(e.attrs, "checked", !1);
      return Tt(e.children.map(W), n);
    }
    case k.Table: {
      const t = e.children.map(ql);
      return Bt(t.length > 0 ? [t[0]] : [], t.slice(1), P(e.attrs, "loading", !1));
    }
    case k.ThematicBreak:
      return wt();
    case k.Callout: {
      const t = new K(
        "callout",
        null,
        null,
        null,
        null,
        null,
        e.children.map(W),
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
      return t.language = y(e.attrs, "type", ""), t.title = y(e.attrs, "title", ""), t;
    }
    case k.Details:
      return new K(
        "details",
        null,
        null,
        null,
        null,
        null,
        e.children.map(W),
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
    case k.MathBlock:
      return new K(
        "math_block",
        null,
        null,
        null,
        Z(e.inlines),
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
    case k.Mermaid:
      return new K(
        "mermaid",
        null,
        null,
        null,
        Z(e.inlines),
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
    case k.QueryBlock:
      return new K(
        "query",
        y(e.attrs, "query", ""),
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
    case k.BlockEmbed:
      return new K(
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
        y(e.attrs, "src", ""),
        null,
        null
      );
    default:
      return Ct(be(e.inlines));
  }
}
function ql(e) {
  return Nt(e.children.map(Fl));
}
function Fl(e) {
  return Ht(
    P(e.attrs, "header", !1),
    be(e.inlines),
    y(e.attrs, "align", "left")
  );
}
const Kl = [$.Strong, $.Em, $.Underline, $.Del, $.Link];
function be(e) {
  return ge(e, 0);
}
function ge(e, t) {
  const n = Kl[t];
  if (n === void 0) return Ql(e);
  const l = [];
  let o = [];
  const r = () => {
    o.length !== 0 && (l.push(Vl(n, o, ge(o, t + 1))), o = []);
  };
  for (const a of e)
    U(a.marks, n) ? o.push(a) : (r(), l.push(...ge([a], t + 1)));
  return r(), l;
}
function Vl(e, t, n) {
  switch (e) {
    case $.Strong:
      return Ft(n);
    case $.Em:
      return qt(n);
    case $.Underline:
      return Wt(n);
    case $.Del:
      return Pt(n);
    case $.Link: {
      const l = y(t[0].attrs, "href", ""), o = y(t[0].attrs, "title", "");
      return Fe(l, o.length > 0 ? o : null, Z(t), n, !1);
    }
    default:
      return n[0];
  }
}
function Ql(e) {
  const t = [];
  for (const n of e) {
    if (n.text === `
`) {
      t.push(It());
      continue;
    }
    if (y(n.attrs, "wikilink", "") !== "") {
      t.push(Lt(n.text));
      continue;
    }
    if (y(n.attrs, "math_inline", "") !== "") {
      t.push(Rt(n.text));
      continue;
    }
    if (U(n.marks, $.Image)) {
      const l = Ot(y(n.attrs, "src", ""), n.text), o = y(n.attrs, "title", "");
      if (o.length > 0 && (l.title = o), U(n.marks, $.Link)) {
        const r = y(n.attrs, "href", "");
        t.push(Fe(r, null, n.text, [l], !1));
      } else
        t.push(l);
      continue;
    }
    if (U(n.marks, $.Code)) {
      t.push(Et(n.text));
      continue;
    }
    t.push(Dt(n.text));
  }
  return t;
}
function we(e) {
  const t = (e ?? "").split(/[_-]+/).filter((n) => n.length > 0);
  return t.length === 0 ? "" : t.map((n) => n.charAt(0).toUpperCase() + n.slice(1)).join("");
}
const G = {};
function Ul(e, t) {
  const n = we(e);
  G[n] = { ...G[n], ...t };
}
function zl(e) {
  delete G[we(e)];
}
function Qo() {
  for (const e of Object.keys(G)) delete G[e];
}
function Le(e, t) {
  return oo([W(e)], t)[0];
}
function jl(e) {
  const t = G[we(e)];
  return t ? {
    view: t.view ?? Le,
    stream: t.stream,
    edit: t.edit
  } : { view: Le };
}
function Uo(e) {
  return jl(e).edit;
}
function zo(e) {
  return (t, n) => h(e, { node: t, ctx: n });
}
function jo(e, t) {
  Ul(e, {
    view: (n, l) => h(t, { mode: "view", node: n, final: l, ctx: null }),
    stream: (n, l) => h(t, { mode: "stream", node: n, final: l, ctx: null }),
    edit: (n, l) => h(t, { mode: "edit", node: n, ctx: l })
  });
}
function Go(e) {
  zl(e);
}
function Gl(e) {
  return (t) => {
    const n = nt(t.node) ?? Jl(t.node);
    return h(e, { mode: "view", node: n, final: t.final ?? !0, ctx: null });
  };
}
function Jl(e) {
  const t = [], n = typeof (e == null ? void 0 : e.code) == "string" ? e.code : "";
  return (e == null ? void 0 : e.type) === "code_block" && (t.push({ key: "language", value: S.Str(String(e.language ?? "")) }), (e == null ? void 0 : e.loading) === !0 && t.push({ key: "loading", value: S.Bool(!0) })), (e == null ? void 0 : e.type) === "query" && t.push({ key: "query", value: S.Str(String(e.content ?? "")) }), (e == null ? void 0 : e.type) === "embed" && t.push({ key: "src", value: S.Str(String(e.src ?? "")) }), {
    id: "nv",
    kind: Yl((e == null ? void 0 : e.type) ?? ""),
    attrs: t,
    children: [],
    inlines: n.length > 0 ? [Pe(n)] : [],
    source: { start: 0, end: 0 }
  };
}
function Yl(e) {
  return e === "code_block" ? k.Fence : e === "mermaid" ? k.Mermaid : e === "query" ? k.QueryBlock : e === "embed" ? k.BlockEmbed : k.MathBlock;
}
function Xl(e) {
  const t = [];
  return (e == null ? void 0 : e.type) === "callout" && (t.push({ key: "type", value: S.Str(String(e.language ?? "")) }), t.push({ key: "title", value: S.Str(String(e.title ?? "")) })), (e == null ? void 0 : e.type) === "details" && (t.push({ key: "summary", value: S.Str(String(e.text ?? "")) }), (e == null ? void 0 : e.loading) === !0 && t.push({ key: "open", value: S.Bool(!0) })), (e == null ? void 0 : e.type) === "list" && (t.push({ key: "ordered", value: S.Bool(e.ordered === !0) }), t.push({ key: "start", value: S.Int(typeof e.start == "number" ? e.start : 1) })), {
    id: "nv",
    kind: k.Paragraph,
    attrs: t,
    children: [],
    inlines: [],
    source: { start: 0, end: 0 }
  };
}
function lt(e) {
  return nt(e) ?? Xl(e);
}
function Zl(e) {
  return (t) => {
    const n = t.final ?? !0, l = ve();
    return h(e, {
      mode: "view",
      node: lt(t.node),
      final: n,
      ctx: null,
      children: Be(l, () => t.renderEmbedded(t.node.children ?? [], n, t.budget)),
      version: 0
    });
  };
}
function eo(e) {
  const t = e.final ?? !0, n = ve();
  return (e.node.items ?? []).map((l, o) => ({
    id: `li-${o}`,
    task: l.checked != null,
    checked: l.checked === !0,
    cls: "list-item" + (l.checked != null ? " task-item" : ""),
    children_slot: Be(n, () => e.renderEmbedded(l.children ?? [], t, e.budget))
  }));
}
function to(e) {
  return (e == null ? void 0 : e.align) === "center" ? "text-center" : (e == null ? void 0 : e.align) === "right" ? "text-right" : "text-left";
}
function ot(e, t) {
  const n = e.final ?? !0, l = ve();
  return (t ?? []).map((o, r) => ({
    id: `cell-${r}`,
    cls: to(o),
    children_slot: Be(l, () => e.renderEmbedded(o.children ?? [], n, e.budget))
  }));
}
function no(e) {
  var t, n;
  return ot(e, ((n = (t = e.node.header) == null ? void 0 : t[0]) == null ? void 0 : n.cells) ?? []);
}
function lo(e) {
  return (e.node.rows ?? []).map((t, n) => ({
    id: `tr-${n}`,
    cells: ot(e, t.cells)
  }));
}
function Be(e, t) {
  return () => {
    const n = t();
    return Array.isArray(n) ? n : [n];
  };
}
se("Codeblock", Gl(En));
se("Callout", Zl(sl));
se(
  "List",
  (e) => h(cl, {
    mode: "view",
    node: lt(e.node),
    final: e.final ?? !0,
    ctx: null,
    items: eo(e),
    version: 0
  })
);
se(
  "Table",
  (e) => h(Pl, {
    mode: "view",
    final: e.final ?? !0,
    ctx: null,
    // filler values for the generated required-prop checks (the 033
    // ctx:null idiom): the view face reads none of these
    controller: null,
    blockId: "",
    readonly: !1,
    columns: [],
    rows: [],
    header_cells: no(e),
    body_rows: lo(e)
  })
);
function oo(e, t, n) {
  const l = n !== void 0 && Number.isFinite(n) ? { remaining: n } : void 0;
  return (e ?? []).map((o, r) => {
    const a = r === e.length - 1;
    return rt(o, r, t, a ? l : void 0);
  });
}
function rt(e, t, n, l) {
  return h("div", { class: "node-slot", "data-node-index": String(t), "data-node-type": e.type }, [
    h("div", { class: "node-content" }, [uo(e, n, l)])
  ]);
}
function ro(e, t, n) {
  const l = (e ?? []).map((o, r) => {
    const a = r === ((e == null ? void 0 : e.length) ?? 0) - 1;
    return rt(o, r, t, a ? n : void 0);
  });
  return h("div", { class: "markdown-renderer" }, l);
}
function Q(e, t, n) {
  return (e ?? []).map((l) => io(l, t, n));
}
function ao(e) {
  return e.content ?? e.code ?? "";
}
function so(e, t) {
  if (!t) return e;
  if (t.remaining <= 0) return "";
  const n = t.remaining >= e.length ? e : e.slice(0, t.remaining);
  return t.remaining -= n.length, n;
}
function io(e, t, n) {
  switch (e.type) {
    case "text":
      return h("span", { class: "whitespace-pre-wrap break-words text-node" }, [h("span", so(e.content, n))]);
    case "strong":
      return h("strong", { class: "strong-node" }, Q(e.children, t, n));
    case "emphasis":
      return h("em", { class: "emphasis-node" }, Q(e.children, t, n));
    case "underline":
      return h("u", { class: "underline-node" }, Q(e.children, t, n));
    case "strikethrough":
      return h("del", { class: "strikethrough-node" }, Q(e.children, t, n));
    case "inline_code":
      return h("code", { class: "inline-code" }, [h("span", e.code)]);
    case "link":
      return h(
        "a",
        {
          class: "link-node",
          href: e.href,
          title: e.title ?? void 0,
          target: "_blank",
          rel: "noopener noreferrer"
        },
        Q(e.children, t, n)
      );
    case "image":
      return h("span", { class: "image-node-container" }, [
        h("img", {
          src: e.src,
          alt: e.alt,
          title: e.alt,
          class: "image-node__img",
          loading: "lazy"
        })
      ]);
    case "hardbreak":
      return h("br");
    case "math_inline": {
      const l = e.code ?? "", o = fn(l, !1);
      return o.error === "" ? h("span", { class: "autodown-math-inline", "data-math-src": l }, [
        h("span", { class: "math-inline-render", innerHTML: o.html })
      ]) : h(
        "span",
        {
          class: "autodown-math-inline autodown-math-error",
          "data-math-src": l,
          title: o.error
        },
        [l]
      );
    }
    case "wikilink": {
      const l = e.title ?? "", o = l.indexOf("#"), r = (o >= 0 ? l.slice(0, o) : l).trim(), a = o >= 0 ? l.slice(o + 1).trim() : void 0, m = a ? `${r}#${a}` : r;
      return h(
        "span",
        {
          class: "autodown-wikilink-label",
          "data-wikilink-title": r,
          onClick: (f) => {
            f.stopPropagation(), Zt(r, a);
          }
        },
        m
      );
    }
    default:
      return h("span", { class: "whitespace-pre-wrap break-words text-node" }, [
        h("span", ao(e))
      ]);
  }
}
function uo(e, t, n) {
  const l = Jt(e), o = Yt(l);
  return o ? o({ node: e, final: t, budget: n, spec: l, renderEmbedded: ro, renderInlineChildren: Q }) : h("div", { class: "unknown-node" }, String(e.type));
}
export {
  Ie as $,
  Qe as A,
  zo as B,
  zl as C,
  Go as D,
  oo as E,
  Mo as F,
  Wo as G,
  Eo as H,
  Oo as I,
  qn as J,
  j as K,
  qo as L,
  fn as M,
  To as N,
  $e as O,
  Cn as P,
  Ye as Q,
  So as R,
  Ao as S,
  Pl as T,
  _e as U,
  vn as V,
  xo as W,
  et as X,
  Fo as Y,
  Yn as Z,
  Xe as _,
  Io as a,
  Gn as a0,
  Ko as a1,
  nt as a2,
  Be as a3,
  Vo as a4,
  sl as a5,
  cl as a6,
  En as a7,
  ve as a8,
  vo as a9,
  yo as aa,
  Do as b,
  Lo as c,
  Bo as d,
  go as e,
  Ho as f,
  wo as g,
  _o as h,
  $o as i,
  Co as j,
  cn as k,
  se as l,
  Po as m,
  we as n,
  Qo as o,
  Uo as p,
  Ue as q,
  No as r,
  Ce as s,
  Ro as t,
  bo as u,
  je as v,
  Gl as w,
  Ul as x,
  jo as y,
  jl as z
};
