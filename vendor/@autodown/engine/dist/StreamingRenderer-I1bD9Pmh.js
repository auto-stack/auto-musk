import { ref as R, watch as U, computed as w, onScopeDispose as ie, defineComponent as Q, onMounted as X, openBlock as B, createElementBlock as I, Fragment as H, renderList as Z, createBlock as P, resolveDynamicComponent as z, onBeforeUnmount as ce, normalizeClass as se, createElementVNode as J, toDisplayString as ue, createVNode as de, mergeProps as fe, createCommentVNode as pe, nextTick as me } from "vue";
import { p as he, s as ge } from "./parser-BfX0E-c9.js";
import { F as Y, x as ke, G as ye, j as ve, k as be, g as we, h as Se, z as Be, _ as Te } from "./render-node-Di_WXNK8.js";
import { createLowlight as Ce, common as Ne } from "lowlight";
import { toHtml as xe } from "hast-util-to-html";
function Ee(e, o, l) {
  let r = o - e;
  return r <= 0 ? e : l <= 0 ? o : r > l ? e + l : o;
}
function Ie(e, o) {
  if (o <= 0)
    return 0;
  let l = e - o;
  return l < 0 ? 0 : l;
}
function Le(e, o, l) {
  if (o - e <= 0)
    return o;
  let a = e + l;
  return a > o ? o : a;
}
const Ae = {
  setTimeout: (e, o) => setTimeout(e, o),
  clearTimeout: (e) => clearTimeout(e)
};
function Oe(e, o) {
  const l = o.timer ?? Ae, r = R(e.value.length), a = R(Number.POSITIVE_INFINITY);
  let i;
  function s(f) {
    i !== void 0 && l.clearTimeout(i), i = l.setTimeout(() => {
      i = void 0, f();
    }, o.batchDelay);
  }
  function p(f) {
    return f ? f.type === "text" ? String(f.content ?? "") : (f.children ?? []).map((T) => p(T)).join("") : "";
  }
  function b() {
    const f = e.value[e.value.length - 1], h = p(f).length;
    if (h <= 0) {
      a.value = Number.POSITIVE_INFINITY;
      return;
    }
    a.value = 0;
    const T = () => {
      const N = Le(a.value, h, o.typewriterChunk);
      a.value = N, N < h && s(T);
    };
    s(T);
  }
  U(
    e,
    (f) => {
      if (i !== void 0 && (l.clearTimeout(i), i = void 0), !o.enabled) {
        r.value = f.length, a.value = Number.POSITIVE_INFINITY;
        return;
      }
      const h = f.length, T = Math.min(h, Math.max(1, Math.floor(o.batchSize / 4) || 1));
      r.value = Math.max(r.value, T);
      const N = () => {
        const _ = Ee(r.value, h, o.batchSize);
        r.value = _, _ < h ? s(N) : o.typewriter && b();
      };
      r.value < h ? s(N) : o.typewriter && b();
    },
    { immediate: !0 }
  );
  const g = w(() => Ie(r.value, o.maxLiveNodes)), k = w(() => e.value.slice(g.value, r.value));
  return ie(() => {
    i !== void 0 && l.clearTimeout(i);
  }), { visibleNodes: k, visibleCount: r, typewriterChars: a, windowStart: g };
}
const _e = { class: "markdown-renderer" }, $ = /* @__PURE__ */ Q({
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
  setup(e) {
    const o = e, l = w(() => he(ge(o.content ?? ""), o.final)), r = w(() => Y(l.value, o.final)), a = Oe(l, {
      enabled: o.batchRendering,
      batchSize: o.renderBatchSize,
      batchDelay: o.renderBatchDelay,
      maxLiveNodes: o.maxLiveNodes,
      typewriter: o.typewriter && !o.final,
      typewriterChunk: 2
    }), i = typeof window > "u", s = R(!1), p = w(
      () => Y(a.visibleNodes.value, o.final, a.typewriterChars.value)
    ), b = w(() => i || !s.value ? r.value : p.value);
    return X(() => {
      s.value = !0;
    }), (g, k) => (B(), I("div", _e, [
      (B(!0), I(H, null, Z(b.value, (f, h) => (B(), P(z(f), { key: h }))), 128))
    ]));
  }
});
function K(e) {
  try {
    return { ok: !0, value: JSON.parse(e) };
  } catch {
    return { ok: !1, value: null };
  }
}
function G(e) {
  return typeof e;
}
function ee(e) {
  return !!e;
}
let te = ["table"], q = {};
function Me(e) {
  const o = e.trim();
  if (o == "")
    return { value: null, valid: !1 };
  const l = K(o);
  if (l.ok)
    return { value: l.value, valid: !0 };
  let r = !1, a = !1, i = [], s = 0;
  for (; s < o.length; ) {
    const g = o[s];
    if (a) {
      a = !1, s += 1;
      continue;
    }
    if (g == "\\") {
      a = !0, s += 1;
      continue;
    }
    if (g == '"') {
      r = !r, s += 1;
      continue;
    }
    if (r) {
      s += 1;
      continue;
    }
    if (g == "{" || g == "[") {
      g == "{" ? i.push("}") : i.push("]"), s += 1;
      continue;
    }
    let k = !1;
    if (g == "}" && (k = !0), g == "]" && (k = !0), k && i.length > 0) {
      const f = i[i.length - 1];
      g == f && i.pop();
    }
    s += 1;
  }
  let p = "";
  r && (p = p + '"'), p = p + i.reverse().join("");
  const b = K(o + p);
  return b.ok ? { value: b.value, valid: !1 } : { value: null, valid: !1 };
}
function Pe(e) {
  let o = [], l = 0;
  for (; l < e.length; ) {
    const r = e.indexOf("```json\n", l);
    if (r == -1)
      break;
    const a = r + 8, i = e.indexOf("\n```", a);
    if (i != -1) {
      const s = i + 4, p = e.slice(a, i);
      o.push({ start: r, end: s, content: p, closed: !0 }), l = s;
    } else {
      const s = e.slice(a);
      o.push({ start: r, end: e.length, content: s, closed: !1 });
      break;
    }
  }
  return o;
}
function Re(e) {
  if (!ee(e) || G(e) != "object")
    return !1;
  const l = e.type;
  return G(l) != "string" ? !1 : te.includes(l);
}
function De(e) {
  const o = RegExp('"type"\\s*:\\s*"([^"]*)"'), l = e.match(o);
  if (l == null)
    return null;
  const r = l[1];
  for (const a of te) {
    const i = a.startsWith(r), s = r.startsWith(a);
    if (i || s)
      return a;
  }
  return null;
}
function qe(e) {
  const o = Pe(e);
  let l = [], r = 0;
  for (const a of o) {
    const i = String(a.start);
    a.start > r && l.push({ type: "markdown", text: e.slice(r, a.start) });
    const s = Me(a.content), p = s.value, b = s.valid, g = De(a.content);
    if (Re(p)) {
      let k = {};
      for (const [f, h] of Object.entries(p))
        f != "type" && (k[f] = h);
      q[i] = k, l.push({ type: "component", componentType: p.type, props: k, final: b && a.closed });
    } else if (g != null) {
      let k = q[i], f = {};
      g == "table" && (f = { columns: [], rows: [] });
      let h = p;
      h == null && (h = k), h == null && (h = f), ee(p) && (q[i] = p), l.push({ type: "component", componentType: g, props: h, final: b && a.closed });
    } else {
      let k = e.slice(a.start, a.end);
      a.closed || (k = k + "\n```"), l.push({ type: "markdown", text: k });
    }
    r = a.end;
  }
  return r < e.length && l.push({ type: "markdown", text: e.slice(r) }), l;
}
function He(e) {
  return { segments: w(() => qe(e.value)) };
}
const ze = ["data-accent"], Fe = {
  key: 2,
  class: "autodown-details",
  "data-details-wrapped": ""
}, je = { class: "details-content" };
ke("Table", { stream: ye });
const Ve = /* @__PURE__ */ Q({
  __name: "StreamingRenderer",
  props: {
    source: {},
    streaming: { type: Boolean, default: !1 },
    placeholderBlockId: {},
    placeholderHeight: {},
    scrollSync: { type: Boolean, default: !0 },
    darkMode: { type: Boolean, default: !1 },
    accent: { default: "indigo" }
  },
  setup(e, { expose: o }) {
    const l = Ce(Ne);
    Se(), ve(), be("highlight") || we();
    const r = e, a = ["indigo", "coral", "ocean", "sage", "amber"], i = w(
      () => a.includes(r.accent) ? r.accent : "indigo"
    ), { segments: s } = He(w(() => r.source)), p = /^:::details[ \t]+([^\n]*)\n/gm;
    function b(t) {
      const c = [];
      let n = 0;
      p.lastIndex = 0;
      let m;
      for (; (m = p.exec(t)) !== null; ) {
        const d = m.index + m[0].length, u = t.indexOf(`
:::`, d);
        m.index > n && c.push({ kind: "markdown", text: t.slice(n, m.index) }), u === -1 ? (c.push({ kind: "details", summary: m[1], body: t.slice(d), closed: !1 }), n = t.length) : (c.push({ kind: "details", summary: m[1], body: t.slice(d, u), closed: !0 }), n = u + 4, p.lastIndex = n);
      }
      return n < t.length && c.push({ kind: "markdown", text: t.slice(n) }), c;
    }
    const g = w(
      () => s.value.flatMap(
        (t) => t.type === "markdown" ? b(t.text) : [{ kind: "component", componentType: t.componentType, props: t.props, final: t.final }]
      )
    ), k = w(() => {
      for (let t = g.value.length - 1; t >= 0; t--)
        if (g.value[t].kind !== "component") return t;
      return -1;
    }), f = {
      showHeader: !0,
      showCopyButton: !0,
      showExpandButton: !0
    }, h = {
      // Future: chart: StreamingChart, form: StreamingForm, ...
    };
    function T(t) {
      const c = t.kind === "component" ? t.componentType : t.kind === "details" ? "details" : "";
      return c ? Be(c).stream : void 0;
    }
    function N(t) {
      return t.kind === "component" ? t.props : t;
    }
    function _(t) {
      return t.kind === "component" ? t.final : !r.streaming;
    }
    const y = R(null);
    function ne(t) {
      t.querySelectorAll(".node-slot > .autodown-block-placeholder").forEach((c) => c.remove());
    }
    let L = null;
    function oe() {
      return new MutationObserver(() => {
        y.value && (F(y.value), W(y.value), j(y.value));
      });
    }
    function re(t, c) {
      const n = t.firstElementChild;
      if (!n) return null;
      const m = n.tagName.toLowerCase();
      return ["h1", "h2", "h3", "p", "pre", "blockquote", "ul", "ol", "hr", "img", "table"].includes(m) ? m : n.classList.contains("table-node-wrapper") ? "table" : n.classList.contains("image-error") || n.classList.contains("autodown-image-wrapper") || n.querySelector(".image-node-container, .image-node__img") ? "img" : n.classList.contains("autodown-callout") || n.classList.contains("admonition") ? "callout" : n.classList.contains("autodown-details") || n.classList.contains("html-block-node") ? "details" : n.classList.contains("autodown-math-block") || n.classList.contains("math-block") ? "math" : n.classList.contains("mermaid-block-container") ? "mermaid" : c && c !== "text" ? c : null;
    }
    function le(t) {
      return t === "blockquote" || t === "ul" || t === "ol" || t === "callout" || t === "admonition";
    }
    function F(t) {
      const c = Array.from(t.querySelectorAll(".node-slot")), n = [];
      c.forEach((d) => {
        const u = d.querySelector(".node-content");
        u && (u.removeAttribute("data-block-id"), u.removeAttribute("data-block-index"));
      });
      const m = t.getBoundingClientRect();
      if (c.forEach((d) => {
        const u = d.querySelector(".node-content");
        if (!u) return;
        const v = d.getAttribute("data-node-type"), S = re(u, v);
        if (!S) return;
        const A = d.getBoundingClientRect(), x = A.top - m.top, O = A.height;
        if (n.some((M) => le(M.type) ? x >= M.top && x < M.top + M.height : !1)) return;
        const C = n[n.length - 1];
        C && x === C.top && O === C.height || n.push({ slot: d, content: u, type: S, top: x, height: O });
      }), n.forEach(({ slot: d, content: u }, v) => {
        const S = `block-${v}`;
        u.setAttribute("data-block-id", S), u.setAttribute("data-block-index", String(v)), d.setAttribute("data-block-slot-id", S);
      }), r.placeholderBlockId != null && r.placeholderHeight != null) {
        const d = n[Number(r.placeholderBlockId.replace("block-", ""))];
        if (d && !d.slot.querySelector(":scope > .autodown-block-placeholder")) {
          const v = document.createElement("div");
          v.className = "autodown-block-placeholder", v.style.height = `${r.placeholderHeight}px`, d.slot.insertBefore(v, d.slot.firstChild);
        }
      }
    }
    async function ae() {
      y.value && (await me(), ne(y.value), F(y.value), W(y.value), j(y.value));
    }
    function j(t) {
      Array.from(
        t.querySelectorAll("details:not([data-details-wrapped])")
      ).forEach((n) => {
        const m = Array.from(n.children).filter((u) => {
          const v = u.tagName.toLowerCase();
          return v !== "summary" && v !== "details" && !u.classList.contains("details-content");
        });
        if (m.length === 0) return;
        const d = document.createElement("div");
        d.className = "details-content", m.forEach((u) => d.appendChild(u)), n.appendChild(d), n.setAttribute("data-details-wrapped", "");
      });
    }
    function V(t) {
      var v, S, D, A, x, O;
      const c = t.target, n = (E) => {
        var C;
        return E.closest("pre") ?? ((C = E.closest(".code-block-container")) == null ? void 0 : C.querySelector("pre[data-language]")) ?? null;
      }, m = (v = c.closest) == null ? void 0 : v.call(c, "[data-codeblock-copy-btn]");
      if (m && y.value) {
        const E = n(m), C = ((S = E == null ? void 0 : E.querySelector("code")) == null ? void 0 : S.textContent) ?? "";
        t.preventDefault(), t.stopPropagation(), navigator.clipboard.writeText(C);
        return;
      }
      const d = (D = c.closest) == null ? void 0 : D.call(c, ".code-block-header"), u = (A = c.closest) == null ? void 0 : A.call(c, "[data-codeblock-language-badge]");
      d && !u && y.value && (t.preventDefault(), t.stopPropagation(), (O = (x = d.closest(".code-block-container")) == null ? void 0 : x.querySelector("pre[data-language]")) == null || O.classList.toggle("is-collapsed"));
    }
    function W(t) {
      Array.from(t.querySelectorAll("pre[data-language] > code")).forEach((n) => {
        const d = n.parentElement.getAttribute("data-language"), u = d === "plaintext" ? "text" : d;
        if (!u || u === "text" || n.getAttribute("data-highlighted") === u || !l.registered(u)) return;
        const v = n.textContent || "";
        if (v)
          try {
            const S = l.highlight(u, v);
            n.innerHTML = xe(S), n.setAttribute("data-highlighted", u);
          } catch {
          }
      });
    }
    return U(
      () => [s.value, r.placeholderBlockId, r.placeholderHeight],
      () => ae(),
      { deep: !0, flush: "post" }
    ), X(() => {
      y.value && (L = oe(), L.observe(y.value, { childList: !0, subtree: !0 }), y.value.addEventListener("click", V, { capture: !0 }));
    }), ce(() => {
      var t;
      L == null || L.disconnect(), (t = y.value) == null || t.removeEventListener("click", V, { capture: !0 });
    }), o({
      containerRef: y
    }), (t, c) => (B(), I("div", {
      ref_key: "containerRef",
      ref: y,
      class: se(["streaming-document", { "is-sync": e.scrollSync, "is-dark": e.darkMode }]),
      "data-accent": i.value
    }, [
      (B(!0), I(H, null, Z(g.value, (n, m) => (B(), I(H, {
        key: n.kind + "-" + m
      }, [
        n.kind === "markdown" ? (B(), P($, {
          key: 0,
          content: n.text,
          final: !e.streaming,
          "max-live-nodes": e.streaming ? 0 : 320,
          "batch-rendering": e.streaming,
          "render-batch-size": 16,
          "render-batch-delay": 8,
          typewriter: e.streaming && m === k.value,
          fade: !1,
          "code-block-props": f
        }, null, 8, ["content", "final", "max-live-nodes", "batch-rendering", "typewriter"])) : T(n) ? (B(), P(z(() => T(n)(N(n), _(n))), { key: 1 })) : n.kind === "details" ? (B(), I("details", Fe, [
          J("summary", null, ue(n.summary), 1),
          J("div", je, [
            de($, {
              content: n.body,
              final: !e.streaming,
              "batch-rendering": e.streaming,
              "render-batch-size": 16,
              "render-batch-delay": 8,
              typewriter: e.streaming && m === k.value,
              fade: !1,
              "code-block-props": f
            }, null, 8, ["content", "final", "batch-rendering", "typewriter"])
          ])
        ])) : n.kind === "component" ? (B(), P(z(h[n.componentType]), fe({
          key: 3,
          ref_for: !0
        }, n.props, {
          final: n.final
        }), null, 16, ["final"])) : pe("", !0)
      ], 64))), 128))
    ], 10, ze));
  }
}), Ge = /* @__PURE__ */ Te(Ve, [["__scopeId", "data-v-c028e64e"]]);
export {
  Ge as S,
  $ as _,
  He as u
};
