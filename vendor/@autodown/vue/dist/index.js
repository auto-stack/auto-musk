import { h as m, ref as V, watch as ve, computed as L, onScopeDispose as Ae, defineComponent as ae, onMounted as we, openBlock as T, createElementBlock as N, Fragment as j, renderList as H, createBlock as ie, resolveDynamicComponent as xe, normalizeClass as Ie, createElementVNode as P, toDisplayString as he, createCommentVNode as Ce, onBeforeUnmount as We, unref as Oe, mergeProps as $e, nextTick as Me } from "vue";
import { createLowlight as Pe, common as qe } from "lowlight";
import { toHtml as De } from "hast-util-to-html";
function je(e) {
  return e.replace(RegExp(`\r
`, "g"), `
`).replace(RegExp("\r", "g"), `
`);
}
function ze(e, t) {
  if (t)
    return e;
  let n = e, r = !0;
  for (; r; ) {
    r = !1;
    const i = n.replace(RegExp(`
 {0,3}([-*+]|\\d{1,9}[.)]) *$`, ""), "");
    i != n && (n = i, r = !0);
    const s = n.replace(RegExp(`
 {0,3}> *$`, ""), "");
    s != n && (n = s, r = !0);
  }
  return n;
}
function W(e) {
  return e.trim() == "";
}
function z(e) {
  let t = 0;
  for (; t < e.length && e[t] == " "; )
    t += 1;
  return t;
}
function I(e) {
  let t = 0, n = 0;
  for (; n < e.length; ) {
    const r = e[n];
    if (r == " ")
      t += 1;
    else if (r == "	")
      t += 4;
    else
      break;
    n += 1;
  }
  return t;
}
function He(e, t) {
  const n = je(e), i = ze(n, t).split(`
`);
  return ce(i, t);
}
function Se(e) {
  if (I(e) >= 4)
    return "";
  const t = e.trim();
  return t.startsWith("```") ? "`" : t.startsWith("~~~") ? "~" : "";
}
function Ue(e) {
  const t = e.trim();
  let n = "", r = 0;
  const i = t[0];
  if (i != "`" && i != "~")
    return "";
  for (; r < t.length; ) {
    const s = t[r];
    if (s == i)
      n = n + s, r += 1;
    else
      break;
  }
  return n;
}
function Ye(e, t, n) {
  if (I(e) >= 4)
    return !1;
  const r = e.trim();
  if (r.length != n.length)
    return !1;
  let i = 0;
  for (; i < r.length; ) {
    if (r[i] != t)
      return !1;
    i += 1;
  }
  return !0;
}
function Ee(e) {
  if (I(e) >= 4)
    return 0;
  const t = e.trim();
  let n = 0;
  for (; n < t.length && t[n] == "#"; )
    n += 1;
  return n == 0 || n > 6 ? 0 : n == t.length ? n : t[n] != " " ? 0 : n;
}
function Fe(e) {
  const t = e.trim(), n = t.match(RegExp("^(.*?)( #{1,})#*$"));
  return n == null ? t : n[1].trimEnd();
}
function Z(e) {
  if (I(e) >= 4)
    return -1;
  const t = e.match(RegExp("^ {0,3}(\\d{1,9})[.)] |^ {0,3}(\\d{1,9})[.)]$"));
  return t == null ? -1 : t[1] != null ? parseInt(t[1], 10) : parseInt(t[2], 10);
}
function K(e) {
  if (I(e) >= 4)
    return "";
  const t = e.match(RegExp("^ {0,3}([-*+]) "));
  return t == null ? "" : t[1];
}
function G(e) {
  return !(I(e) >= 4 || e.match(RegExp("^ {0,3}[-*+]$")) == null);
}
function _e(e) {
  if (I(e) >= 4)
    return !1;
  const t = e.trim();
  if (t.length < 3)
    return !1;
  const n = t[0];
  if (n != "-" && n != "*" && n != "_")
    return !1;
  let r = 0, i = 0;
  for (; i < t.length; ) {
    const s = t[i];
    if (s == n)
      r += 1;
    else if (s != " ")
      return !1;
    i += 1;
  }
  return r >= 3;
}
function Qe(e) {
  if (I(e) >= 4)
    return 0;
  const t = e.trim();
  if (t.length == 0)
    return 0;
  const n = t[0];
  if (n != "=" && n != "-")
    return 0;
  let r = 0;
  for (; r < t.length; ) {
    if (t[r] != n)
      return 0;
    r += 1;
  }
  return n == "=" ? 1 : 2;
}
function le(e) {
  return I(e) >= 4 ? !1 : e.trimStart().startsWith(">");
}
function Je(e) {
  const n = e.trimStart().slice(1);
  return n.startsWith(" ") ? n.slice(1) : n;
}
function se(e) {
  return W(e) ? !1 : e.includes("|");
}
function Ve(e) {
  if (W(e) || !e.includes("-"))
    return !1;
  const t = M(e);
  if (t.length == 0)
    return !1;
  for (const n of t)
    if (n.trim().match(RegExp("^:?-+:?$")) == null)
      return !1;
  return !0;
}
function M(e) {
  let t = e.trim();
  return t.startsWith("|") && (t = t.slice(1)), t.endsWith("|") && (t = t.slice(0, t.length - 1)), t.split("|");
}
function Ze(e) {
  const t = e.trim(), n = t.startsWith(":"), r = t.endsWith(":");
  return n ? r ? "center" : "left" : r ? "right" : "left";
}
function ce(e, t) {
  let n = [], r = 0;
  for (; r < e.length; ) {
    const i = e[r];
    if (W(i)) {
      r += 1;
      continue;
    }
    const s = Se(i);
    if (s != "") {
      const a = Ue(i), b = i.trim().slice(a.length).trim();
      let k = [], E = r + 1, R = !1;
      for (; E < e.length; ) {
        if (Ye(e[E], s, a)) {
          R = !0;
          break;
        }
        k.push(e[E]), E += 1;
      }
      let B = k.join(`
`);
      if (R)
        k.length > 0 ? B = B + `
` : B = "", n.push({ type: "code_block", language: b, code: B, loading: !1 });
      else {
        for (; k.length > 0; ) {
          const C = k[k.length - 1].trim();
          if (C == "" || C.match(RegExp("^[`~]+$")) == null)
            break;
          k.pop();
        }
        let v = k.join(`
`);
        v = v.replace(RegExp(`
 +$`), `
`), n.push({ type: "code_block", language: b, code: v, loading: !t });
      }
      r = E + 1;
      continue;
    }
    const o = Ee(i);
    if (o > 0) {
      const p = i.trim().slice(o).trim(), b = Fe(p);
      let k = q(b, t);
      b == "" && (k = []), n.push({ type: "heading", level: o, children: k }), r += 1;
      continue;
    }
    if (_e(i)) {
      n.push({ type: "thematic_break" }), r += 1;
      continue;
    }
    if (le(i)) {
      let a = [], p = r;
      for (; p < e.length; )
        if (le(e[p]))
          a.push(Je(e[p])), p += 1;
        else {
          if (W(e[p]))
            break;
          if (Ke(e[p]))
            a.push(e[p]), p += 1;
          else
            break;
        }
      const b = ce(a, t);
      n.push({ type: "blockquote", children: b }), r = p;
      continue;
    }
    if (se(i) && r + 1 < e.length && Ve(e[r + 1])) {
      const a = M(i).length, p = M(e[r + 1]).length;
      if (a == p) {
        r = Ge(e, r, n, t);
        continue;
      }
    }
    const l = Z(i), f = K(i);
    if (l >= 0) {
      r = ne(e, r, !0, l, n, t);
      continue;
    }
    if (f != "") {
      r = ne(e, r, !1, 0, n, t);
      continue;
    }
    if (G(i)) {
      r = ne(e, r, !1, 0, n, t);
      continue;
    }
    let u = [], c = r, h = 0;
    for (; c < e.length; ) {
      const a = e[c];
      if (W(a))
        break;
      if (u.length > 0) {
        const p = Qe(a);
        if (p > 0) {
          h = p, c += 1;
          break;
        }
      }
      if (X(a, e, c))
        break;
      u.push(a.replace(RegExp("^ {0,4}"), "")), c += 1;
    }
    if (h > 0) {
      const a = u.join(`
`), p = q(a, t);
      n.push({ type: "heading", level: h, children: p }), r = c;
      continue;
    }
    if (u.length > 0) {
      if (!t) {
        let b = !1;
        if (u.length >= 2 && (b = !0), c < e.length && (b = !0), b) {
          const k = u[0];
          if (se(k) && k.trim().endsWith("|")) {
            const E = M(k);
            if (E.length >= 2) {
              let R = !0, B = 0;
              for (; B < u.length; )
                u[B].trim().startsWith("|") || (R = !1), B += 1;
              if (R) {
                let v = [];
                for (const g of E) {
                  const x = g.trim(), d = q(x, t);
                  v.push({ type: "table_cell", header: !0, children: d, align: "left" });
                }
                const C = { type: "table_row", cells: v };
                n.push({ type: "table", header: C, rows: [], loading: !0 }), r = c;
                continue;
              }
            }
          }
        }
      }
      const a = u.join(`
`), p = q(a, t);
      n.push({ type: "paragraph", children: p }), r = c;
      continue;
    }
    r += 1;
  }
  return n;
}
function X(e, t, n) {
  return n == 0 ? !1 : !!(Se(e) != "" || Ee(e) > 0 || _e(e) || le(e) || K(e) != "" || G(e) || Z(e) >= 0);
}
function Ke(e) {
  return !X(e, [], 0);
}
function ne(e, t, n, r, i, s) {
  let o = [], l = t, f = null;
  for (n && r != 1 && (f = r); l < e.length; ) {
    const u = e[l];
    if (W(u)) {
      let v = l + 1;
      for (; v < e.length && W(e[v]); )
        v += 1;
      if (v < e.length) {
        if (K(e[v]) != "") {
          l = v;
          continue;
        }
        if (G(e[v])) {
          l = v;
          continue;
        }
        if (Z(e[v]) >= 0) {
          l = v;
          continue;
        }
      }
      break;
    }
    let c = 0, h = 0, a = !1, p = !1;
    const b = K(u), k = Z(u);
    if (b != "")
      c = z(u), u.trimStart(), h = 2 + z(u), a = !0, p = !1;
    else if (G(u))
      c = z(u), h = 1 + z(u), a = !0, p = !1;
    else if (k >= 0) {
      c = z(u);
      const C = u.trimStart().match(RegExp("^(\\d{1,9}[.)])( *)"));
      if (C != null) {
        const g = C[1].length;
        let x = C[2].length;
        x > 4 && (x = 1), x < 1 && (x = 1), h = c + g + x, a = !0, p = !0;
      }
    }
    if (!a || p != n)
      break;
    if (!n) {
      const C = u.trimStart()[0], x = e[t].trimStart()[0];
      if (C != x)
        break;
    }
    let E = [], R = u.slice(h);
    for (E.push(R), l += 1; l < e.length; ) {
      const v = e[l];
      if (W(v)) {
        let C = l + 1;
        for (; C < e.length && W(e[C]); )
          C += 1;
        if (C < e.length && I(e[C]) >= h) {
          X(e[C], e, C);
          let g = l;
          for (; g < C; )
            E.push(""), g += 1;
          l = C;
          continue;
        }
        break;
      }
      if (I(v) >= h) {
        let C = v.slice(h);
        E.push(C), l += 1;
        continue;
      }
      if (!X(v, e, l)) {
        E.push(v), l += 1;
        continue;
      }
      break;
    }
    const B = ce(E, s);
    o.push({ type: "list_item", children: B });
  }
  return n ? f != null ? i.push({ type: "list", ordered: !0, start: f, items: o }) : i.push({ type: "list", ordered: !0, items: o }) : i.push({ type: "list", ordered: !1, items: o }), l;
}
function Ge(e, t, n, r) {
  const i = M(e[t]), s = [], o = M(e[t + 1]);
  for (const a of o)
    s.push(Ze(a));
  let l = [], f = t + 2;
  for (; f < e.length && se(e[f]); ) {
    const a = M(e[f]);
    let p = [], b = 0;
    for (; b < i.length; ) {
      let k = "";
      b < a.length && (k = a[b].trim());
      let E = "left";
      b < s.length && (E = s[b]);
      let R = q(k, r);
      k == "" && (R = []), p.push({ type: "table_cell", header: !1, children: R, align: E }), b += 1;
    }
    l.push({ type: "table_row", cells: p }), f += 1;
  }
  let u = [], c = 0;
  for (; c < i.length; ) {
    const a = i[c].trim();
    let p = "left";
    c < s.length && (p = s[c]);
    const b = q(a, r);
    u.push({ type: "table_cell", header: !0, children: b, align: p }), c += 1;
  }
  const h = { type: "table_row", cells: u };
  return n.push({ type: "table", header: h, rows: l, loading: !1 }), f;
}
function q(e, t) {
  return e == "" ? [] : $(e, t);
}
function $(e, t) {
  let n = [], r = "", i = 0, s = !1;
  for (; i < e.length; ) {
    const o = e[i];
    if (o == `
`) {
      let l = 0;
      for (; l < r.length && r[r.length - 1 - l] == " "; )
        l += 1;
      if (l >= 2) {
        const f = r.slice(0, r.length - l);
        f != "" && n.push(A(f)), n.push({ type: "hardbreak" }), r = "";
      } else
        l > 0 && (r = r.slice(0, r.length - l)), r = r + `
`;
      i += 1;
      continue;
    }
    if (o == "*") {
      if (e.startsWith("**", i)) {
        let f = Q(e, i, "**", !0, t);
        if (f != null) {
          r != "" && (n.push(A(r)), r = ""), n.push({ type: "strong", children: $(f[1], t) }), i = f[0];
          continue;
        }
      }
      let l = Q(e, i, "*", !1, t);
      if (l != null) {
        r != "" && (n.push(A(r)), r = ""), n.push({ type: "emphasis", children: $(l[1], t) }), i = l[0];
        continue;
      }
      r = r + o, i += 1;
      continue;
    }
    if (o == "_") {
      let l = Q(e, i, "_", !1, t);
      if (l != null) {
        r != "" && (n.push(A(r)), r = ""), n.push({ type: "emphasis", children: $(l[1], t) }), i = l[0];
        continue;
      }
      r = r + o, i += 1;
      continue;
    }
    if (o == "~") {
      if (e.startsWith("~~", i)) {
        let l = Q(e, i, "~~", !1, t);
        if (l != null) {
          r != "" && (n.push(A(r)), r = ""), n.push({ type: "strikethrough", children: $(l[1], t) }), i = l[0];
          continue;
        }
      }
      r = r + o, i += 1;
      continue;
    }
    if (o == "`") {
      let l = 0;
      for (; e.startsWith("`", i + l); )
        l += 1;
      let f = lt(e, i + l, l);
      if (f != -1) {
        let u = e.slice(i + l, f);
        const h = u.replace(RegExp("^ "), "").replace(RegExp(" $"), "");
        u.startsWith(" ") && u.endsWith(" ") && u.trim() != "" && (u = h), r != "" && (n.push(A(r)), r = ""), n.push({ type: "inline_code", code: u }), s = !0, i = f + l;
        continue;
      }
      if (!t && l == 1 && e.slice(i + l).trim() == "" && r == "") {
        i = e.length;
        continue;
      }
      if (l == 1 && !t) {
        const u = e.slice(i + 1);
        r != "" && (n.push(A(r)), r = ""), n.push({ type: "inline_code", code: u }), s = !0, i = e.length;
        continue;
      }
      r = r + e.slice(i, i + l), i = i + l;
      continue;
    }
    if (o == "!") {
      if (e.startsWith("![", i)) {
        let l = ge(e, i + 1, t, s);
        if (l != null) {
          r != "" && (n.push(A(r)), r = ""), n.push({ type: "image", src: l[2], alt: l[1], title: null, loading: !1 }), i = l[0];
          continue;
        }
      }
      r = r + o, i += 1;
      continue;
    }
    if (o == "[") {
      let l = ge(e, i, t, s);
      if (l != null) {
        r != "" && (n.push(A(r)), r = ""), l[3] ? (n.push({ type: "link", href: l[2], title: l[4], text: l[1], children: $(l[1], t), loading: !0 }), l[5] != "" && n.push(A(l[5]))) : n.push({ type: "link", loading: !1, href: l[2], title: l[4], text: l[1], children: $(l[1], t) }), i = l[0];
        continue;
      }
      r = r + o, i += 1;
      continue;
    }
    if (o == "\\" && i + 1 < e.length) {
      const l = e[i + 1];
      if (Re(l)) {
        l == '"' ? r = r + "" : l == "'" ? r = r + "" : r = r + l, i += 2;
        continue;
      }
    }
    r = r + o, i += 1;
  }
  return r != "" && n.push(A(r)), t || Xe(n), n;
}
function Xe(e) {
  if (e.length > 0) {
    const t = e[e.length - 1];
    t.type == "text" && et(t, e);
  }
}
function et(e, t) {
  let n = e.content, r = !1, i = n.replace(RegExp(" ?<[/!a-zA-Z][^>]*$"), "");
  i == n && (i = n.replace(RegExp("<$"), "")), i != n && (r = !0), n = i;
  let s = n.replace(RegExp("\\(+\\s*$"), "");
  s != n && (r = !0), n = s;
  let o = n.replace(RegExp("\\* +$"), "");
  o == n && n.endsWith("*") && (n.endsWith("**") || (o = n.slice(0, n.length - 1))), o != n && (r = !0), n = o, r || (n = n.replace(RegExp(" +$"), "")), n.trim() == "|" && (n = ""), n == "" ? t.pop() : e.content = n;
}
function A(e) {
  let t = it(e);
  return t = t.split("").join('"'), t = t.split("").join("'"), { type: "text", content: t };
}
function oe(e) {
  return RegExp("\\w", "u").test(e);
}
function tt(e) {
  return RegExp(`[\\)\\]},.;:!?\\u2026"'\\uff09\\uff0c\\uff0e\\u3002\\uff1b\\uff1a\\uff01\\uff1f\\u300d\\u300f\\u3009\\u300b]`).test(e);
}
let nt = "“", F = "”", rt = "‘", pe = "’";
function it(e) {
  let t = "", n = 0;
  for (; n < e.length; ) {
    const r = e[n];
    if (r == '"') {
      let i = !1;
      if (t == "")
        i = !0;
      else {
        const o = t.charAt(t.length - 1);
        o == " " && (i = !0), o == `
` && (i = !0), o == "(" && (i = !0), o == "[" && (i = !0), o == "{" && (i = !0);
      }
      if (i) {
        t = t + nt, n += 1;
        continue;
      }
      if (n + 1 >= e.length) {
        t = t + F, n += 1;
        continue;
      }
      const s = e[n + 1];
      s == " " || s == `
` || tt(s) ? t = t + F : t = t + r, n += 1;
      continue;
    }
    if (r == "'") {
      let i = !1;
      if (n > 0 && n + 1 < e.length) {
        const s = e[n - 1], o = e[n + 1], l = oe(s), f = oe(o);
        l && f && (i = !0);
      }
      if (i)
        t = t + pe;
      else {
        let s = !1;
        if (t == "")
          s = !0;
        else {
          const o = t.charAt(t.length - 1);
          o == " " && (s = !0), o == `
` && (s = !0), o == "(" && (s = !0), o == "[" && (s = !0);
        }
        s ? t = t + rt : t = t + pe;
      }
      n += 1;
      continue;
    }
    t = t + r, n += 1;
  }
  return t;
}
function Re(e) {
  return RegExp("\\p{P}|[-+/=@$^`|~]", "u").test(e);
}
function Q(e, t, n, r, i) {
  if (n == "_" && t > 0) {
    const c = e[t - 1];
    if (oe(c))
      return null;
  }
  const s = t + n.length, o = e.slice(s);
  let l = o.indexOf(n), f = o;
  if (l != -1 && (f = o.slice(0, l)), f == "")
    return null;
  const u = f[0];
  return Re(u) ? null : l != -1 ? [s + l + n.length, f] : !r && i || o == "" || o.startsWith(" ") ? null : [e.length, o];
}
function lt(e, t, n) {
  let r = t;
  for (; r < e.length; ) {
    if (e[r] != "`") {
      r += 1;
      continue;
    }
    let i = 0;
    for (; e.startsWith("`", r + i); )
      i += 1;
    if (i == n)
      return r;
    r = r + i;
  }
  return -1;
}
function ge(e, t, n, r) {
  let i = e.indexOf("]", t);
  if (i == -1)
    return null;
  const s = e.slice(t + 1, i);
  let o = i + 1;
  if (!e.startsWith("(", o))
    return null;
  let l = e.indexOf(")", o);
  if (l == -1) {
    const a = e.slice(o + 1);
    if (a.match(RegExp("^https?:\\/\\/.+")) != null) {
      let k = a.replace(RegExp("[.,:;!?)]+$"), ""), E = a.slice(k.length), R = "";
      return r && (R = null), [e.length, s, k, !0, R, E];
    }
    return a.match(RegExp("^[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$")) != null ? [e.length, s, "http://" + a, !0, null, ""] : [e.length, s, "", !0, null, ""];
  }
  let f = e.slice(o + 1, l), u = f, c = null;
  const h = f.indexOf(' "');
  if (h != -1) {
    u = f.slice(0, h);
    const a = f.slice(h + 2);
    a.endsWith('"') && (c = a.slice(0, a.length - 1));
  } else
    l + 1 >= e.length || r || (c = "");
  return [l + 1, s, u, !1, c, ""];
}
function me(e, t, n) {
  const r = n !== void 0 && Number.isFinite(n) ? { remaining: n } : void 0;
  return (e ?? []).map((i, s) => {
    const o = s === e.length - 1;
    return Te(i, s, t, o ? r : void 0);
  });
}
function Te(e, t, n, r) {
  return m("div", { class: "node-slot", "data-node-index": String(t), "data-node-type": e.type }, [
    m("div", { class: "node-content" }, [ct(e, n, r)])
  ]);
}
function J(e, t, n) {
  const r = (e ?? []).map((i, s) => {
    const o = s === ((e == null ? void 0 : e.length) ?? 0) - 1;
    return Te(i, s, t, o ? n : void 0);
  });
  return m("div", { class: "markstream-vue markdown-renderer" }, r);
}
function D(e, t, n) {
  return (e ?? []).map((r) => at(r, t, n));
}
function st(e) {
  return e.content ?? e.code ?? "";
}
function ot(e, t) {
  if (!t) return e;
  if (t.remaining <= 0) return "";
  const n = t.remaining >= e.length ? e : e.slice(0, t.remaining);
  return t.remaining -= n.length, n;
}
function at(e, t, n) {
  switch (e.type) {
    case "text":
      return m("span", { class: "whitespace-pre-wrap break-words text-node" }, [m("span", ot(e.content, n))]);
    case "strong":
      return m("strong", { class: "strong-node" }, D(e.children, t, n));
    case "emphasis":
      return m("em", { class: "emphasis-node" }, D(e.children, t, n));
    case "strikethrough":
      return m("del", { class: "strikethrough-node" }, D(e.children, t, n));
    case "inline_code":
      return m("code", { class: "inline-code" }, [m("span", e.code)]);
    case "link":
      return m(
        "a",
        {
          class: "link-node",
          href: e.href,
          title: e.title ?? void 0,
          target: "_blank",
          rel: "noopener noreferrer"
        },
        D(e.children, t, n)
      );
    case "image":
      return m("span", { class: "image-node-container" }, [
        m("img", {
          src: e.src,
          alt: e.alt,
          title: e.alt,
          class: "image-node__img",
          loading: "lazy"
        })
      ]);
    case "hardbreak":
      return m("br");
    default:
      return m("span", { class: "whitespace-pre-wrap break-words text-node" }, [
        m("span", st(e))
      ]);
  }
}
function be(e) {
  return e.align === "center" ? "text-center" : e.align === "right" ? "text-right" : "text-left";
}
function ct(e, t, n) {
  var r;
  switch (e.type) {
    case "heading": {
      const i = Math.min(6, Math.max(1, e.level));
      return m(`h${i}`, { class: `heading-node heading-${i}`, dir: "auto" }, [
        ...D(e.children, t, n)
      ]);
    }
    case "paragraph":
      return m("p", { class: "paragraph-node", dir: "auto" }, D(e.children, t, n));
    case "text":
      return m("span", { class: "whitespace-pre-wrap break-words text-node" }, [m("span", e.content)]);
    case "thematic_break":
      return m("hr", { class: "hr-node" });
    case "code_block":
      return ut(e);
    case "blockquote":
      return m("blockquote", { class: "blockquote", dir: "auto" }, [
        J(e.children, t, n)
      ]);
    case "list": {
      const i = e.ordered ? "ol" : "ul";
      return m(
        i,
        { class: e.ordered ? "list-node list-decimal" : "list-node list-disc" },
        (e.items ?? []).map(
          (s) => m("li", { class: "list-item", dir: "auto" }, [J(s.children ?? [], t, n)])
        )
      );
    }
    case "table":
      return m("table", { class: "table-node", "aria-busy": "false" }, [
        m("thead", {}, [
          m(
            "tr",
            {},
            (((r = e.header) == null ? void 0 : r.cells) ?? []).map(
              (i) => m("th", { dir: "auto", class: be(i) }, [
                J(i.children ?? [], t, n),
                m("button", { type: "button", class: "table-node__resize-handle" })
              ])
            )
          )
        ]),
        m(
          "tbody",
          {},
          (e.rows ?? []).map(
            (i) => m(
              "tr",
              {},
              (i.cells ?? []).map(
                (s) => m("td", { dir: "auto", class: be(s) }, [
                  J(s.children ?? [], t, n)
                ])
              )
            )
          )
        )
      ]);
    default:
      return m("div", { class: "unknown-node" }, String(e.type));
  }
}
function ut(e) {
  const t = e.language ? String(e.language) : "";
  return m("div", { class: "code-block-container rounded-lg border" }, [
    m("div", { class: "code-block-header flex justify-between items-center" }, [
      m("div", { class: "code-header-main" }, [
        m("div", { class: "code-header-copy" }, [
          m("div", { class: "code-header-title" }, t)
        ])
      ]),
      m("div", { class: "flex items-center gap-0.5" })
    ]),
    m(
      "pre",
      {
        class: `language-${t || "text"} code-pre-fallback is-wrap`,
        "data-language": t,
        "aria-busy": "false",
        tabindex: "0"
      },
      [m("code", { translate: "no" }, e.code)]
    )
  ]);
}
function ft(e, t, n) {
  let r = t - e;
  return r <= 0 ? e : n <= 0 ? t : r > n ? e + n : t;
}
function dt(e, t) {
  if (t <= 0)
    return 0;
  let n = e - t;
  return n < 0 ? 0 : n;
}
function ht(e, t, n) {
  if (t - e <= 0)
    return t;
  let i = e + n;
  return i > t ? t : i;
}
const pt = {
  setTimeout: (e, t) => setTimeout(e, t),
  clearTimeout: (e) => clearTimeout(e)
};
function gt(e, t) {
  const n = t.timer ?? pt, r = V(e.value.length), i = V(Number.POSITIVE_INFINITY);
  let s;
  function o(h) {
    s !== void 0 && n.clearTimeout(s), s = n.setTimeout(() => {
      s = void 0, h();
    }, t.batchDelay);
  }
  function l(h) {
    return h ? h.type === "text" ? String(h.content ?? "") : (h.children ?? []).map((p) => l(p)).join("") : "";
  }
  function f() {
    const h = e.value[e.value.length - 1], a = l(h).length;
    if (a <= 0) {
      i.value = Number.POSITIVE_INFINITY;
      return;
    }
    i.value = 0;
    const p = () => {
      const b = ht(i.value, a, t.typewriterChunk);
      i.value = b, b < a && o(p);
    };
    o(p);
  }
  ve(
    e,
    (h) => {
      if (s !== void 0 && (n.clearTimeout(s), s = void 0), !t.enabled) {
        r.value = h.length, i.value = Number.POSITIVE_INFINITY;
        return;
      }
      const a = h.length, p = Math.min(a, Math.max(1, Math.floor(t.batchSize / 4) || 1));
      r.value = Math.max(r.value, p);
      const b = () => {
        const k = ft(r.value, a, t.batchSize);
        r.value = k, k < a ? o(b) : t.typewriter && f();
      };
      r.value < a ? o(b) : t.typewriter && f();
    },
    { immediate: !0 }
  );
  const u = L(() => dt(r.value, t.maxLiveNodes)), c = L(() => e.value.slice(u.value, r.value));
  return Ae(() => {
    s !== void 0 && n.clearTimeout(s);
  }), { visibleNodes: c, visibleCount: r, typewriterChars: i, windowStart: u };
}
const mt = { class: "markstream-vue markdown-renderer" }, bt = /* @__PURE__ */ ae({
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
    const t = e, n = L(() => He(t.content ?? "", t.final)), r = L(() => me(n.value, t.final)), i = gt(n, {
      enabled: t.batchRendering,
      batchSize: t.renderBatchSize,
      batchDelay: t.renderBatchDelay,
      maxLiveNodes: t.maxLiveNodes,
      typewriter: t.typewriter && !t.final,
      typewriterChunk: 2
    }), s = typeof window > "u", o = V(!1), l = L(
      () => me(i.visibleNodes.value, t.final, i.typewriterChars.value)
    ), f = L(() => s || !o.value ? r.value : l.value);
    return we(() => {
      o.value = !0;
    }), (u, c) => (T(), N("div", mt, [
      (T(!0), N(j, null, H(f.value, (h, a) => (T(), ie(xe(h), { key: a }))), 128))
    ]));
  }
}), ee = {};
function ue(e, t, n) {
  ee[e] = { enabled: t, factory: n };
}
function kt(e) {
  ue("katex", !0, e);
}
function yt(e) {
  ue("mermaid", !0, e);
}
function Pt(e) {
  ue("highlight", !0, e);
}
function qt(e) {
  var t;
  return ((t = ee[e]) == null ? void 0 : t.enabled) === !0;
}
function Dt() {
  for (const e of Object.keys(ee))
    delete ee[e];
}
function ke(e) {
  try {
    return { ok: !0, value: JSON.parse(e) };
  } catch {
    return { ok: !1, value: null };
  }
}
function ye(e) {
  return typeof e;
}
function Be(e) {
  return !!e;
}
let Ne = ["table"], re = {};
function vt(e) {
  const t = e.trim();
  if (t == "")
    return { value: null, valid: !1 };
  const n = ke(t);
  if (n.ok)
    return { value: n.value, valid: !0 };
  let r = !1, i = !1, s = [], o = 0;
  for (; o < t.length; ) {
    const u = t[o];
    if (i) {
      i = !1, o += 1;
      continue;
    }
    if (u == "\\") {
      i = !0, o += 1;
      continue;
    }
    if (u == '"') {
      r = !r, o += 1;
      continue;
    }
    if (r) {
      o += 1;
      continue;
    }
    if (u == "{" || u == "[") {
      u == "{" ? s.push("}") : s.push("]"), o += 1;
      continue;
    }
    let c = !1;
    if (u == "}" && (c = !0), u == "]" && (c = !0), c && s.length > 0) {
      const h = s[s.length - 1];
      u == h && s.pop();
    }
    o += 1;
  }
  let l = "";
  r && (l = l + '"'), l = l + s.reverse().join("");
  const f = ke(t + l);
  return f.ok ? { value: f.value, valid: !1 } : { value: null, valid: !1 };
}
function wt(e) {
  let t = [], n = 0;
  for (; n < e.length; ) {
    const r = e.indexOf("```json\n", n);
    if (r == -1)
      break;
    const i = r + 8, s = e.indexOf("\n```", i);
    if (s != -1) {
      const o = s + 4, l = e.slice(i, s);
      t.push({ start: r, end: o, content: l, closed: !0 }), n = o;
    } else {
      const o = e.slice(i);
      t.push({ start: r, end: e.length, content: o, closed: !1 });
      break;
    }
  }
  return t;
}
function xt(e) {
  if (!Be(e) || ye(e) != "object")
    return !1;
  const n = e.type;
  return ye(n) != "string" ? !1 : Ne.includes(n);
}
function Ct(e) {
  const t = RegExp('"type"\\s*:\\s*"([^"]*)"'), n = e.match(t);
  if (n == null)
    return null;
  const r = n[1];
  for (const i of Ne) {
    const s = i.startsWith(r), o = r.startsWith(i);
    if (s || o)
      return i;
  }
  return null;
}
function St(e) {
  const t = wt(e);
  let n = [], r = 0;
  for (const i of t) {
    const s = String(i.start);
    i.start > r && n.push({ type: "markdown", text: e.slice(r, i.start) });
    const o = vt(i.content), l = o.value, f = o.valid, u = Ct(i.content);
    if (xt(l)) {
      let c = {};
      for (const [h, a] of Object.entries(l))
        h != "type" && (c[h] = a);
      re[s] = c, n.push({ type: "component", componentType: l.type, props: c, final: f && i.closed });
    } else if (u != null) {
      let c = re[s], h = {};
      u == "table" && (h = { columns: [], rows: [] });
      let a = l;
      a == null && (a = c), a == null && (a = h), Be(l) && (re[s] = l), n.push({ type: "component", componentType: u, props: a, final: f && i.closed });
    } else {
      let c = e.slice(i.start, i.end);
      i.closed || (c = c + "\n```"), n.push({ type: "markdown", text: c });
    }
    r = i.end;
  }
  return r < e.length && n.push({ type: "markdown", text: e.slice(r) }), n;
}
function Et(e) {
  return { segments: L(() => St(e.value)) };
}
function _t(e, t) {
  let n = e;
  n == null && (n = []);
  let r = t;
  return r == null && (r = []), [n, r];
}
const Rt = {
  key: 0,
  class: "loading-row"
}, Tt = ["colspan"], Bt = /* @__PURE__ */ ae({
  __name: "StreamingTable",
  props: {
    columns: { default: () => [] },
    rows: { default: () => [] },
    final: { type: Boolean, default: !1 }
  },
  setup(e) {
    const t = e, n = L(() => _t(t.columns, t.rows)), r = L(() => n.value[0]), i = L(() => n.value[1]);
    return (s, o) => (T(), N("div", {
      class: Ie(["streaming-table", { final: e.final }])
    }, [
      P("table", null, [
        P("thead", null, [
          P("tr", null, [
            (T(!0), N(j, null, H(r.value, (l) => (T(), N("th", { key: l }, he(l), 1))), 128))
          ])
        ]),
        P("tbody", null, [
          (T(!0), N(j, null, H(i.value, (l, f) => (T(), N("tr", { key: f }, [
            (T(!0), N(j, null, H(r.value, (u) => (T(), N("td", { key: u }, he(l[u] ?? ""), 1))), 128))
          ]))), 128)),
          e.final ? Ce("", !0) : (T(), N("tr", Rt, [
            P("td", {
              colspan: Math.max(1, r.value.length)
            }, [...o[0] || (o[0] = [
              P("span", { class: "loading-dots" }, "Loading", -1)
            ])], 8, Tt)
          ]))
        ])
      ])
    ], 2));
  }
}), Le = (e, t) => {
  const n = e.__vccOpts || e;
  for (const [r, i] of t)
    n[r] = i;
  return n;
}, Nt = /* @__PURE__ */ Le(Bt, [["__scopeId", "data-v-492993c5"]]), Lt = '<span class="codeblock-copy-icon"></span>', At = /* @__PURE__ */ ae({
  __name: "StreamingRenderer",
  props: {
    source: {},
    streaming: { type: Boolean },
    placeholderBlockId: {},
    placeholderHeight: {}
  },
  setup(e, { expose: t }) {
    const n = Pe(qe);
    kt(), yt();
    const r = e, i = L(() => o(r.source)), { segments: s } = Et(i);
    function o(g) {
      return g.replace(
        /:::details\s+(.*?)\n([\s\S]*?)\n:::/g,
        `<details>
<summary>$1</summary>
$2
</details>`
      );
    }
    const l = L(() => {
      for (let g = s.value.length - 1; g >= 0; g--)
        if (s.value[g].type === "markdown") return g;
      return -1;
    }), f = {
      showHeader: !0,
      showCopyButton: !0,
      showExpandButton: !0
    }, u = {
      table: Nt
      // Future: chart: StreamingChart, form: StreamingForm, ...
    }, c = V(null);
    function h(g) {
      g.querySelectorAll(".autodown-block-placeholder").forEach((x) => x.remove());
    }
    const a = new MutationObserver(() => {
      c.value && (k(c.value), C(c.value), R(c.value), B(c.value));
    });
    function p(g, x) {
      const d = g.firstElementChild;
      if (!d) return null;
      const _ = d.tagName.toLowerCase();
      return ["h1", "h2", "h3", "p", "pre", "blockquote", "ul", "ol", "hr", "img", "table"].includes(_) ? _ : d.classList.contains("table-node-wrapper") ? "table" : d.classList.contains("image-error") || d.classList.contains("autodown-image-wrapper") || d.querySelector(".image-node-container, .image-node__img") ? "img" : d.classList.contains("autodown-callout") || d.classList.contains("admonition") ? "callout" : d.classList.contains("autodown-details") || d.classList.contains("html-block-node") ? "details" : d.classList.contains("autodown-math-block") || d.classList.contains("math-block") ? "math" : d.classList.contains("mermaid-block-container") ? "mermaid" : x && x !== "text" ? x : null;
    }
    function b(g) {
      return g === "blockquote" || g === "ul" || g === "ol" || g === "callout" || g === "admonition";
    }
    function k(g) {
      const x = Array.from(g.querySelectorAll(".node-slot")), d = [];
      x.forEach((w) => {
        const y = w.querySelector(".node-content");
        y && (y.removeAttribute("data-block-id"), y.removeAttribute("data-block-index"));
      });
      const _ = g.getBoundingClientRect();
      if (x.forEach((w) => {
        const y = w.querySelector(".node-content");
        if (!y) return;
        const S = w.getAttribute("data-node-type"), O = p(y, S);
        if (!O) return;
        const fe = w.getBoundingClientRect(), U = fe.top - _.top, de = fe.height;
        if (d.some((Y) => b(Y.type) ? U >= Y.top && U < Y.top + Y.height : !1)) return;
        const te = d[d.length - 1];
        te && U === te.top && de === te.height || d.push({ slot: w, content: y, type: O, top: U, height: de });
      }), d.forEach(({ slot: w, content: y }, S) => {
        const O = `block-${S}`;
        y.setAttribute("data-block-id", O), y.setAttribute("data-block-index", String(S)), w.setAttribute("data-block-slot-id", O);
      }), r.placeholderBlockId != null && r.placeholderHeight != null) {
        const w = d[Number(r.placeholderBlockId.replace("block-", ""))];
        if (w && !w.slot.querySelector(":scope > .autodown-block-placeholder")) {
          const S = document.createElement("div");
          S.className = "autodown-block-placeholder", S.style.height = `${r.placeholderHeight}px`, w.slot.insertBefore(S, w.slot.firstChild);
        }
      }
    }
    async function E() {
      c.value && (await Me(), h(c.value), k(c.value), C(c.value), R(c.value), B(c.value));
    }
    function R(g) {
      Array.from(
        g.querySelectorAll("pre[data-language]:not([data-header-added])")
      ).forEach((d) => {
        const _ = d.getAttribute("data-language") || "", w = document.createElement("div");
        w.className = "codeblock-language-badge", w.setAttribute("data-codeblock-language-badge", _);
        const y = document.createElement("span");
        y.className = "codeblock-language-label", y.textContent = _;
        const S = document.createElement("button");
        S.type = "button", S.className = "codeblock-copy-btn", S.setAttribute("data-codeblock-copy-btn", ""), S.setAttribute("title", "复制"), S.innerHTML = Lt, w.appendChild(y), w.appendChild(S), d.appendChild(w), d.setAttribute("data-header-added", "");
      });
    }
    function B(g) {
      Array.from(
        g.querySelectorAll("details:not([data-details-wrapped])")
      ).forEach((d) => {
        const _ = Array.from(d.children).filter((y) => {
          const S = y.tagName.toLowerCase();
          return S !== "summary" && S !== "details" && !y.classList.contains("details-content");
        });
        if (_.length === 0) return;
        const w = document.createElement("div");
        w.className = "details-content", _.forEach((y) => w.appendChild(y)), d.appendChild(w), d.setAttribute("data-details-wrapped", "");
      });
    }
    function v(g) {
      var y, S;
      const x = g.target, d = (y = x.closest) == null ? void 0 : y.call(x, "[data-codeblock-copy-btn]");
      if (!d || !c.value) return;
      const _ = d.closest("pre"), w = ((S = _ == null ? void 0 : _.querySelector("code")) == null ? void 0 : S.textContent) ?? "";
      g.preventDefault(), g.stopPropagation(), navigator.clipboard.writeText(w);
    }
    function C(g) {
      Array.from(g.querySelectorAll("pre[data-language] > code")).forEach((d) => {
        const w = d.parentElement.getAttribute("data-language"), y = w === "plaintext" ? "text" : w;
        if (!y || y === "text" || d.getAttribute("data-highlighted") === y || !n.registered(y)) return;
        const S = d.textContent || "";
        if (S)
          try {
            const O = n.highlight(y, S);
            d.innerHTML = De(O), d.setAttribute("data-highlighted", y);
          } catch {
          }
      });
    }
    return ve(
      () => [s.value, r.placeholderBlockId, r.placeholderHeight],
      () => E(),
      { deep: !0, flush: "post" }
    ), we(() => {
      c.value && (a.observe(c.value, { childList: !0, subtree: !0 }), c.value.addEventListener("click", v, { capture: !0 }));
    }), We(() => {
      var g;
      a.disconnect(), (g = c.value) == null || g.removeEventListener("click", v, { capture: !0 });
    }), t({
      containerRef: c
    }), (g, x) => (T(), N("div", {
      ref_key: "containerRef",
      ref: c,
      class: "streaming-document"
    }, [
      (T(!0), N(j, null, H(Oe(s), (d, _) => (T(), N(j, {
        key: d.type + "-" + _
      }, [
        d.type === "markdown" ? (T(), ie(bt, {
          key: 0,
          content: d.text,
          final: !e.streaming,
          "max-live-nodes": e.streaming ? 0 : 320,
          "batch-rendering": e.streaming,
          "render-batch-size": 16,
          "render-batch-delay": 8,
          typewriter: e.streaming && _ === l.value,
          fade: !1,
          "code-block-props": f
        }, null, 8, ["content", "final", "max-live-nodes", "batch-rendering", "typewriter"])) : d.type === "component" ? (T(), ie(xe(u[d.componentType]), $e({
          key: 1,
          ref_for: !0
        }, d.props, {
          final: d.final
        }), null, 16, ["final"])) : Ce("", !0)
      ], 64))), 128))
    ], 512));
  }
}), jt = /* @__PURE__ */ Le(At, [["__scopeId", "data-v-81173826"]]);
export {
  bt as MarkdownRender,
  jt as StreamingRenderer,
  Nt as StreamingTable,
  Dt as clearOptionalCapabilities,
  Pt as enableHighlight,
  kt as enableKatex,
  yt as enableMermaid,
  qt as isCapabilityEnabled,
  He as parseDocument,
  Et as useStreamingDocument
};
