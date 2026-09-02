import { ref as O, watch as U, computed as B, onScopeDispose as ae, defineComponent as G, onMounted as Q, openBlock as S, createElementBlock as C, Fragment as q, renderList as X, createBlock as _, resolveDynamicComponent as H, onBeforeUnmount as ie, normalizeClass as se, createElementVNode as V, toDisplayString as ce, createVNode as ue, mergeProps as de, createCommentVNode as fe, h as pe, nextTick as me } from "vue";
import { p as he, s as ge } from "./markdown-parser-0FkmfLuR.js";
import { E as J, j as ke, k as ye, g as ve, h as be, T as we, z as Se, _ as Be } from "./render-node-DdquDFdQ.js";
import { createLowlight as Te, common as xe } from "lowlight";
import { toHtml as Ne } from "hast-util-to-html";
function Ce(t, o, l) {
  let r = o - t;
  return r <= 0 ? t : l <= 0 ? o : r > l ? t + l : o;
}
function Ee(t, o) {
  if (o <= 0)
    return 0;
  let l = t - o;
  return l < 0 ? 0 : l;
}
function Ie(t, o, l) {
  if (o - t <= 0)
    return o;
  let a = t + l;
  return a > o ? o : a;
}
const Le = {
  setTimeout: (t, o) => setTimeout(t, o),
  clearTimeout: (t) => clearTimeout(t)
};
function Ae(t, o) {
  const l = o.timer ?? Le, r = O(t.value.length), a = O(Number.POSITIVE_INFINITY);
  let i;
  function c(p) {
    i !== void 0 && l.clearTimeout(i), i = l.setTimeout(() => {
      i = void 0, p();
    }, o.batchDelay);
  }
  function f(p) {
    return p ? p.type === "text" ? String(p.content ?? "") : (p.children ?? []).map((T) => f(T)).join("") : "";
  }
  function b() {
    const p = t.value[t.value.length - 1], h = f(p).length;
    if (h <= 0) {
      a.value = Number.POSITIVE_INFINITY;
      return;
    }
    a.value = 0;
    const T = () => {
      const x = Ie(a.value, h, o.typewriterChunk);
      a.value = x, x < h && c(T);
    };
    c(T);
  }
  U(
    t,
    (p) => {
      if (i !== void 0 && (l.clearTimeout(i), i = void 0), !o.enabled) {
        r.value = p.length, a.value = Number.POSITIVE_INFINITY;
        return;
      }
      const h = p.length, T = Math.min(h, Math.max(1, Math.floor(o.batchSize / 4) || 1));
      r.value = Math.max(r.value, T);
      const x = () => {
        const g = Ce(r.value, h, o.batchSize);
        r.value = g, g < h ? c(x) : o.typewriter && b();
      };
      r.value < h ? c(x) : o.typewriter && b();
    },
    { immediate: !0 }
  );
  const k = B(() => Ee(r.value, o.maxLiveNodes)), y = B(() => t.value.slice(k.value, r.value));
  return ae(() => {
    i !== void 0 && l.clearTimeout(i);
  }), { visibleNodes: y, visibleCount: r, typewriterChars: a, windowStart: k };
}
const _e = { class: "markdown-renderer" }, Y = /* @__PURE__ */ G({
  __name: "MarkdownRender",
  props: {
    content: { default: "" },
    final: { type: Boolean, default: !0 },
    batchRendering: { type: Boolean, default: !0 },
    initialRenderBatchSize: { default: 40 },
    renderBatchSize: { default: 80 },
    renderBatchDelay: { default: 16 },
    typewriter: { type: Boolean, default: !1 },
    fade: { type: Boolean, default: !0 },
    maxLiveNodes: { default: 320 }
  },
  setup(t) {
    const o = t, l = B(() => he(ge(o.content ?? ""), o.final)), r = B(() => J(l.value, o.final)), a = Ae(l, {
      enabled: o.batchRendering,
      batchSize: o.renderBatchSize,
      batchDelay: o.renderBatchDelay,
      maxLiveNodes: o.maxLiveNodes,
      typewriter: o.typewriter && !o.final,
      typewriterChunk: 2
    }), i = typeof window > "u", c = O(!1), f = B(
      () => J(a.visibleNodes.value, o.final, a.typewriterChars.value)
    ), b = B(() => i || !c.value ? r.value : f.value);
    return Q(() => {
      c.value = !0;
    }), (k, y) => (S(), C("div", _e, [
      (S(!0), C(q, null, X(b.value, (p, h) => (S(), _(H(p), { key: h }))), 128))
    ]));
  }
});
function $(t) {
  try {
    return { ok: !0, value: JSON.parse(t) };
  } catch {
    return { ok: !1, value: null };
  }
}
function K(t) {
  return typeof t;
}
function Z(t) {
  return !!t;
}
let ee = ["table"], M = {};
function Oe(t) {
  const o = t.trim();
  if (o == "")
    return { value: null, valid: !1 };
  const l = $(o);
  if (l.ok)
    return { value: l.value, valid: !0 };
  let r = !1, a = !1, i = [], c = 0;
  for (; c < o.length; ) {
    const k = o[c];
    if (a) {
      a = !1, c += 1;
      continue;
    }
    if (k == "\\") {
      a = !0, c += 1;
      continue;
    }
    if (k == '"') {
      r = !r, c += 1;
      continue;
    }
    if (r) {
      c += 1;
      continue;
    }
    if (k == "{" || k == "[") {
      k == "{" ? i.push("}") : i.push("]"), c += 1;
      continue;
    }
    let y = !1;
    if (k == "}" && (y = !0), k == "]" && (y = !0), y && i.length > 0) {
      const p = i[i.length - 1];
      k == p && i.pop();
    }
    c += 1;
  }
  let f = "";
  r && (f = f + '"'), f = f + i.reverse().join("");
  const b = $(o + f);
  return b.ok ? { value: b.value, valid: !1 } : { value: null, valid: !1 };
}
function Pe(t) {
  let o = [], l = 0;
  for (; l < t.length; ) {
    const r = t.indexOf("```json\n", l);
    if (r == -1)
      break;
    const a = r + 8, i = t.indexOf("\n```", a);
    if (i != -1) {
      const c = i + 4, f = t.slice(a, i);
      o.push({ start: r, end: c, content: f, closed: !0 }), l = c;
    } else {
      const c = t.slice(a);
      o.push({ start: r, end: t.length, content: c, closed: !1 });
      break;
    }
  }
  return o;
}
function Re(t) {
  if (!Z(t) || K(t) != "object")
    return !1;
  const l = t.type;
  return K(l) != "string" ? !1 : ee.includes(l);
}
function De(t) {
  const o = RegExp('"type"\\s*:\\s*"([^"]*)"'), l = t.match(o);
  if (l == null)
    return null;
  const r = l[1];
  for (const a of ee) {
    const i = a.startsWith(r), c = r.startsWith(a);
    if (i || c)
      return a;
  }
  return null;
}
function Me(t) {
  const o = Pe(t);
  let l = [], r = 0;
  for (const a of o) {
    const i = String(a.start);
    a.start > r && l.push({ type: "markdown", text: t.slice(r, a.start) });
    const c = Oe(a.content), f = c.value, b = c.valid, k = De(a.content);
    if (Re(f)) {
      let y = {};
      for (const [p, h] of Object.entries(f))
        p != "type" && (y[p] = h);
      M[i] = y, l.push({ type: "component", componentType: f.type, props: y, final: b && a.closed });
    } else if (k != null) {
      let y = M[i], p = {};
      k == "table" && (p = { columns: [], rows: [] });
      let h = f;
      h == null && (h = y), h == null && (h = p), Z(f) && (M[i] = f), l.push({ type: "component", componentType: k, props: h, final: b && a.closed });
    } else {
      let y = t.slice(a.start, a.end);
      a.closed || (y = y + "\n```"), l.push({ type: "markdown", text: y });
    }
    r = a.end;
  }
  return r < t.length && l.push({ type: "markdown", text: t.slice(r) }), l;
}
function qe(t) {
  return { segments: B(() => Me(t.value)) };
}
const He = {
  key: 2,
  class: "autodown-details",
  "data-details-wrapped": ""
}, ze = { class: "details-content" }, Fe = /* @__PURE__ */ G({
  __name: "StreamingRenderer",
  props: {
    source: {},
    streaming: { type: Boolean, default: !1 },
    placeholderBlockId: {},
    placeholderHeight: {},
    scrollSync: { type: Boolean, default: !0 }
  },
  setup(t, { expose: o }) {
    const l = Te(xe);
    be(), ke(), ye("highlight") || ve();
    const r = t, { segments: a } = qe(B(() => r.source)), i = /^:::details[ \t]+([^\n]*)\n/gm;
    function c(e) {
      const u = [];
      let n = 0;
      i.lastIndex = 0;
      let m;
      for (; (m = i.exec(e)) !== null; ) {
        const d = m.index + m[0].length, s = e.indexOf(`
:::`, d);
        m.index > n && u.push({ kind: "markdown", text: e.slice(n, m.index) }), s === -1 ? (u.push({ kind: "details", summary: m[1], body: e.slice(d), closed: !1 }), n = e.length) : (u.push({ kind: "details", summary: m[1], body: e.slice(d, s), closed: !0 }), n = s + 4, i.lastIndex = n);
      }
      return n < e.length && u.push({ kind: "markdown", text: e.slice(n) }), u;
    }
    const f = B(
      () => a.value.flatMap(
        (e) => e.type === "markdown" ? c(e.text) : [{ kind: "component", componentType: e.componentType, props: e.props, final: e.final }]
      )
    ), b = B(() => {
      for (let e = f.value.length - 1; e >= 0; e--)
        if (f.value[e].kind !== "component") return e;
      return -1;
    }), k = {
      showHeader: !0,
      showCopyButton: !0,
      showExpandButton: !0
    }, p = {
      table: (e) => pe(we, {
        mode: "stream",
        controller: null,
        blockId: "",
        readonly: !0,
        final: e.final ?? !1,
        header_cells: [],
        body_rows: [],
        columns: e.columns,
        rows: e.rows
      })
      // Future: chart: StreamingChart, form: StreamingForm, ...
    };
    function h(e) {
      const u = e.kind === "component" ? e.componentType : e.kind === "details" ? "details" : "";
      return u ? Se(u).stream : void 0;
    }
    function T(e) {
      return e.kind === "component" ? e.props : e;
    }
    function x(e) {
      return e.kind === "component" ? e.final : !r.streaming;
    }
    const g = O(null);
    function te(e) {
      e.querySelectorAll(".node-slot > .autodown-block-placeholder").forEach((u) => u.remove());
    }
    let E = null;
    function ne() {
      return new MutationObserver(() => {
        g.value && (z(g.value), j(g.value), F(g.value));
      });
    }
    function oe(e, u) {
      const n = e.firstElementChild;
      if (!n) return null;
      const m = n.tagName.toLowerCase();
      return ["h1", "h2", "h3", "p", "pre", "blockquote", "ul", "ol", "hr", "img", "table"].includes(m) ? m : n.classList.contains("table-node-wrapper") ? "table" : n.classList.contains("image-error") || n.classList.contains("autodown-image-wrapper") || n.querySelector(".image-node-container, .image-node__img") ? "img" : n.classList.contains("autodown-callout") || n.classList.contains("admonition") ? "callout" : n.classList.contains("autodown-details") || n.classList.contains("html-block-node") ? "details" : n.classList.contains("autodown-math-block") || n.classList.contains("math-block") ? "math" : n.classList.contains("mermaid-block-container") ? "mermaid" : u && u !== "text" ? u : null;
    }
    function re(e) {
      return e === "blockquote" || e === "ul" || e === "ol" || e === "callout" || e === "admonition";
    }
    function z(e) {
      const u = Array.from(e.querySelectorAll(".node-slot")), n = [];
      u.forEach((d) => {
        const s = d.querySelector(".node-content");
        s && (s.removeAttribute("data-block-id"), s.removeAttribute("data-block-index"));
      });
      const m = e.getBoundingClientRect();
      if (u.forEach((d) => {
        const s = d.querySelector(".node-content");
        if (!s) return;
        const v = d.getAttribute("data-node-type"), w = oe(s, v);
        if (!w) return;
        const I = d.getBoundingClientRect(), N = I.top - m.top, L = I.height;
        if (n.some((A) => re(A.type) ? N >= A.top && N < A.top + A.height : !1)) return;
        const D = n[n.length - 1];
        D && N === D.top && L === D.height || n.push({ slot: d, content: s, type: w, top: N, height: L });
      }), n.forEach(({ slot: d, content: s }, v) => {
        const w = `block-${v}`;
        s.setAttribute("data-block-id", w), s.setAttribute("data-block-index", String(v)), d.setAttribute("data-block-slot-id", w);
      }), r.placeholderBlockId != null && r.placeholderHeight != null) {
        const d = n[Number(r.placeholderBlockId.replace("block-", ""))];
        if (d && !d.slot.querySelector(":scope > .autodown-block-placeholder")) {
          const v = document.createElement("div");
          v.className = "autodown-block-placeholder", v.style.height = `${r.placeholderHeight}px`, d.slot.insertBefore(v, d.slot.firstChild);
        }
      }
    }
    async function le() {
      g.value && (await me(), te(g.value), z(g.value), j(g.value), F(g.value));
    }
    function F(e) {
      Array.from(
        e.querySelectorAll("details:not([data-details-wrapped])")
      ).forEach((n) => {
        const m = Array.from(n.children).filter((s) => {
          const v = s.tagName.toLowerCase();
          return v !== "summary" && v !== "details" && !s.classList.contains("details-content");
        });
        if (m.length === 0) return;
        const d = document.createElement("div");
        d.className = "details-content", m.forEach((s) => d.appendChild(s)), n.appendChild(d), n.setAttribute("data-details-wrapped", "");
      });
    }
    function W(e) {
      var w, P, I, N;
      const u = e.target, n = (L) => {
        var R;
        return L.closest("pre") ?? ((R = L.closest(".code-block-container")) == null ? void 0 : R.querySelector("pre[data-language]")) ?? null;
      }, m = (w = u.closest) == null ? void 0 : w.call(u, "[data-codeblock-expand-btn]");
      if (m && g.value) {
        e.preventDefault(), e.stopPropagation(), (P = n(m)) == null || P.classList.toggle("is-collapsed");
        return;
      }
      const d = (I = u.closest) == null ? void 0 : I.call(u, "[data-codeblock-copy-btn]");
      if (!d || !g.value) return;
      const s = n(d), v = ((N = s == null ? void 0 : s.querySelector("code")) == null ? void 0 : N.textContent) ?? "";
      e.preventDefault(), e.stopPropagation(), navigator.clipboard.writeText(v);
    }
    function j(e) {
      Array.from(e.querySelectorAll("pre[data-language] > code")).forEach((n) => {
        const d = n.parentElement.getAttribute("data-language"), s = d === "plaintext" ? "text" : d;
        if (!s || s === "text" || n.getAttribute("data-highlighted") === s || !l.registered(s)) return;
        const v = n.textContent || "";
        if (v)
          try {
            const w = l.highlight(s, v);
            n.innerHTML = Ne(w), n.setAttribute("data-highlighted", s);
          } catch {
          }
      });
    }
    return U(
      () => [a.value, r.placeholderBlockId, r.placeholderHeight],
      () => le(),
      { deep: !0, flush: "post" }
    ), Q(() => {
      g.value && (E = ne(), E.observe(g.value, { childList: !0, subtree: !0 }), g.value.addEventListener("click", W, { capture: !0 }));
    }), ie(() => {
      var e;
      E == null || E.disconnect(), (e = g.value) == null || e.removeEventListener("click", W, { capture: !0 });
    }), o({
      containerRef: g
    }), (e, u) => (S(), C("div", {
      ref_key: "containerRef",
      ref: g,
      class: se(["streaming-document", { "is-sync": t.scrollSync }])
    }, [
      (S(!0), C(q, null, X(f.value, (n, m) => (S(), C(q, {
        key: n.kind + "-" + m
      }, [
        n.kind === "markdown" ? (S(), _(Y, {
          key: 0,
          content: n.text,
          final: !t.streaming,
          "max-live-nodes": t.streaming ? 0 : 320,
          "batch-rendering": t.streaming,
          "render-batch-size": 16,
          "render-batch-delay": 8,
          typewriter: t.streaming && m === b.value,
          fade: !1,
          "code-block-props": k
        }, null, 8, ["content", "final", "max-live-nodes", "batch-rendering", "typewriter"])) : h(n) ? (S(), _(H(() => h(n)(T(n), x(n))), { key: 1 })) : n.kind === "details" ? (S(), C("details", He, [
          V("summary", null, ce(n.summary), 1),
          V("div", ze, [
            ue(Y, {
              content: n.body,
              final: !t.streaming,
              "batch-rendering": t.streaming,
              "render-batch-size": 16,
              "render-batch-delay": 8,
              typewriter: t.streaming && m === b.value,
              fade: !1,
              "code-block-props": k
            }, null, 8, ["content", "final", "batch-rendering", "typewriter"])
          ])
        ])) : n.kind === "component" ? (S(), _(H(p[n.componentType]), de({
          key: 3,
          ref_for: !0
        }, n.props, {
          final: n.final
        }), null, 16, ["final"])) : fe("", !0)
      ], 64))), 128))
    ], 2));
  }
}), $e = /* @__PURE__ */ Be(Fe, [["__scopeId", "data-v-f5218f7d"]]);
export {
  $e as S,
  Y as _,
  qe as u
};
