import { ref as D, watch as X, computed as w, onScopeDispose as ue, defineComponent as Z, onMounted as F, openBlock as B, createElementBlock as L, Fragment as H, renderList as ee, createBlock as P, resolveDynamicComponent as z, onBeforeUnmount as Y, normalizeClass as de, createElementVNode as $, toDisplayString as fe, createVNode as me, mergeProps as pe, createCommentVNode as he, nextTick as ge } from "vue";
import { p as ke, s as ve } from "./parser-BfX0E-c9.js";
import { F as K, x as ye, G as be, j as we, k as Se, g as Be, h as Te, z as Ce, _ as Ne } from "./render-node-jJKN3LNR.js";
import { createLowlight as xe, common as Ee } from "lowlight";
import { toHtml as Le } from "hast-util-to-html";
function Ie(t, o, a) {
  let r = o - t;
  return r <= 0 ? t : a <= 0 ? o : r > a ? t + a : o;
}
function Ae(t, o) {
  if (o <= 0)
    return 0;
  let a = t - o;
  return a < 0 ? 0 : a;
}
function _e(t, o, a) {
  if (o - t <= 0)
    return o;
  let l = t + a;
  return l > o ? o : l;
}
const Oe = {
  setTimeout: (t, o) => setTimeout(t, o),
  clearTimeout: (t) => clearTimeout(t)
};
function De(t, o) {
  const a = o.timer ?? Oe, r = D(t.value.length), l = D(Number.POSITIVE_INFINITY);
  let s;
  function c(m) {
    s !== void 0 && a.clearTimeout(s), s = a.setTimeout(() => {
      s = void 0, m();
    }, o.batchDelay);
  }
  function h(m) {
    return m ? m.type === "text" ? String(m.content ?? "") : (m.children ?? []).map((T) => h(T)).join("") : "";
  }
  function b() {
    const m = t.value[t.value.length - 1], p = h(m).length;
    if (p <= 0) {
      l.value = Number.POSITIVE_INFINITY;
      return;
    }
    l.value = 0;
    const T = () => {
      const N = _e(l.value, p, o.typewriterChunk);
      l.value = N, N < p && c(T);
    };
    c(T);
  }
  X(
    t,
    (m) => {
      if (s !== void 0 && (a.clearTimeout(s), s = void 0), !o.enabled) {
        r.value = m.length, l.value = Number.POSITIVE_INFINITY;
        return;
      }
      const p = m.length, T = Math.min(p, Math.max(1, Math.floor(o.batchSize / 4) || 1));
      r.value = Math.max(r.value, T);
      const N = () => {
        const I = Ie(r.value, p, o.batchSize);
        r.value = I, I < p ? c(N) : o.typewriter && b();
      };
      r.value < p ? c(N) : o.typewriter && b();
    },
    { immediate: !0 }
  );
  const g = w(() => Ae(r.value, o.maxLiveNodes)), v = w(() => t.value.slice(g.value, r.value));
  return ue(() => {
    s !== void 0 && a.clearTimeout(s);
  }), { visibleNodes: v, visibleCount: r, typewriterChars: l, windowStart: g };
}
const Me = { class: "markdown-renderer" }, G = /* @__PURE__ */ Z({
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
    const o = t, a = w(() => ke(ve(o.content ?? ""), o.final)), r = w(() => K(a.value, o.final)), l = De(a, {
      enabled: o.batchRendering,
      batchSize: o.renderBatchSize,
      batchDelay: o.renderBatchDelay,
      maxLiveNodes: o.maxLiveNodes,
      typewriter: o.typewriter && !o.final,
      typewriterChunk: 2
    }), s = typeof window > "u", c = D(!1), h = w(
      () => K(l.visibleNodes.value, o.final, l.typewriterChars.value)
    ), b = w(() => s || !c.value ? r.value : h.value);
    return F(() => {
      c.value = !0;
    }), (g, v) => (B(), L("div", Me, [
      (B(!0), L(H, null, ee(b.value, (m, p) => (B(), P(z(m), { key: p }))), 128))
    ]));
  }
});
function U(t) {
  try {
    return { ok: !0, value: JSON.parse(t) };
  } catch {
    return { ok: !1, value: null };
  }
}
function Q(t) {
  return typeof t;
}
function te(t) {
  return !!t;
}
let ne = ["table"], q = {};
function Pe(t) {
  const o = t.trim();
  if (o == "")
    return { value: null, valid: !1 };
  const a = U(o);
  if (a.ok)
    return { value: a.value, valid: !0 };
  let r = !1, l = !1, s = [], c = 0;
  for (; c < o.length; ) {
    const g = o[c];
    if (l) {
      l = !1, c += 1;
      continue;
    }
    if (g == "\\") {
      l = !0, c += 1;
      continue;
    }
    if (g == '"') {
      r = !r, c += 1;
      continue;
    }
    if (r) {
      c += 1;
      continue;
    }
    if (g == "{" || g == "[") {
      g == "{" ? s.push("}") : s.push("]"), c += 1;
      continue;
    }
    let v = !1;
    if (g == "}" && (v = !0), g == "]" && (v = !0), v && s.length > 0) {
      const m = s[s.length - 1];
      g == m && s.pop();
    }
    c += 1;
  }
  let h = "";
  r && (h = h + '"'), h = h + s.reverse().join("");
  const b = U(o + h);
  return b.ok ? { value: b.value, valid: !1 } : { value: null, valid: !1 };
}
function Re(t) {
  let o = [], a = 0;
  for (; a < t.length; ) {
    const r = t.indexOf("```json\n", a);
    if (r == -1)
      break;
    const l = r + 8, s = t.indexOf("\n```", l);
    if (s != -1) {
      const c = s + 4, h = t.slice(l, s);
      o.push({ start: r, end: c, content: h, closed: !0 }), a = c;
    } else {
      const c = t.slice(l);
      o.push({ start: r, end: t.length, content: c, closed: !1 });
      break;
    }
  }
  return o;
}
function qe(t) {
  if (!te(t) || Q(t) != "object")
    return !1;
  const a = t.type;
  return Q(a) != "string" ? !1 : ne.includes(a);
}
function Fe(t) {
  const o = RegExp('"type"\\s*:\\s*"([^"]*)"'), a = t.match(o);
  if (a == null)
    return null;
  const r = a[1];
  for (const l of ne) {
    const s = l.startsWith(r), c = r.startsWith(l);
    if (s || c)
      return l;
  }
  return null;
}
function He(t) {
  const o = Re(t);
  let a = [], r = 0;
  for (const l of o) {
    const s = String(l.start);
    l.start > r && a.push({ type: "markdown", text: t.slice(r, l.start) });
    const c = Pe(l.content), h = c.value, b = c.valid, g = Fe(l.content);
    if (qe(h)) {
      let v = {};
      for (const [m, p] of Object.entries(h))
        m != "type" && (v[m] = p);
      q[s] = v, a.push({ type: "component", componentType: h.type, props: v, final: b && l.closed });
    } else if (g != null) {
      let v = q[s], m = {};
      g == "table" && (m = { columns: [], rows: [] });
      let p = h;
      p == null && (p = v), p == null && (p = m), te(h) && (q[s] = h), a.push({ type: "component", componentType: g, props: p, final: b && l.closed });
    } else {
      let v = t.slice(l.start, l.end);
      l.closed || (v = v + "\n```"), a.push({ type: "markdown", text: v });
    }
    r = l.end;
  }
  return r < t.length && a.push({ type: "markdown", text: t.slice(r) }), a;
}
function ze(t) {
  return { segments: w(() => He(t.value)) };
}
const je = ["data-accent"], Ve = {
  key: 2,
  class: "autodown-details",
  "data-details-wrapped": ""
}, We = { class: "details-content" };
ye("Table", { stream: be });
const Je = /* @__PURE__ */ Z({
  __name: "StreamingRenderer",
  props: {
    source: {},
    streaming: { type: Boolean, default: !1 },
    placeholderBlockId: {},
    placeholderHeight: {},
    scrollSync: { type: Boolean, default: !1 },
    darkMode: { type: Boolean, default: !1 },
    accent: { default: "indigo" }
  },
  setup(t, { expose: o }) {
    const a = xe(Ee);
    Te(), we(), Se("highlight") || Be();
    const r = t, l = ["indigo", "coral", "ocean", "sage", "amber"], s = w(
      () => l.includes(r.accent) ? r.accent : "indigo"
    ), c = D(!1), h = w(() => r.darkMode || c.value);
    F(() => {
      var e;
      if (typeof document < "u") {
        const i = () => {
          var f;
          c.value = !!(document.documentElement.classList.contains("dark") || document.documentElement.getAttribute("data-theme") === "dark" || (f = k.value) != null && f.closest('.dark, [data-theme="dark"]'));
        };
        i();
        const n = new MutationObserver(i);
        n.observe(document.documentElement, { attributes: !0, attributeFilter: ["class", "data-theme"] }), (e = k.value) != null && e.parentElement && n.observe(k.value.parentElement, { attributes: !0, attributeFilter: ["class", "data-theme"] }), Y(() => n.disconnect());
      }
    });
    const { segments: b } = ze(w(() => r.source)), g = /^:::details[ \t]+([^\n]*)\n/gm;
    function v(e) {
      const i = [];
      let n = 0;
      g.lastIndex = 0;
      let f;
      for (; (f = g.exec(e)) !== null; ) {
        const d = f.index + f[0].length, u = e.indexOf(`
:::`, d);
        f.index > n && i.push({ kind: "markdown", text: e.slice(n, f.index) }), u === -1 ? (i.push({ kind: "details", summary: f[1], body: e.slice(d), closed: !1 }), n = e.length) : (i.push({ kind: "details", summary: f[1], body: e.slice(d, u), closed: !0 }), n = u + 4, g.lastIndex = n);
      }
      return n < e.length && i.push({ kind: "markdown", text: e.slice(n) }), i;
    }
    const m = w(
      () => b.value.flatMap(
        (e) => e.type === "markdown" ? v(e.text) : [{ kind: "component", componentType: e.componentType, props: e.props, final: e.final }]
      )
    ), p = w(() => {
      for (let e = m.value.length - 1; e >= 0; e--)
        if (m.value[e].kind !== "component") return e;
      return -1;
    }), T = {
      showHeader: !0,
      showCopyButton: !0,
      showExpandButton: !0
    }, N = {
      // Future: chart: StreamingChart, form: StreamingForm, ...
    };
    function I(e) {
      const i = e.kind === "component" ? e.componentType : e.kind === "details" ? "details" : "";
      return i ? Ce(i).stream : void 0;
    }
    function oe(e) {
      return e.kind === "component" ? e.props : e;
    }
    function re(e) {
      return e.kind === "component" ? e.final : !r.streaming;
    }
    const k = D(null);
    function ae(e) {
      e.querySelectorAll(".node-slot > .autodown-block-placeholder").forEach((i) => i.remove());
    }
    let A = null;
    function le() {
      return new MutationObserver(() => {
        k.value && (j(k.value), J(k.value), V(k.value));
      });
    }
    function ie(e, i) {
      const n = e.firstElementChild;
      if (!n) return null;
      const f = n.tagName.toLowerCase();
      return ["h1", "h2", "h3", "p", "pre", "blockquote", "ul", "ol", "hr", "img", "table"].includes(f) ? f : n.classList.contains("table-node-wrapper") ? "table" : n.classList.contains("image-error") || n.classList.contains("autodown-image-wrapper") || n.querySelector(".image-node-container, .image-node__img") ? "img" : n.classList.contains("autodown-callout") || n.classList.contains("admonition") ? "callout" : n.classList.contains("autodown-details") || n.classList.contains("html-block-node") ? "details" : n.classList.contains("autodown-math-block") || n.classList.contains("math-block") ? "math" : n.classList.contains("mermaid-block-container") ? "mermaid" : i && i !== "text" ? i : null;
    }
    function se(e) {
      return e === "blockquote" || e === "ul" || e === "ol" || e === "callout" || e === "admonition";
    }
    function j(e) {
      const i = Array.from(e.querySelectorAll(".node-slot")), n = [];
      i.forEach((d) => {
        const u = d.querySelector(".node-content");
        u && (u.removeAttribute("data-block-id"), u.removeAttribute("data-block-index"));
      });
      const f = e.getBoundingClientRect();
      if (i.forEach((d) => {
        const u = d.querySelector(".node-content");
        if (!u) return;
        const y = d.getAttribute("data-node-type"), S = ie(u, y);
        if (!S) return;
        const _ = d.getBoundingClientRect(), x = _.top - f.top, O = _.height;
        if (n.some((M) => se(M.type) ? x >= M.top && x < M.top + M.height : !1)) return;
        const C = n[n.length - 1];
        C && x === C.top && O === C.height || n.push({ slot: d, content: u, type: S, top: x, height: O });
      }), n.forEach(({ slot: d, content: u }, y) => {
        const S = `block-${y}`;
        u.setAttribute("data-block-id", S), u.setAttribute("data-block-index", String(y)), d.setAttribute("data-block-slot-id", S);
      }), r.placeholderBlockId != null && r.placeholderHeight != null) {
        const d = n[Number(r.placeholderBlockId.replace("block-", ""))];
        if (d && !d.slot.querySelector(":scope > .autodown-block-placeholder")) {
          const y = document.createElement("div");
          y.className = "autodown-block-placeholder", y.style.height = `${r.placeholderHeight}px`, d.slot.insertBefore(y, d.slot.firstChild);
        }
      }
    }
    async function ce() {
      k.value && (await ge(), ae(k.value), j(k.value), J(k.value), V(k.value));
    }
    function V(e) {
      Array.from(
        e.querySelectorAll("details:not([data-details-wrapped])")
      ).forEach((n) => {
        const f = Array.from(n.children).filter((u) => {
          const y = u.tagName.toLowerCase();
          return y !== "summary" && y !== "details" && !u.classList.contains("details-content");
        });
        if (f.length === 0) return;
        const d = document.createElement("div");
        d.className = "details-content", f.forEach((u) => d.appendChild(u)), n.appendChild(d), n.setAttribute("data-details-wrapped", "");
      });
    }
    function W(e) {
      var y, S, R, _, x, O;
      const i = e.target, n = (E) => {
        var C;
        return E.closest("pre") ?? ((C = E.closest(".code-block-container")) == null ? void 0 : C.querySelector("pre[data-language]")) ?? null;
      }, f = (y = i.closest) == null ? void 0 : y.call(i, "[data-codeblock-copy-btn]");
      if (f && k.value) {
        const E = n(f), C = ((S = E == null ? void 0 : E.querySelector("code")) == null ? void 0 : S.textContent) ?? "";
        e.preventDefault(), e.stopPropagation(), navigator.clipboard.writeText(C);
        return;
      }
      const d = (R = i.closest) == null ? void 0 : R.call(i, ".code-block-header"), u = (_ = i.closest) == null ? void 0 : _.call(i, "[data-codeblock-language-badge]");
      d && !u && k.value && (e.preventDefault(), e.stopPropagation(), (O = (x = d.closest(".code-block-container")) == null ? void 0 : x.querySelector("pre[data-language]")) == null || O.classList.toggle("is-collapsed"));
    }
    function J(e) {
      Array.from(e.querySelectorAll("pre[data-language] > code")).forEach((n) => {
        const d = n.parentElement.getAttribute("data-language"), u = d === "plaintext" ? "text" : d;
        if (!u || u === "text" || n.getAttribute("data-highlighted") === u || !a.registered(u)) return;
        const y = n.textContent || "";
        if (y)
          try {
            const S = a.highlight(u, y);
            n.innerHTML = Le(S), n.setAttribute("data-highlighted", u);
          } catch {
          }
      });
    }
    return X(
      () => [b.value, r.placeholderBlockId, r.placeholderHeight],
      () => ce(),
      { deep: !0, flush: "post" }
    ), F(() => {
      k.value && (A = le(), A.observe(k.value, { childList: !0, subtree: !0 }), k.value.addEventListener("click", W, { capture: !0 }));
    }), Y(() => {
      var e;
      A == null || A.disconnect(), (e = k.value) == null || e.removeEventListener("click", W, { capture: !0 });
    }), o({
      containerRef: k
    }), (e, i) => (B(), L("div", {
      ref_key: "containerRef",
      ref: k,
      class: de(["streaming-document", { "is-sync": t.scrollSync, "is-dark": h.value }]),
      "data-accent": s.value
    }, [
      (B(!0), L(H, null, ee(m.value, (n, f) => (B(), L(H, {
        key: n.kind + "-" + f
      }, [
        n.kind === "markdown" ? (B(), P(G, {
          key: 0,
          content: n.text,
          final: !t.streaming,
          "max-live-nodes": t.streaming ? 0 : 320,
          "batch-rendering": t.streaming,
          "render-batch-size": 16,
          "render-batch-delay": 8,
          typewriter: t.streaming && f === p.value,
          fade: !1,
          "code-block-props": T
        }, null, 8, ["content", "final", "max-live-nodes", "batch-rendering", "typewriter"])) : I(n) ? (B(), P(z(() => I(n)(oe(n), re(n))), { key: 1 })) : n.kind === "details" ? (B(), L("details", Ve, [
          $("summary", null, fe(n.summary), 1),
          $("div", We, [
            me(G, {
              content: n.body,
              final: !t.streaming,
              "batch-rendering": t.streaming,
              "render-batch-size": 16,
              "render-batch-delay": 8,
              typewriter: t.streaming && f === p.value,
              fade: !1,
              "code-block-props": T
            }, null, 8, ["content", "final", "batch-rendering", "typewriter"])
          ])
        ])) : n.kind === "component" ? (B(), P(z(N[n.componentType]), pe({
          key: 3,
          ref_for: !0
        }, n.props, {
          final: n.final
        }), null, 16, ["final"])) : he("", !0)
      ], 64))), 128))
    ], 10, je));
  }
}), Qe = /* @__PURE__ */ Ne(Je, [["__scopeId", "data-v-514af75b"]]);
export {
  Qe as S,
  G as _,
  ze as u
};
