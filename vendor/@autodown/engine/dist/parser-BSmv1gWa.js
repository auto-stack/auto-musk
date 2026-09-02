import { i as u, c as s, e as k, h as c, M as g, t as V, d as N, y as h, V as d } from "./markdown-parser-0FkmfLuR.js";
function L(e, t) {
  let r = "";
  for (let n = 0; n < t; n++)
    r = r + e;
  return r;
}
function T(e, t) {
  const r = Number(e.length), n = Number(t.length);
  return r < n ? !1 : e.slice(r - n, r) == t;
}
function $(e) {
  for (let t = 0; t < Number(e.length); t++)
    if (e.slice(t, t + 1) == `
`)
      return !0;
  return !1;
}
function A(e) {
  if (e.text == `
`)
    return `  
`;
  let t = e.text;
  const r = s(e.attrs, "wikilink", "");
  Number(r.length) > 0 && (t = "[[" + r + "]]");
  const n = s(e.attrs, "math_inline", "");
  if (Number(n.length) > 0 && (t = "$" + n + "$"), c(e.marks, g.Code) && (t = "`" + t + "`"), c(e.marks, g.Strong) && (t = "**" + t + "**"), c(e.marks, g.Em) && (t = "*" + t + "*"), c(e.marks, g.Underline) && (t = "__" + t + "__"), c(e.marks, g.Del) && (t = "~~" + t + "~~"), c(e.marks, g.Link)) {
    const l = s(e.attrs, "href", ""), i = s(e.attrs, "title", "");
    Number(i.length) > 0 ? t = "[" + t + "](" + l + ' "' + i + '")' : t = "[" + t + "](" + l + ")";
  }
  if (c(e.marks, g.Image)) {
    const l = s(e.attrs, "src", ""), i = s(e.attrs, "title", "");
    Number(i.length) > 0 ? t = "![" + t + "](" + l + ' "' + i + '")' : t = "![" + t + "](" + l + ")";
  }
  return t;
}
function b(e) {
  let t = "";
  for (const r of e)
    t = t + A(r);
  return t;
}
function C(e) {
  return e == "left" ? ":---" : e == "center" ? ":---:" : e == "right" ? "---:" : "---";
}
function M(e) {
  let t = "|";
  for (const r of e.children)
    t = t + " " + b(r.inlines) + " |";
  return t;
}
function F(e) {
  let t = "|";
  for (const r of e.children)
    t = t + " " + C(s(r.attrs, "align", "")) + " |";
  return t;
}
function G(e) {
  const t = e;
  return t._tag === "Null" ? [] : t._tag === "Str" ? (t.value, []) : t._tag === "Int" ? (t.value, []) : t._tag === "Bool" ? (t.value, []) : t._tag === "ListV" ? t.value : t._tag === "AttrsV" ? (t.value, []) : [];
}
function I(e) {
  const t = e;
  return t._tag === "Null" ? [] : t._tag === "Str" ? (t.value, []) : t._tag === "Int" ? (t.value, []) : t._tag === "Bool" ? (t.value, []) : t._tag === "ListV" ? (t.value, []) : t._tag === "AttrsV" ? t.value : [];
}
function D(e) {
  const t = e;
  return t._tag === "Null" ? null : t._tag === "Str" ? (t.value, null) : t._tag === "Int" ? t.value : t._tag === "Bool" || t._tag === "ListV" ? (t.value, null) : (t._tag === "AttrsV" && t.value, null);
}
function v(e) {
  const t = G(e);
  let r = [];
  for (let n = 0; n < Number(t.length); n++)
    r.push(D(t[n]));
  return r;
}
function y(e) {
  for (let t = 0; t < Number(e.length); t++)
    if (e[t] != null)
      return !0;
  return !1;
}
function B(e) {
  let t = "";
  for (let r = 0; r < Number(e.length); r++) {
    r > 0 && (t = t + ",");
    const n = e[r];
    n == null ? t = t + '"auto"' : t = t + String(n ?? 0);
  }
  return t;
}
function O(e) {
  const t = h(e, "cols"), r = h(e, "rows"), n = v(t ?? d.Null()), l = v(r ?? d.Null());
  let i = [];
  y(n) && i.push("cols:[" + B(n) + "]"), y(l) && i.push("rows:[" + B(l) + "]");
  let a = "";
  for (let o = 0; o < Number(i.length); o++)
    o > 0 && (a = a + ", "), a = a + i[o];
  return a;
}
function j(e) {
  if (Number(e.children.length) == 0)
    return "";
  let t = "";
  for (let n = 0; n < Number(e.children.length); n++) {
    n > 0 && (t = t + `
`);
    const l = e.children[n];
    n == 0 ? t = t + M(l) + `
` + F(l) : t = t + M(l);
  }
  const r = h(e.attrs, "ial");
  if (r != null) {
    const n = O(I(r ?? d.Null()));
    Number(n.length) > 0 && (t = t + `
{` + n + "}");
  }
  return t;
}
function _(e, t) {
  let r = "";
  for (let n = 0; n < Number(e.length); n++)
    n > 0 && (e[n].kind == u.ListBlock ? r = r + `
` : r = r + `

`), r = r + w(e[n], t);
  return r;
}
function z(e, t) {
  const n = _(e.children, t).split(`
`);
  let l = "";
  for (let i = 0; i < Number(n.length); i++)
    i > 0 && (l = l + `
`), Number(n[i].length) > 0 ? l = l + "> " + n[i] : l = l + ">";
  return l;
}
function E(e, t) {
  const r = N(e.attrs, "ordered", !1), n = V(e.attrs, "start", 1);
  let l = "";
  for (let i = 0; i < Number(e.children.length); i++) {
    i > 0 && (l = l + `
`);
    let a = "- ", o = 2;
    if (r) {
      const f = n + i;
      a = String(f) + ". ", o = Number(a.length);
    } else {
      const f = e.children[i];
      h(f.attrs, "checked") != null && (N(f.attrs, "checked", !1) ? a = "- [x] " : a = "- [ ] ");
    }
    const m = _(e.children[i].children, t).split(`
`), q = L(" ", o);
    for (let f = 0; f < Number(m.length); f++)
      f > 0 && (l = l + `
`), f == 0 ? l = l + a + m[f] : Number(m[f].length) > 0 && (l = l + q + m[f]);
  }
  return l;
}
function R(e) {
  const t = s(e.attrs, "language", ""), r = k(e.inlines);
  let n = "```" + t + `
` + r;
  return Number(r.length) > 0 && (T(r, `
`) || (n = n + `
`)), n = n + "```", n;
}
function W(e, t) {
  const r = V(e.attrs, "level", 1);
  let n = b(e.inlines);
  if (t && (n = x(n, s(e.attrs, "anchor", ""))), r <= 2 && $(n)) {
    let l = "---";
    return r == 1 && (l = "==="), n + `
` + l;
  }
  return L("#", r) + " " + n;
}
function p(e, t) {
  return e + ': "' + t + '"';
}
function S(e, t, r, n) {
  return "$" + e + "(" + t + `) {
` + _(r.children, n) + `
}`;
}
function H(e, t) {
  let r = p("type", s(e.attrs, "type", ""));
  const n = s(e.attrs, "title", "");
  return Number(n.length) > 0 && (r = r + ", " + p("title", n)), S("callout", r, e, t);
}
function Q(e, t) {
  let r = p("summary", s(e.attrs, "summary", ""));
  return N(e.attrs, "open", !1) && (r = r + ", open: true"), S("details", r, e, t);
}
function U(e) {
  const t = s(e.attrs, "target", ""), r = s(e.attrs, "anchor", "");
  return Number(r.length) > 0 ? "[[" + t + "#" + r + "]]" : "[[" + t + "]]";
}
function w(e, t) {
  const r = e.kind;
  if (r == u.Heading)
    return W(e, t);
  if (r == u.Fence)
    return R(e);
  if (r == u.Blockquote)
    return z(e, t);
  if (r == u.ListBlock)
    return E(e, t);
  if (r == u.ListItem)
    return _(e.children, t);
  if (r == u.Table)
    return j(e);
  if (r == u.TableRow)
    return M(e);
  if (r == u.TableCell)
    return b(e.inlines);
  if (r == u.ThematicBreak)
    return "---";
  if (r == u.Callout)
    return H(e, t);
  if (r == u.Details)
    return Q(e, t);
  if (r == u.WikilinkBlock)
    return U(e);
  if (r == u.QueryBlock)
    return "$query(" + s(e.attrs, "query", "") + ")";
  if (r == u.BlockEmbed)
    return '$embed(src: "' + s(e.attrs, "src", "") + '")';
  if (r == u.Mermaid)
    return "```mermaid\n" + k(e.inlines) + "\n```";
  if (r == u.MathBlock)
    return `%{
` + k(e.inlines) + `
}%`;
  const n = b(e.inlines);
  return t ? x(n, s(e.attrs, "anchor", "")) : n;
}
function x(e, t) {
  const r = "^" + t;
  return Number(t.length) == 0 || T(e, r) ? e : e + " " + r;
}
function J(e, t) {
  let r = "";
  for (let n = 0; n < Number(e.length); n++) {
    n > 0 && (r = r + `

`);
    const l = e[n];
    r = r + w(l, t);
  }
  return r;
}
function X(e, t) {
  const r = J(e.children, t);
  return Number(r.length) == 0 ? "" : r + `
`;
}
export {
  J as a,
  b as i,
  X as s
};
