import { computed as S, defineComponent as J, openBlock as g, createElementBlock as b, normalizeClass as F, createElementVNode as w, Fragment as C, renderList as L, toDisplayString as H, createCommentVNode as W, ref as U, watch as G, onMounted as Q, onBeforeUnmount as X, unref as D, createBlock as $, resolveDynamicComponent as Z, mergeProps as ee, nextTick as te } from "vue";
import { enableKatex as ne, enableMermaid as oe, MarkdownRender as ae } from "markstream-vue";
import { createLowlight as re, common as le } from "lowlight";
import { toHtml as ce } from "hast-util-to-html";
function se(t) {
  const i = t.trim();
  if (!i) return { value: null, valid: !1 };
  try {
    return { value: JSON.parse(i), valid: !0 };
  } catch {
  }
  let l = !1, o = !1;
  const r = [];
  for (let f = 0; f < i.length; f++) {
    const h = i[f];
    if (o) {
      o = !1;
      continue;
    }
    if (h === "\\") {
      o = !0;
      continue;
    }
    if (h === '"') {
      l = !l;
      continue;
    }
    if (!l) {
      if (h === "{" || h === "[")
        r.push(h === "{" ? "}" : "]");
      else if ((h === "}" || h === "]") && r.length > 0) {
        const k = r[r.length - 1];
        h === k && r.pop();
      }
    }
  }
  let d = "";
  l && (d += '"'), d += r.reverse().join("");
  try {
    return { value: JSON.parse(i + d), valid: !1 };
  } catch {
    return { value: null, valid: !1 };
  }
}
function ie(t) {
  const i = [];
  let l = 0;
  for (; l < t.length; ) {
    const o = t.indexOf("```json\n", l);
    if (o === -1) break;
    const r = o + 8, d = t.indexOf("\n```", r);
    if (d !== -1)
      i.push({
        start: o,
        end: d + 4,
        content: t.slice(r, d),
        closed: !0
      }), l = d + 4;
    else {
      i.push({
        start: o,
        end: t.length,
        content: t.slice(r),
        closed: !1
      });
      break;
    }
  }
  return i;
}
const j = /* @__PURE__ */ new Set(["table"]);
function ue(t) {
  return t && typeof t == "object" && typeof t.type == "string" && j.has(t.type);
}
function de(t) {
  const i = t.match(/"type"\s*:\s*"([^"]*)"/);
  if (!i) return null;
  const l = i[1];
  for (const o of j)
    if (o.startsWith(l) || l.startsWith(o)) return o;
  return null;
}
const N = /* @__PURE__ */ new Map();
function pe(t) {
  const i = ie(t), l = [];
  let o = 0;
  for (const r of i) {
    const d = `${r.start}`;
    r.start > o && l.push({ type: "markdown", text: t.slice(o, r.start) });
    const { value: f, valid: h } = se(r.content), k = de(r.content);
    if (ue(f)) {
      const { type: y, ...u } = f;
      N.set(d, u), l.push({
        type: "component",
        componentType: y,
        props: u,
        final: h && r.closed
      });
    } else if (k) {
      const y = N.get(d), E = f ?? y ?? (k === "table" ? { columns: [], rows: [] } : {});
      f && N.set(d, f), l.push({
        type: "component",
        componentType: k,
        props: E,
        final: h && r.closed
      });
    } else {
      const y = r.closed ? t.slice(r.start, r.end) : t.slice(r.start, r.end) + "\n```";
      l.push({ type: "markdown", text: y });
    }
    o = r.end;
  }
  return o < t.length && l.push({ type: "markdown", text: t.slice(o) }), l;
}
function fe(t) {
  return { segments: S(() => pe(t.value)) };
}
const me = {
  key: 0,
  class: "loading-row"
}, he = ["colspan"], ge = /* @__PURE__ */ J({
  __name: "StreamingTable",
  props: {
    columns: { default: () => [] },
    rows: { default: () => [] },
    final: { type: Boolean, default: !1 }
  },
  setup(t) {
    const i = t, l = S(() => i.columns ?? []), o = S(() => i.rows ?? []);
    return (r, d) => (g(), b("div", {
      class: F(["streaming-table", { final: t.final }])
    }, [
      w("table", null, [
        w("thead", null, [
          w("tr", null, [
            (g(!0), b(C, null, L(l.value, (f) => (g(), b("th", { key: f }, H(f), 1))), 128))
          ])
        ]),
        w("tbody", null, [
          (g(!0), b(C, null, L(o.value, (f, h) => (g(), b("tr", { key: h }, [
            (g(!0), b(C, null, L(l.value, (k) => (g(), b("td", { key: k }, H(f[k] ?? ""), 1))), 128))
          ]))), 128)),
          t.final ? W("", !0) : (g(), b("tr", me, [
            w("td", {
              colspan: Math.max(1, l.value.length)
            }, [...d[0] || (d[0] = [
              w("span", { class: "loading-dots" }, "Loading", -1)
            ])], 8, he)
          ]))
        ])
      ])
    ], 2));
  }
}), z = (t, i) => {
  const l = t.__vccOpts || t;
  for (const [o, r] of i)
    l[o] = r;
  return l;
}, be = /* @__PURE__ */ z(ge, [["__scopeId", "data-v-f2a1f208"]]), ke = '<span class="codeblock-copy-icon"></span>', ye = /* @__PURE__ */ J({
  __name: "StreamingRenderer",
  props: {
    source: {},
    streaming: { type: Boolean },
    placeholderBlockId: {},
    placeholderHeight: {}
  },
  setup(t, { expose: i }) {
    const l = re(le);
    ne(), oe();
    const o = t, r = S(() => f(o.source)), { segments: d } = fe(r);
    function f(n) {
      return n.replace(
        /:::details\s+(.*?)\n([\s\S]*?)\n:::/g,
        `<details>
<summary>$1</summary>
$2
</details>`
      );
    }
    const h = S(() => {
      for (let n = d.value.length - 1; n >= 0; n--)
        if (d.value[n].type === "markdown") return n;
      return -1;
    }), k = {
      showHeader: !0,
      showCopyButton: !0,
      showExpandButton: !0
    }, y = {
      table: be
      // Future: chart: StreamingChart, form: StreamingForm, ...
    }, u = U(null);
    function E(n) {
      n.querySelectorAll(".autodown-block-placeholder").forEach((m) => m.remove());
    }
    const _ = new MutationObserver(() => {
      u.value && (T(u.value), M(u.value), O(u.value), q(u.value));
    });
    function K(n, m) {
      const e = n.firstElementChild;
      if (!e) return null;
      const p = e.tagName.toLowerCase();
      return ["h1", "h2", "h3", "p", "pre", "blockquote", "ul", "ol", "hr", "img", "table"].includes(p) ? p : e.classList.contains("table-node-wrapper") ? "table" : e.classList.contains("image-error") || e.classList.contains("autodown-image-wrapper") || e.querySelector(".image-node-container, .image-node__img") ? "img" : e.classList.contains("autodown-callout") || e.classList.contains("admonition") ? "callout" : e.classList.contains("autodown-details") || e.classList.contains("html-block-node") ? "details" : e.classList.contains("autodown-math-block") || e.classList.contains("math-block") ? "math" : e.classList.contains("mermaid-block-container") ? "mermaid" : m && m !== "text" ? m : null;
    }
    function V(n) {
      return n === "blockquote" || n === "ul" || n === "ol" || n === "callout" || n === "admonition";
    }
    function T(n) {
      const m = Array.from(n.querySelectorAll(".node-slot")), e = [];
      m.forEach((c) => {
        const a = c.querySelector(".node-content");
        a && (a.removeAttribute("data-block-id"), a.removeAttribute("data-block-index"));
      });
      const p = n.getBoundingClientRect();
      if (m.forEach((c) => {
        const a = c.querySelector(".node-content");
        if (!a) return;
        const s = c.getAttribute("data-node-type"), v = K(a, s);
        if (!v) return;
        const P = c.getBoundingClientRect(), B = P.top - p.top, R = P.height;
        if (e.some((A) => V(A.type) ? B >= A.top && B < A.top + A.height : !1)) return;
        const x = e[e.length - 1];
        x && B === x.top && R === x.height || e.push({ slot: c, content: a, type: v, top: B, height: R });
      }), e.forEach(({ slot: c, content: a }, s) => {
        const v = `block-${s}`;
        a.setAttribute("data-block-id", v), a.setAttribute("data-block-index", String(s)), c.setAttribute("data-block-slot-id", v);
      }), o.placeholderBlockId != null && o.placeholderHeight != null) {
        const c = e[Number(o.placeholderBlockId.replace("block-", ""))];
        if (c && !c.slot.querySelector(":scope > .autodown-block-placeholder")) {
          const s = document.createElement("div");
          s.className = "autodown-block-placeholder", s.style.height = `${o.placeholderHeight}px`, c.slot.insertBefore(s, c.slot.firstChild);
        }
      }
    }
    async function Y() {
      u.value && (await te(), E(u.value), T(u.value), M(u.value), O(u.value), q(u.value));
    }
    function O(n) {
      Array.from(
        n.querySelectorAll("pre[data-language]:not([data-header-added])")
      ).forEach((e) => {
        const p = e.getAttribute("data-language") || "", c = document.createElement("div");
        c.className = "codeblock-language-badge", c.setAttribute("data-codeblock-language-badge", p);
        const a = document.createElement("span");
        a.className = "codeblock-language-label", a.textContent = p;
        const s = document.createElement("button");
        s.type = "button", s.className = "codeblock-copy-btn", s.setAttribute("data-codeblock-copy-btn", ""), s.setAttribute("title", "复制"), s.innerHTML = ke, c.appendChild(a), c.appendChild(s), e.appendChild(c), e.setAttribute("data-header-added", "");
      });
    }
    function q(n) {
      Array.from(
        n.querySelectorAll("details:not([data-details-wrapped])")
      ).forEach((e) => {
        const p = Array.from(e.children).filter((a) => {
          const s = a.tagName.toLowerCase();
          return s !== "summary" && s !== "details" && !a.classList.contains("details-content");
        });
        if (p.length === 0) return;
        const c = document.createElement("div");
        c.className = "details-content", p.forEach((a) => c.appendChild(a)), e.appendChild(c), e.setAttribute("data-details-wrapped", "");
      });
    }
    function I(n) {
      var a, s;
      const m = n.target, e = (a = m.closest) == null ? void 0 : a.call(m, "[data-codeblock-copy-btn]");
      if (!e || !u.value) return;
      const p = e.closest("pre"), c = ((s = p == null ? void 0 : p.querySelector("code")) == null ? void 0 : s.textContent) ?? "";
      n.preventDefault(), n.stopPropagation(), navigator.clipboard.writeText(c);
    }
    function M(n) {
      Array.from(n.querySelectorAll("pre[data-language] > code")).forEach((e) => {
        const c = e.parentElement.getAttribute("data-language"), a = c === "plaintext" ? "text" : c;
        if (!a || a === "text" || e.getAttribute("data-highlighted") === a || !l.registered(a)) return;
        const s = e.textContent || "";
        if (s)
          try {
            const v = l.highlight(a, s);
            e.innerHTML = ce(v), e.setAttribute("data-highlighted", a);
          } catch {
          }
      });
    }
    return G(
      () => [d.value, o.placeholderBlockId, o.placeholderHeight],
      () => Y(),
      { deep: !0, flush: "post" }
    ), Q(() => {
      u.value && (_.observe(u.value, { childList: !0, subtree: !0 }), u.value.addEventListener("click", I, { capture: !0 }));
    }), X(() => {
      var n;
      _.disconnect(), (n = u.value) == null || n.removeEventListener("click", I, { capture: !0 });
    }), i({
      containerRef: u
    }), (n, m) => (g(), b("div", {
      ref_key: "containerRef",
      ref: u,
      class: "streaming-document"
    }, [
      (g(!0), b(C, null, L(D(d), (e, p) => (g(), b(C, {
        key: e.type + "-" + p
      }, [
        e.type === "markdown" ? (g(), $(D(ae), {
          key: 0,
          content: e.text,
          final: !t.streaming,
          "max-live-nodes": t.streaming ? 0 : 320,
          "batch-rendering": t.streaming,
          "render-batch-size": 16,
          "render-batch-delay": 8,
          typewriter: t.streaming && p === h.value,
          fade: !1,
          "code-block-props": k
        }, null, 8, ["content", "final", "max-live-nodes", "batch-rendering", "typewriter"])) : e.type === "component" ? (g(), $(Z(y[e.componentType]), ee({
          key: 1,
          ref_for: !0
        }, e.props, {
          final: e.final
        }), null, 16, ["final"])) : W("", !0)
      ], 64))), 128))
    ], 512));
  }
}), Le = /* @__PURE__ */ z(ye, [["__scopeId", "data-v-ef10c443"]]);
export {
  Le as StreamingRenderer,
  be as StreamingTable,
  fe as useStreamingDocument
};
