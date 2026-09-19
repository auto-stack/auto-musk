class Rt {
  constructor(t, n) {
    this.start = t, this.end = n;
  }
}
function v(e, t) {
  return new Rt(e, t);
}
const k = {
  Null: () => ({ _tag: "Null" }),
  Str: (e) => ({ _tag: "Str", value: e }),
  Int: (e) => ({ _tag: "Int", value: e }),
  Bool: (e) => ({ _tag: "Bool", value: e }),
  ListV: (e) => ({ _tag: "ListV", value: e }),
  AttrsV: (e) => ({ _tag: "AttrsV", value: e })
};
class Be {
  constructor(t, n) {
    this.key = t, this.value = n;
  }
}
function ee(e, t) {
  for (let n = 0; n < Number(e.length); n++)
    if (e[n].key == t)
      return e[n].value;
  return null;
}
function T(e, t, n) {
  const u = ee(e, t) ?? k.Str(n);
  return u._tag === "Null" ? n : u._tag === "Str" ? u.value : u._tag === "Int" || u._tag === "Bool" || u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function ot(e, t, n) {
  const u = ee(e, t) ?? k.Int(n);
  return u._tag === "Null" ? n : u._tag === "Str" ? (u.value, n) : u._tag === "Int" ? u.value : u._tag === "Bool" || u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function Te(e, t, n) {
  const u = ee(e, t) ?? k.Bool(n);
  return u._tag === "Null" ? n : u._tag === "Str" || u._tag === "Int" ? (u.value, n) : u._tag === "Bool" ? u.value : u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function S(e, t, n) {
  let l = [], r = -1;
  for (let u = 0; u < Number(e.length); u++)
    e[u].key == t && (r = u), l.push(e[u]);
  return r >= 0 ? l[r] = new Be(t, n) : l.push(new Be(t, n)), l;
}
function ge(e) {
  let t = [];
  for (const n of e)
    t.push(n);
  return t;
}
var O = /* @__PURE__ */ ((e) => (e[e.Strong = 0] = "Strong", e[e.Em = 1] = "Em", e[e.Code = 2] = "Code", e[e.Link = 3] = "Link", e[e.Image = 4] = "Image", e[e.Del = 5] = "Del", e[e.Underline = 6] = "Underline", e))(O || {});
function Y(e, t) {
  for (let n = 0; n < Number(e.length); n++)
    if (e[n] == t)
      return !0;
  return !1;
}
function X(e, t) {
  let n = [], l = !1;
  for (let r = 0; r < Number(e.length); r++)
    e[r] == t && (l = !0), n.push(e[r]);
  return l || n.push(t), n;
}
function kl(e, t) {
  let n = [];
  for (let l = 0; l < Number(e.length); l++)
    e[l] != t && n.push(e[l]);
  return n;
}
class le {
  constructor(t, n, l) {
    this.text = t, this.marks = n, this.attrs = l;
  }
}
function Z(e) {
  return new le(e, [], []);
}
function G(e, t, n) {
  return new le(e, t, n);
}
function oe(e) {
  let t = "";
  for (const n of e)
    t = t + n.text;
  return t;
}
function We(e, t, n) {
  let l = [], r = 0, u = !1;
  for (const i of e)
    if (u)
      l.push(i);
    else {
      const s = Number(i.text.length);
      if (t <= r + s) {
        const f = t - r;
        if (f === s && i.marks.length > 0)
          l.push(i), l.push(Z(n)), u = !0;
        else {
          const o = i.text.slice(0, f) + n + i.text.slice(f);
          l.push(new le(o, i.marks, i.attrs)), u = !0;
        }
      } else
        l.push(i);
      r = r + s;
    }
  return u || l.push(Z(n)), l;
}
function Pt(e, t, n) {
  let l = [], r = 0;
  for (const u of e) {
    const i = Number(u.text.length), s = r + i;
    let f = !0;
    if (s <= t && (f = !1), r >= n && (f = !1), f) {
      let o = "";
      t > r && (o = u.text.slice(0, t - r)), n < s && (o = o + u.text.slice(n - r)), Number(o.length) > 0 && l.push(new le(o, u.marks, u.attrs));
    } else
      l.push(u);
    r = s;
  }
  return l;
}
class Mt {
  constructor(t, n) {
    this.before = t, this.after = n;
  }
}
function Ot(e, t) {
  let n = [], l = [], r = 0, u = !1;
  for (const i of e)
    if (u)
      l.push(i);
    else {
      const s = Number(i.text.length), f = r + s;
      if (t <= r)
        l.push(i), u = !0;
      else if (t >= f)
        n.push(i);
      else {
        const o = i.text.slice(0, t - r), c = i.text.slice(t - r);
        Number(o.length) > 0 && n.push(new le(o, i.marks, i.attrs)), Number(c.length) > 0 && l.push(new le(c, i.marks, i.attrs)), u = !0;
      }
      r = f;
    }
  return new Mt(n, l);
}
var p = /* @__PURE__ */ ((e) => (e[e.Heading = 0] = "Heading", e[e.Paragraph = 1] = "Paragraph", e[e.Fence = 2] = "Fence", e[e.Blockquote = 3] = "Blockquote", e[e.ListBlock = 4] = "ListBlock", e[e.ListItem = 5] = "ListItem", e[e.Table = 6] = "Table", e[e.TableRow = 7] = "TableRow", e[e.TableCell = 8] = "TableCell", e[e.ThematicBreak = 9] = "ThematicBreak", e[e.Callout = 10] = "Callout", e[e.Details = 11] = "Details", e[e.WikilinkBlock = 12] = "WikilinkBlock", e[e.QueryBlock = 13] = "QueryBlock", e[e.BlockEmbed = 14] = "BlockEmbed", e[e.Mermaid = 15] = "Mermaid", e[e.MathBlock = 16] = "MathBlock", e))(p || {});
class P {
  constructor(t, n, l, r, u, i) {
    this.id = t, this.kind = n, this.attrs = l, this.children = r, this.inlines = u, this.source = i;
  }
}
function qt(e, t) {
  return new P(e, t, [], [], [], v(0, 0));
}
function I(e, t, n, l, r, u) {
  return new P(e, t, n, l, r, u);
}
function Qe(e, t) {
  return new Be(e, t);
}
function wl(e, t, n) {
  return new P(e, t, [], [], [Z(n)], v(0, Number(n.length)));
}
function Le(e) {
  return oe(e.inlines);
}
function He(e, t) {
  return new P(e.id, e.kind, e.attrs, e.children, t, e.source);
}
function Et(e, t) {
  return new P(e.id, t, e.attrs, e.children, e.inlines, e.source);
}
function de(e, t) {
  return new P(e.id, e.kind, e.attrs, t, e.inlines, e.source);
}
function ye(e) {
  let t = [];
  for (const n of e)
    t.push(n);
  return t;
}
function Cl(e) {
  return T(e.attrs, "anchor", "");
}
function Vt(e, t) {
  return new P(t, e.kind, S(e.attrs, "anchor", k.Str(t)), e.children, e.inlines, e.source);
}
function Dt(e, t) {
  return new P(t, e.kind, S(e.attrs, "anchor", k.Str(t)), e.children, e.inlines, e.source);
}
function ct(e, t) {
  if (e.id == t)
    return !0;
  for (let n = 0; n < Number(e.children.length); n++)
    if (ct(e.children[n], t))
      return !0;
  return !1;
}
function jt(e, t, n) {
  if (!ct(e, t))
    return e;
  if (e.id == t)
    return Vt(e, n);
  if (Number(e.children.length) == 0)
    return e;
  let l = [];
  for (let r = 0; r < Number(e.children.length); r++)
    l.push(jt(e.children[r], t, n));
  return new P(e.id, e.kind, e.attrs, l, e.inlines, e.source);
}
function W(e, t) {
  if (e.id == t)
    return e;
  for (let n = 0; n < Number(e.children.length); n++) {
    const l = W(e.children[n], t);
    if (l != null)
      return l;
  }
  return null;
}
function me(e, t) {
  for (let n = 0; n < Number(e.children.length); n++) {
    if (e.children[n].id == t)
      return e;
    const l = me(e.children[n], t);
    if (l != null)
      return l;
  }
  return null;
}
function be(e, t) {
  for (let n = 0; n < Number(e.children.length); n++)
    if (e.children[n].id == t)
      return n;
  return -1;
}
function at(e, t, n) {
  const l = be(e, t);
  if (l >= 0) {
    let u = [];
    for (let i = 0; i < Number(e.children.length); i++)
      if (i == l)
        for (const s of n)
          u.push(s);
      else
        u.push(e.children[i]);
    return de(e, u);
  }
  let r = [];
  for (const u of e.children)
    r.push(at(u, t, n));
  return de(e, r);
}
function $(e, t, n) {
  return e.id == t ? Number(n.length) > 0 ? n[0] : e : at(e, t, n);
}
function ht(e, t, n, l, r) {
  if (e.id == t) {
    let i = [];
    for (let s = 0; s < Number(e.children.length); s++) {
      if (s < n && i.push(e.children[s]), s == n)
        for (const f of r)
          i.push(f);
      s >= l && i.push(e.children[s]);
    }
    return de(e, i);
  }
  let u = [];
  for (const i of e.children)
    u.push(ht(i, t, n, l, r));
  return de(e, u);
}
class $e {
  constructor(t, n) {
    this.blockId = t, this.offset = n;
  }
}
class Ut {
  constructor(t, n) {
    this.anchor = t, this.head = n;
  }
}
function re(e, t) {
  return new Ut(new $e(e, t), new $e(e, t));
}
class Al {
  constructor(t, n) {
    this.pos = t, this.text = n;
  }
}
class _l {
  constructor(t, n) {
    this.pos = t, this.newId = n;
  }
}
class Sl {
  constructor(t, n) {
    this.aId = t, this.bId = n;
  }
}
class vl {
  constructor(t, n) {
    this.id = t, this.kind = n;
  }
}
class Ll {
  constructor(t, n) {
    this.sel = t, this.text = n;
  }
}
const xl = {
  InsertText: (e) => ({ _tag: "InsertText", value: e }),
  SplitBlock: (e) => ({ _tag: "SplitBlock", value: e }),
  MergeBlocks: (e) => ({ _tag: "MergeBlocks", value: e }),
  SetBlockType: (e) => ({ _tag: "SetBlockType", value: e }),
  LiftBlock: (e) => ({ _tag: "LiftBlock", value: e }),
  WrapBlock: (e) => ({ _tag: "WrapBlock", value: e }),
  ReplaceRange: (e) => ({ _tag: "ReplaceRange", value: e })
};
class _ {
  constructor(t, n) {
    this.tree = t, this.selection = n;
  }
}
function V() {
  return qt(
    "",
    1
    /* Paragraph */
  );
}
function Wt(e) {
  return e == 0 ? 1 : e;
}
function Qt(e, t) {
  return e == 0 ? [] : ge(t);
}
function Il(e, t, n) {
  const l = n;
  if (l._tag === "InsertText") {
    const r = l.value, i = W(e, r.pos.blockId) ?? V();
    if (i.id == "")
      return new _(e, t);
    const s = We(i.inlines, r.pos.offset, r.text), f = $(e, i.id, [He(i, s)]);
    return new _(f, re(r.pos.blockId, r.pos.offset + Number(r.text.length)));
  } else if (l._tag === "SplitBlock") {
    const r = l.value, i = W(e, r.pos.blockId) ?? V();
    if (i.id == "")
      return new _(e, t);
    const s = Ot(i.inlines, r.pos.offset), f = new P(i.id, i.kind, ge(i.attrs), ye(i.children), s.before, v(i.source.start, i.source.start + r.pos.offset)), o = new P(r.newId, Wt(i.kind), Qt(i.kind, i.attrs), ye(i.children), s.after, v(i.source.start + r.pos.offset, i.source.end)), c = $(e, i.id, [f, o]);
    return new _(c, re(r.newId, 0));
  } else if (l._tag === "MergeBlocks") {
    const r = l.value, i = W(e, r.aId) ?? V(), f = W(e, r.bId) ?? V();
    if (i.id == "")
      return new _(e, t);
    if (f.id == "")
      return new _(e, t);
    const o = Number(Le(i).length);
    let c = [];
    for (const b of i.inlines)
      c.push(b);
    for (const b of f.inlines)
      c.push(b);
    let g = [];
    for (const b of i.children)
      g.push(b);
    for (const b of f.children)
      g.push(b);
    const m = new P(i.id, i.kind, i.attrs, g, c, v(i.source.start, f.source.end)), a = $(e, r.bId, []), h = $(a, r.aId, [m]);
    return new _(h, re(r.aId, o));
  } else if (l._tag === "SetBlockType") {
    const r = l.value, i = W(e, r.id) ?? V();
    if (i.id == "")
      return new _(e, t);
    const s = $(e, r.id, [Et(i, r.kind)]);
    return new _(s, t);
  } else if (l._tag === "LiftBlock") {
    const r = l.value, i = W(e, r.id) ?? V();
    if (i.id == "")
      return new _(e, t);
    const f = me(e, r.id) ?? V();
    if (f.id == "")
      return new _(e, t);
    if (f.id == e.id)
      return new _(e, t);
    const o = be(f, r.id);
    let c = [], g = [];
    for (let h = 0; h < Number(f.children.length); h++)
      h < o && c.push(f.children[h]), h > o && g.push(f.children[h]);
    let m = [];
    Number(c.length) > 0 && m.push(new P(f.id, f.kind, ge(f.attrs), c, f.inlines, f.source)), m.push(i), Number(g.length) > 0 && m.push(new P(f.id + "-l", f.kind, ge(f.attrs), g, f.inlines, f.source));
    const a = $(e, f.id, m);
    return new _(a, t);
  } else if (l._tag === "WrapBlock") {
    const r = l.value, i = W(e, r.id) ?? V();
    if (i.id == "")
      return new _(e, t);
    const s = new P(r.newId, r.kind, [], [i], [], v(i.source.start, i.source.end)), f = $(e, r.id, [s]);
    return new _(f, t);
  } else if (l._tag === "ReplaceRange") {
    const r = l.value, u = r.sel;
    if (u.anchor.blockId == u.head.blockId) {
      const x = W(e, u.anchor.blockId) ?? V();
      if (x.id == "")
        return new _(e, t);
      let d = u.anchor.offset, w = u.head.offset;
      if (d > w) {
        const z = d;
        d = w, w = z;
      }
      const U = We(Pt(x.inlines, d, w), d, r.text), q = $(e, x.id, [He(x, U)]);
      return new _(q, re(u.anchor.blockId, d + Number(r.text.length)));
    }
    const s = me(e, u.anchor.blockId) ?? V(), o = me(e, u.head.blockId) ?? V();
    if (s.id == "")
      return new _(e, t);
    if (o.id == "")
      return new _(e, t);
    if (s.id != o.id)
      return new _(e, t);
    const c = be(s, u.anchor.blockId), g = be(s, u.head.blockId);
    if (c < 0)
      return new _(e, t);
    if (g < 0)
      return new _(e, t);
    if (c > g)
      return new _(e, t);
    const m = s.children[c], a = s.children[g], h = Le(m).slice(0, u.anchor.offset) + r.text + Le(a).slice(u.head.offset);
    let b = [];
    for (const L of m.children)
      b.push(L);
    for (const L of a.children)
      b.push(L);
    const N = new P(m.id, m.kind, m.attrs, b, [Z(h)], v(m.source.start, a.source.end)), C = ht(e, s.id, c, g + 1, [N]);
    return new _(C, re(u.anchor.blockId, u.anchor.offset + Number(r.text.length)));
  }
  return new _(e, t);
}
class gt {
  constructor(t, n) {
    this.cols = t, this.rows = n;
  }
}
class Ht {
  constructor(t, n) {
    this.md = t, this.tableAttrs = n;
  }
}
function M(e, t) {
  return Number(e.length) < Number(t.length) ? !1 : e.slice(0, Number(t.length)) == t;
}
function D(e, t, n) {
  return n < 0 || n + Number(t.length) > Number(e.length) ? !1 : e.slice(n, n + Number(t.length)) == t;
}
function yt(e, t) {
  return Number(e.length) < Number(t.length) ? !1 : e.slice(Number(e.length) - Number(t.length), Number(e.length)) == t;
}
function se(e) {
  let t = 0;
  for (; t < Number(e.length); ) {
    const n = e.charCodeAt(t);
    if (n == 32)
      t += 1;
    else if (n == 9)
      t += 1;
    else
      break;
  }
  return t == 0 ? e : e.slice(t);
}
function $t(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const n = e.charCodeAt(t - 1);
    if (n == 32)
      t -= 1;
    else if (n == 9)
      t -= 1;
    else if (n == 10)
      t -= 1;
    else if (n == 13)
      t -= 1;
    else
      break;
  }
  return t == Number(e.length) ? e : e.slice(0, t);
}
function pe(e, t) {
  let n = 0;
  for (; n < Number(e.length); ) {
    if (e.charCodeAt(n) == t)
      return !0;
    n += 1;
  }
  return !1;
}
function Ve(e, t) {
  const n = Number(t.length), l = Number(e.length);
  if (n == 0)
    return 0;
  if (l < n)
    return -1;
  let r = 0;
  for (; r + n <= l; ) {
    if (e.slice(r, r + n) == t)
      return r;
    r += 1;
  }
  return -1;
}
function Ne(e, t, n) {
  const l = Number(t.length), r = Number(e.length);
  if (l == 0)
    return n <= r ? n : -1;
  let u = n;
  for (u < 0 && (u = 0); u + l <= r; ) {
    if (e.slice(u, u + l) == t)
      return u;
    u += 1;
  }
  return -1;
}
function mt(e, t) {
  let n = Number(e.length) - 1;
  for (; n >= 0; ) {
    if (e.charCodeAt(n) == t)
      return n;
    n -= 1;
  }
  return -1;
}
function bt(e) {
  let t = 0, n = !1;
  Number(e.length) > 0 && (e.charCodeAt(0) == 45 ? (n = !0, t = 1) : e.charCodeAt(0) == 43 && (t = 1));
  let l = 0, r = 0;
  for (; t < Number(e.length); ) {
    const u = e.charCodeAt(t);
    if (u >= 48)
      if (u <= 57)
        l = l * 10 + u - 48, r += 1, t += 1;
      else
        break;
    else
      break;
  }
  return r == 0 ? null : (n && (l = -l), l);
}
function Gt(e) {
  const t = e.trim();
  if (Number(t.length) == 0)
    return t;
  let n = 0, l = Number(t.length);
  const r = t.charCodeAt(0);
  if ((r == 34 || r == 39) && (n = 1), l > n) {
    const u = t.charCodeAt(l - 1);
    (u == 34 || u == 39) && (l = l - 1);
  }
  return t.slice(n, l);
}
function Kt(e) {
  const t = Gt(e);
  return t == "auto" ? null : bt(t);
}
function Ge(e) {
  const t = e.split(",");
  let n = [], l = 0;
  for (; l < Number(t.length); ) {
    const r = t[l];
    n.push(Kt(r)), l += 1;
  }
  return n;
}
function Ke(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const n = e.charCodeAt(t - 1);
    if (n == 32)
      t -= 1;
    else if (n == 9)
      t -= 1;
    else
      break;
  }
  return !(t < 2 || e.charCodeAt(0) != 124 || e.charCodeAt(t - 1) != 124);
}
function zt(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const l = e.charCodeAt(t - 1);
    if (l == 32)
      t -= 1;
    else if (l == 9)
      t -= 1;
    else
      break;
  }
  if (t < 3 || e.charCodeAt(0) != 124 || e.charCodeAt(t - 1) != 124)
    return !1;
  let n = 1;
  for (; n < t - 1; ) {
    const l = e.charCodeAt(n);
    let r = !1;
    if ((l == 45 || l == 58 || l == 124 || l == 32 || l == 9) && (r = !0), !r)
      return !1;
    n += 1;
  }
  return !0;
}
function Yt(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const o = e.charCodeAt(t - 1);
    if (o == 32)
      t -= 1;
    else if (o == 9)
      t -= 1;
    else
      break;
  }
  if (t < 9)
    return null;
  const n = e.slice(0, t);
  if (!M(n, "{cols:[") || !yt(n, "}"))
    return null;
  const l = Number(n.length) - 1;
  let r = -1, u = 7;
  for (; u < l; ) {
    if (n.charCodeAt(u) == 93) {
      r = u;
      break;
    }
    u += 1;
  }
  if (r == -1)
    return null;
  const i = Ge(n.slice(7, r));
  let s = [];
  const f = n.slice(r + 1, l);
  if (f != "") {
    if (!M(f, ","))
      return null;
    let o = f.slice(1), c = !0;
    for (; c; )
      if (c = !1, Number(o.length) > 0) {
        const h = o.charCodeAt(0);
        (h == 32 || h == 9 || h == 10 || h == 13) && (o = o.slice(1), c = !0);
      }
    if (!M(o, "rows:["))
      return null;
    const g = o.slice(6);
    let m = -1, a = 0;
    for (; a < Number(g.length); ) {
      if (g.charCodeAt(a) == 93) {
        m = a;
        break;
      }
      a += 1;
    }
    if (m == -1 || m + 1 != Number(g.length))
      return null;
    s = Ge(g.slice(0, m));
  }
  return new gt(i, s);
}
function Xt(e) {
  const t = e.split(`
`);
  let n = [], l = [], r = 0;
  for (; r < Number(t.length); ) {
    let u = !1;
    if (r + 1 < Number(t.length) && Ke(t[r]) && zt(t[r + 1])) {
      let i = r + 2, s = 0, f = !0;
      for (; f; )
        f = !1, i < Number(t.length) && Ke(t[i]) && (i += 1, s += 1, f = !0);
      if (s >= 1 && i < Number(t.length)) {
        const o = Yt(t[i]);
        if (o != null) {
          n.push(o ?? new gt([], []));
          let c = r;
          for (; c < i; )
            l.push(t[c]), c += 1;
          r = i + 1, u = !0;
        }
      }
    }
    u || (l.push(t[r]), r += 1);
  }
  return new Ht(l.join(`
`), n);
}
class A {
  constructor(t, n, l, r, u, i, s, f, o, c, g, m, a, h, b, N, C, L, x, d, w) {
    this.type = t, this.content = n, this.level = l, this.language = r, this.code = u, this.loading = i, this.children = s, this.ordered = f, this.start = o, this.items = c, this.cells = g, this.header = m, this.rows = a, this.isHeader = h, this.align = b, this.href = N, this.title = C, this.text = L, this.src = x, this.alt = d, this.checked = w;
  }
}
function B() {
  return [];
}
function ze(e, t, n) {
  return new A("code_block", null, null, e, t, n, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Ye(e, t) {
  return new A("heading", null, e, null, null, null, t, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Jt() {
  return new A("thematic_break", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Zt(e) {
  return new A("blockquote", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Ft(e) {
  return new A("paragraph", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function dt(e, t, n) {
  return new A("table", null, null, null, null, n, null, null, null, null, null, e, t, null, null, null, null, null, null, null, null);
}
function Re(e) {
  return new A("table_row", null, null, null, null, null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null);
}
function Pe(e, t, n) {
  return new A("table_cell", null, null, null, null, null, t, null, null, null, null, null, null, e, n, null, null, null, null, null, null);
}
function xe(e, t, n) {
  return new A("list", null, null, null, null, null, null, e, t, n, null, null, null, null, null, null, null, null, null, null, null);
}
function en(e, t) {
  return new A("list_item", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, t);
}
function Xe(e) {
  return new A("strong", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function ce(e) {
  return new A("emphasis", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Je(e) {
  return new A("underline", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function tn(e) {
  return new A("strikethrough", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Ze(e) {
  return new A("inline_code", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function nn() {
  return new A("hardbreak", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function ln(e, t) {
  return new A("image", null, null, null, null, !1, null, null, null, null, null, null, null, null, null, null, null, null, e, t, null);
}
function Fe(e, t, n, l, r) {
  return new A("link", null, null, null, null, r, l, null, null, null, null, null, null, null, null, e, t, n, null, null, null);
}
function rn(e) {
  return new A("wikilink", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, e, null, null, null, null);
}
function un(e) {
  return new A("math_inline", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function sn(e, t, n) {
  return new A("callout", null, null, e, null, null, n, null, null, null, null, null, null, null, null, null, t, null, null, null, null);
}
function fn(e, t, n) {
  return new A("details", null, null, null, null, t, n, null, null, null, null, null, null, null, null, null, null, e, null, null, null);
}
function on(e) {
  return new A("query", e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function cn(e) {
  return new A("embed", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, e, null, null);
}
function an(e) {
  return new A("math_block", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function hn(e) {
  return new A("mermaid", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function pt(e) {
  return E(e) >= 4 ? !1 : e.trim() == "%{";
}
function gn(e) {
  if (Number(e.length) < 2 || e.charCodeAt(0) != 125 || e.charCodeAt(1) != 37)
    return !1;
  let t = 2;
  for (; t < Number(e.length); ) {
    if (e.charCodeAt(t) != 32)
      return !1;
    t += 1;
  }
  return !0;
}
function mn(e) {
  return new A("text", e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
class Q {
  constructor(t, n) {
    this.next = t, this.inner = n;
  }
}
class te {
  constructor(t, n, l, r, u, i) {
    this.next = t, this.text = n, this.href = l, this.loading = r, this.title = u, this.tail = i;
  }
}
function bn(e) {
  return e.split(`\r
`).join(`
`).split("\r").join(`
`);
}
function dn(e) {
  const t = mt(e, 10);
  if (t == -1)
    return e;
  let n = t + 1, l = 0;
  for (; n < Number(e.length) && e.charCodeAt(n) == 32; )
    l += 1, n += 1;
  if (l > 3 || n >= Number(e.length))
    return e;
  let r = e.charCodeAt(n), u = -1;
  if ((r == 45 || r == 42 || r == 43) && (u = n + 1), u == -1) {
    let s = n, f = 0;
    for (; s < Number(e.length); ) {
      const c = e.charCodeAt(s);
      if (c >= 48)
        if (c <= 57)
          f += 1, s += 1;
        else
          break;
      else
        break;
    }
    if (f == 0 || f > 9 || s >= Number(e.length))
      return e;
    const o = e.charCodeAt(s);
    if (o == 46)
      u = s + 1;
    else if (o == 41)
      u = s + 1;
    else
      return e;
  }
  let i = u;
  for (; i < Number(e.length); )
    if (e.charCodeAt(i) == 32)
      i += 1;
    else
      return e;
  return e.slice(0, t);
}
function pn(e) {
  const t = mt(e, 10);
  if (t == -1)
    return e;
  let n = t + 1, l = 0;
  for (; n < Number(e.length) && e.charCodeAt(n) == 32; )
    l += 1, n += 1;
  if (l > 3 || n >= Number(e.length) || e.charCodeAt(n) != 62)
    return e;
  let r = n + 1;
  for (; r < Number(e.length); )
    if (e.charCodeAt(r) == 32)
      r += 1;
    else
      return e;
  return e.slice(0, t);
}
function Nn(e, t) {
  if (t)
    return e;
  let n = e, l = !0;
  for (; l; ) {
    l = !1;
    const r = dn(n);
    r != n && (n = r, l = !0);
    const u = pn(n);
    u != n && (n = u, l = !0);
  }
  return n;
}
function H(e) {
  return e.trim() == "";
}
function ue(e) {
  let t = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    t += 1;
  return t;
}
function E(e) {
  let t = 0, n = 0;
  for (; n < Number(e.length); ) {
    const l = e.charCodeAt(n);
    if (l == 32)
      t += 1;
    else if (l == 9)
      t += 4;
    else
      break;
    n += 1;
  }
  return t;
}
function kn(e, t) {
  const n = bn(e), r = Nn(n, t).split(`
`);
  return Ae(r, t);
}
function Nt(e) {
  if (E(e) >= 4)
    return "";
  const t = e.trim();
  return M(t, "```") ? "`" : M(t, "~~~") ? "~" : "";
}
function wn(e) {
  const t = e.trim();
  let n = "", l = 0;
  if (Number(t.length) == 0)
    return "";
  const r = t.charCodeAt(0);
  if (r != 96 && r != 126)
    return "";
  for (; l < Number(t.length) && t.charCodeAt(l) == r; )
    n += t.slice(l, l + 1), l += 1;
  return n;
}
function Cn(e, t, n) {
  if (E(e) >= 4)
    return !1;
  const l = e.trim();
  if (Number(l.length) != Number(n.length))
    return !1;
  let r = 0, u = 96;
  for (t == "~" && (u = 126); r < Number(l.length); ) {
    if (l.charCodeAt(r) != u)
      return !1;
    r += 1;
  }
  return !0;
}
function kt(e) {
  if (E(e) >= 4)
    return 0;
  const t = e.trim();
  let n = 0;
  for (; n < Number(t.length) && t.charCodeAt(n) == 35; )
    n += 1;
  return n == 0 || n > 6 ? 0 : n == Number(t.length) ? n : t.charCodeAt(n) != 32 ? 0 : n;
}
function An(e) {
  const t = e.trim();
  if (Number(t.length) == 0 || t.charCodeAt(Number(t.length) - 1) != 35)
    return t;
  let n = Number(t.length) - 1;
  for (; n > 0 && t.charCodeAt(n - 1) == 35; )
    n -= 1;
  let l = n;
  for (; l > 0 && t.charCodeAt(l - 1) == 32; )
    l -= 1;
  if (l == n)
    return t;
  const r = t.slice(0, l);
  return $t(r);
}
function ke(e) {
  if (E(e) >= 4)
    return -1;
  let t = 0, n = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    n += 1, t += 1;
  if (n > 3)
    return -1;
  let l = t, r = 0;
  for (; l < Number(e.length); ) {
    const o = e.charCodeAt(l);
    if (o >= 48)
      if (o <= 57)
        r += 1, l += 1;
      else
        break;
    else
      break;
  }
  if (r == 0 || r > 9 || l >= Number(e.length))
    return -1;
  const u = e.charCodeAt(l);
  if (u != 46 && u != 41)
    return -1;
  let i = l + 1, s = !1;
  return (i == Number(e.length) || e.charCodeAt(i) == 32) && (s = !0), s ? bt(e.slice(t, l)) ?? -1 : -1;
}
function we(e) {
  if (E(e) >= 4)
    return "";
  let t = 0, n = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    n += 1, t += 1;
  if (n > 3 || t >= Number(e.length))
    return "";
  const l = e.charCodeAt(t);
  let r = !1;
  return (l == 45 || l == 42 || l == 43) && (r = !0), !r || t + 1 >= Number(e.length) || e.charCodeAt(t + 1) != 32 ? "" : e.slice(t, t + 1);
}
function Ce(e) {
  if (E(e) >= 4)
    return !1;
  let t = 0, n = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    n += 1, t += 1;
  if (n > 3 || t + 1 != Number(e.length))
    return !1;
  const l = e.charCodeAt(t);
  return l == 45 || l == 42 || l == 43;
}
function wt(e) {
  if (E(e) >= 4)
    return !1;
  const t = e.trim();
  if (Number(t.length) < 3)
    return !1;
  const n = t.charCodeAt(0);
  if (n != 45 && n != 42 && n != 95)
    return !1;
  let l = 0, r = 0;
  for (; r < Number(t.length); ) {
    const u = t.charCodeAt(r);
    if (u == n)
      l += 1;
    else if (u != 32)
      return !1;
    r += 1;
  }
  return l >= 3;
}
function _n(e) {
  if (E(e) >= 4)
    return 0;
  const t = e.trim();
  if (Number(t.length) == 0)
    return 0;
  const n = t.charCodeAt(0);
  if (n != 61 && n != 45)
    return 0;
  let l = 0;
  for (; l < Number(t.length); ) {
    if (t.charCodeAt(l) != n)
      return 0;
    l += 1;
  }
  return n == 61 ? 1 : 2;
}
function Me(e) {
  return E(e) >= 4 ? !1 : M(se(e), ">");
}
function Sn(e) {
  const n = se(e).slice(1);
  return M(n, " ") ? n.slice(1) : n;
}
function Oe(e) {
  return H(e) ? !1 : pe(e, 124);
}
function vn(e) {
  if (Number(e.length) == 0)
    return !1;
  let t = 0;
  if (e.charCodeAt(0) == 58 && (t = 1), t >= Number(e.length))
    return !1;
  let n = t;
  for (; n < Number(e.length) && e.charCodeAt(n) == 45; )
    n += 1;
  return n == t ? !1 : n == Number(e.length) || n + 1 == Number(e.length) && e.charCodeAt(n) == 58;
}
function Ln(e) {
  if (H(e) || !pe(e, 45))
    return !1;
  const t = F(e);
  if (Number(t.length) == 0)
    return !1;
  for (const n of t) {
    const l = n.trim();
    if (!vn(l))
      return !1;
  }
  return !0;
}
function F(e) {
  let t = e.trim();
  M(t, "|") && (t = t.slice(1)), y(t, "|") && (t = t.slice(0, Number(t.length) - 1));
  let n = [], l = "", r = 0;
  for (; r < Number(t.length); ) {
    const u = t.charCodeAt(r);
    if (u == 92 && r + 1 < Number(t.length)) {
      const i = t.charCodeAt(r + 1);
      if (i == 124) {
        l = l + "|", r += 2;
        continue;
      }
      if (i == 92) {
        l = l + "\\", r += 2;
        continue;
      }
      l = l + "\\", r += 1;
      continue;
    }
    if (u == 124) {
      n.push(l), l = "", r += 1;
      continue;
    }
    l = l + t.slice(r, r + 1), r += 1;
  }
  return n.push(l), n;
}
function xn(e) {
  const t = e.trim(), n = M(t, ":"), l = y(t, ":");
  return n ? l ? "center" : "left" : l ? "right" : "left";
}
function In(e) {
  if (Number(e.length) == 0)
    return !1;
  let t = 0;
  for (; t < Number(e.length); ) {
    const n = e.charCodeAt(t);
    if (n != 96) {
      if (n != 126) return !1;
    }
    t += 1;
  }
  return !0;
}
function Bn(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) || t == 0 ? e : e.charCodeAt(t - 1) == 10 ? e.slice(0, t) : e;
}
function Tn(e) {
  let t = 0;
  for (; t < 4 && t < Number(e.length); )
    if (e.charCodeAt(t) == 32)
      t += 1;
    else
      break;
  return t == 0 ? e : e.slice(t);
}
class De {
  constructor(t, n, l) {
    this.name = t, this.argstr = n, this.afterParen = l;
  }
}
function Rn(e) {
  return e >= 97 && e <= 122 || e >= 65 && e <= 90 || e >= 48 && e <= 57 || e == 95;
}
function je(e) {
  let t = 0, n = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    n += 1, t += 1;
  if (n > 3 || t >= Number(e.length) || e.charCodeAt(t) != 36)
    return null;
  let l = t + 1;
  const r = l;
  for (; l < Number(e.length) && Rn(e.charCodeAt(l)); )
    l += 1;
  if (l == r || l >= Number(e.length) || e.charCodeAt(l) != 40)
    return null;
  let u = l + 1, i = !1;
  for (; u < Number(e.length); ) {
    const o = e.charCodeAt(u);
    if (i)
      o == 34 && (i = !1);
    else if (o == 34)
      i = !0;
    else if (o == 41)
      break;
    u += 1;
  }
  if (u >= Number(e.length))
    return null;
  const s = e.slice(r, l), f = e.slice(l + 1, u);
  return new De(s, f, u);
}
function et(e) {
  const t = je(e);
  if (t == null)
    return !1;
  const n = t ?? new De("", "", 0);
  if (n.name != "callout" && n.name != "details")
    return !1;
  let l = n.afterParen + 1;
  for (; l < Number(e.length) && e.charCodeAt(l) == 32; )
    l += 1;
  if (l >= Number(e.length) || e.charCodeAt(l) != 123)
    return !1;
  let r = l + 1;
  for (; r < Number(e.length); ) {
    if (e.charCodeAt(r) != 32)
      return !1;
    r += 1;
  }
  return !0;
}
function tt(e, t) {
  let n = t + 1;
  for (; n < Number(e.length); ) {
    if (e.charCodeAt(n) != 32)
      return !1;
    n += 1;
  }
  return !0;
}
function Pn(e) {
  if (Number(e.length) == 0 || e.charCodeAt(0) != 125)
    return !1;
  let t = 1;
  for (; t < Number(e.length); ) {
    if (e.charCodeAt(t) != 32)
      return !1;
    t += 1;
  }
  return !0;
}
function Ct(e, t) {
  const n = t + ":";
  let l = 0;
  const r = Number(e.length);
  for (; l < r; ) {
    const u = e.charCodeAt(l);
    (u == 44 || u == 32) && (l += 1);
    let i = l;
    for (; i < r && e.charCodeAt(i) == 32; )
      i += 1;
    if (i + Number(n.length) <= r && e.slice(i, i + Number(n.length)) == n) {
      let f = i + Number(n.length);
      for (; f < r && e.charCodeAt(f) == 32; )
        f += 1;
      if (f < r)
        return f;
    }
    let s = !1;
    for (; l < r; ) {
      const f = e.charCodeAt(l);
      if (s)
        f == 34 && (s = !1);
      else if (f == 34)
        s = !0;
      else if (f == 44)
        break;
      l += 1;
    }
    l < r && (l += 1);
  }
  return -1;
}
function ae(e, t) {
  const n = Ct(e, t);
  if (n == -1 || e.charCodeAt(n) != 34)
    return null;
  let l = n + 1;
  const r = Number(e.length);
  for (; l < r && e.charCodeAt(l) != 34; )
    l += 1;
  return l >= r ? null : e.slice(n + 1, l);
}
function Mn(e, t) {
  const n = Ct(e, t);
  return n == -1 ? null : e.slice(n, n + 4) == "true" ? !0 : e.slice(n, n + 5) == "false" ? !1 : null;
}
function Ae(e, t) {
  let n = [], l = 0;
  for (; l < Number(e.length); ) {
    const r = e[l];
    if (H(r)) {
      l += 1;
      continue;
    }
    const u = Nt(r);
    if (u != "") {
      const a = wn(r), b = r.trim().slice(Number(a.length)).trim();
      let N = [], C = l + 1, L = !1;
      for (; C < Number(e.length); ) {
        if (Cn(e[C], u, a)) {
          L = !0;
          break;
        }
        N.push(e[C]), C += 1;
      }
      let x = N.join(`
`);
      if (L)
        if (Number(N.length) > 0 ? x += `
` : x = "", b == "mermaid") {
          const d = N.join(`
`);
          n.push(hn(d));
        } else
          n.push(ze(b, x, !1));
      else {
        for (; Number(N.length) > 0; ) {
          const w = N[Number(N.length) - 1].trim();
          if (w == "" || !In(w))
            break;
          N.pop();
        }
        let d = N.join(`
`);
        d = Bn(d), n.push(ze(b, d, !t));
      }
      l = C + 1;
      continue;
    }
    if (pt(r)) {
      let a = [], h = l + 1, b = !1;
      for (; h < Number(e.length); ) {
        if (gn(e[h])) {
          b = !0;
          break;
        }
        a.push(e[h]), h += 1;
      }
      if (b) {
        const N = a.join(`
`);
        n.push(an(N)), l = h + 1;
        continue;
      }
    }
    const i = kt(r);
    if (i > 0) {
      const h = r.trim().slice(i).trim(), b = An(h);
      let N = ne(b, t);
      b == "" && (N = B()), n.push(Ye(i, N)), l += 1;
      continue;
    }
    if (wt(r)) {
      n.push(Jt()), l += 1;
      continue;
    }
    const s = je(r);
    if (s != null) {
      const a = s ?? new De("", "", 0);
      if (et(r)) {
        let h = [], b = l + 1, N = 1, C = !1;
        for (; b < Number(e.length); ) {
          if (Pn(e[b])) {
            if (N -= 1, N == 0) {
              C = !0;
              break;
            }
          } else
            et(e[b]) && (N += 1);
          h.push(e[b]), b += 1;
        }
        if (C) {
          const L = Ae(h, t);
          if (a.name == "callout") {
            const x = ae(a.argstr, "title") ?? "";
            n.push(sn(ae(a.argstr, "type") ?? "", x, L));
          } else {
            const x = Mn(a.argstr, "open") ?? !1;
            n.push(fn(ae(a.argstr, "summary") ?? "", x, L));
          }
          l = b + 1;
          continue;
        }
      } else {
        if (a.name == "query" && tt(r, a.afterParen)) {
          n.push(on(a.argstr.trim())), l += 1;
          continue;
        }
        if (a.name == "embed" && tt(r, a.afterParen)) {
          n.push(cn(ae(a.argstr, "src") ?? "")), l += 1;
          continue;
        }
      }
    }
    if (Me(r)) {
      let a = [], h = l;
      for (; h < Number(e.length); )
        if (Me(e[h]))
          a.push(Sn(e[h])), h += 1;
        else {
          if (H(e[h]))
            break;
          if (On(e[h]))
            a.push(e[h]), h += 1;
          else
            break;
        }
      const b = Ae(a, t);
      n.push(Zt(b)), l = h;
      continue;
    }
    if (Oe(r) && l + 1 < Number(e.length) && Ln(e[l + 1])) {
      const a = Number(F(r).length), h = Number(F(e[l + 1]).length);
      if (a == h) {
        l = qn(e, l, n, t);
        continue;
      }
    }
    const f = ke(r), o = we(r);
    if (f >= 0) {
      l = Ie(e, l, !0, f, n, t);
      continue;
    }
    if (o != "") {
      l = Ie(e, l, !1, 0, n, t);
      continue;
    }
    if (Ce(r)) {
      l = Ie(e, l, !1, 0, n, t);
      continue;
    }
    let c = [], g = l, m = 0;
    for (; g < Number(e.length); ) {
      const a = e[g];
      if (H(a))
        break;
      if (Number(c.length) > 0) {
        const h = _n(a);
        if (h > 0) {
          m = h, g += 1;
          break;
        }
      }
      if (_e(a, e, g))
        break;
      c.push(Tn(a)), g += 1;
    }
    if (m > 0) {
      const a = c.join(`
`), h = ne(a, t);
      n.push(Ye(m, h)), l = g;
      continue;
    }
    if (Number(c.length) > 0) {
      if (!t) {
        let b = !1;
        if (Number(c.length) >= 2 && (b = !0), g < Number(e.length) && (b = !0), b) {
          const N = c[0];
          if (Oe(N) && y(N.trim(), "|")) {
            const C = F(N);
            if (Number(C.length) >= 2) {
              let L = !0, x = 0;
              for (; x < Number(c.length); ) {
                const d = c[x].trim();
                M(d, "|") || (L = !1), x += 1;
              }
              if (L) {
                let d = [];
                for (const U of C) {
                  const q = U.trim(), z = ne(q, t);
                  d.push(Pe(!0, z, "left"));
                }
                const w = Re(d);
                n.push(dt([w], B(), !0)), l = g;
                continue;
              }
            }
          }
        }
      }
      const a = c.join(`
`), h = ne(a, t);
      n.push(Ft(h)), l = g;
      continue;
    }
    l += 1;
  }
  return n;
}
function _e(e, t, n) {
  return n == 0 ? !1 : !!(Nt(e) != "" || kt(e) > 0 || wt(e) || Me(e) || we(e) != "" || Ce(e) || ke(e) >= 0 || je(e) != null || pt(e));
}
function On(e) {
  return !_e(e, [], 0);
}
function Ie(e, t, n, l, r, u) {
  let i = [], s = t, f = null;
  for (n && l != 1 && (f = l); s < Number(e.length); ) {
    const o = e[s];
    if (H(o)) {
      let d = s + 1;
      for (; d < Number(e.length) && H(e[d]); )
        d += 1;
      if (d < Number(e.length)) {
        if (we(e[d]) != "") {
          s = d;
          continue;
        }
        if (Ce(e[d])) {
          s = d;
          continue;
        }
        if (ke(e[d]) >= 0) {
          s = d;
          continue;
        }
      }
      break;
    }
    let c = 0, g = 0, m = !1, a = !1;
    const h = we(o), b = ke(o);
    if (h != "")
      c = ue(o), g = 2 + ue(o), m = !0, a = !1;
    else if (Ce(o))
      c = ue(o), g = 1 + ue(o), m = !0, a = !1;
    else if (b >= 0) {
      c = ue(o);
      const d = se(o);
      let w = 0;
      for (; w < Number(d.length); ) {
        const Ue = d.charCodeAt(w);
        if (Ue >= 48)
          if (Ue <= 57)
            w += 1;
          else
            break;
        else
          break;
      }
      let U = w + 1, q = 0, z = w + 1;
      for (; z < Number(d.length) && d.charCodeAt(z) == 32; )
        q += 1, z += 1;
      q > 4 && (q = 1), q < 1 && (q = 1), g = c + U + q, m = !0, a = !0;
    }
    if (!m || a != n)
      break;
    if (!n) {
      const w = se(o).slice(0, 1), q = se(e[t]).slice(0, 1);
      if (w != q)
        break;
    }
    let N = [], C = o.slice(g), L = null;
    for (a || (M(C, "[ ] ") ? (C = C.slice(4), L = !1) : (M(C, "[x] ") || M(C, "[X] ")) && (C = C.slice(4), L = !0)), N.push(C), s += 1; s < Number(e.length); ) {
      const d = e[s];
      if (H(d)) {
        let w = s + 1;
        for (; w < Number(e.length) && H(e[w]); )
          w += 1;
        if (w < Number(e.length) && E(e[w]) >= g) {
          _e(e[w], e, w);
          let U = s;
          for (; U < w; )
            N.push(""), U += 1;
          s = w;
          continue;
        }
        break;
      }
      if (E(d) >= g) {
        let w = d.slice(g);
        N.push(w), s += 1;
        continue;
      }
      if (!_e(d, e, s)) {
        N.push(d), s += 1;
        continue;
      }
      break;
    }
    const x = Ae(N, u);
    i.push(en(x, L));
  }
  return n ? f != null ? r.push(xe(!0, f, i)) : r.push(xe(!0, null, i)) : r.push(xe(!1, null, i)), s;
}
function qn(e, t, n, l) {
  const r = F(e[t]), u = [], i = F(e[t + 1]);
  for (const m of i)
    u.push(xn(m));
  let s = [], f = t + 2;
  for (; f < Number(e.length) && Oe(e[f]); ) {
    const m = F(e[f]);
    let a = [], h = 0;
    for (; h < Number(r.length); ) {
      let b = "";
      h < Number(m.length) && (b = m[h].trim());
      let N = "left";
      h < Number(u.length) && (N = u[h]);
      let C = ne(b, l);
      b == "" && (C = B()), a.push(Pe(!1, C, N)), h += 1;
    }
    s.push(Re(a)), f += 1;
  }
  let o = [], c = 0;
  for (; c < Number(r.length); ) {
    const m = r[c].trim();
    let a = "left";
    c < Number(u.length) && (a = u[c]);
    const h = ne(m, l);
    o.push(Pe(!0, h, a)), c += 1;
  }
  const g = Re(o);
  return n.push(dt([g], s, !1)), f;
}
function ne(e, t) {
  return e == "" ? B() : j(e, t);
}
function j(e, t) {
  let n = [], l = "", r = 0, u = !1;
  const i = Number(e.length);
  for (; r < i; ) {
    const s = e.slice(r, r + 1);
    if (s == `
`) {
      let f = 0;
      for (; f < Number(l.length) && l.charCodeAt(Number(l.length) - 1 - f) == 32; )
        f += 1;
      if (f >= 2) {
        const o = l.slice(0, Number(l.length) - f);
        o != "" && n.push(R(o)), n.push(nn()), l = "";
      } else
        f > 0 && (l = l.slice(0, Number(l.length) - f)), l += `
`;
      r += 1;
      continue;
    }
    if (s == "*") {
      if (D(e, "***", r)) {
        let o = J(e, r, "***", !1, t);
        if (o != null) {
          const c = o ?? new Q(0, "");
          l != "" && (n.push(R(l)), l = ""), n.push(Xe([ce(j(c.inner, t))])), r = c.next;
          continue;
        }
      }
      if (D(e, "**", r)) {
        let o = J(e, r, "**", !0, t);
        if (o != null) {
          const c = o ?? new Q(0, "");
          l != "" && (n.push(R(l)), l = ""), n.push(Xe(j(c.inner, t))), r = c.next;
          continue;
        }
      }
      let f = J(e, r, "*", !1, t);
      if (f != null) {
        const o = f ?? new Q(0, "");
        l != "" && (n.push(R(l)), l = ""), n.push(ce(j(o.inner, t))), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "_") {
      if (D(e, "___", r)) {
        let o = J(e, r, "___", !1, t);
        if (o != null) {
          const c = o ?? new Q(0, "");
          l != "" && (n.push(R(l)), l = ""), n.push(Je([ce(j(c.inner, t))])), r = c.next;
          continue;
        }
      }
      if (D(e, "__", r)) {
        let o = J(e, r, "__", !1, t);
        if (o != null) {
          const c = o ?? new Q(0, "");
          l != "" && (n.push(R(l)), l = ""), n.push(Je(j(c.inner, t))), r = c.next;
          continue;
        }
      }
      let f = J(e, r, "_", !1, t);
      if (f != null) {
        const o = f ?? new Q(0, "");
        l != "" && (n.push(R(l)), l = ""), n.push(ce(j(o.inner, t))), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "~") {
      if (D(e, "~~", r)) {
        let f = J(e, r, "~~", !1, t);
        if (f != null) {
          const o = f ?? new Q(0, "");
          l != "" && (n.push(R(l)), l = ""), n.push(tn(j(o.inner, t))), r = o.next;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "`") {
      let f = 0;
      for (; D(e, "`", r + f); )
        f += 1;
      let o = Kn(e, r + f, f);
      if (o != -1) {
        let c = e.slice(r + f, o);
        M(c, " ") && y(c, " ") && c.trim() != "" && (c = c.slice(1, Number(c.length) - 1)), l != "" && (n.push(R(l)), l = ""), n.push(Ze(c)), u = !0, r = o + f;
        continue;
      }
      if (!t && f == 1 && e.slice(r + f).trim() == "" && l == "") {
        r = Number(e.length);
        continue;
      }
      if (f == 1 && !t) {
        const c = e.slice(r + 1);
        l != "" && (n.push(R(l)), l = ""), n.push(Ze(c)), u = !0, r = Number(e.length);
        continue;
      }
      l += e.slice(r, r + f), r += f;
      continue;
    }
    if (s == "!") {
      if (D(e, "![", r)) {
        let f = lt(e, r + 1, t, u);
        if (f != null) {
          const o = f ?? new te(0, "", "", !1, null, "");
          l != "" && (n.push(R(l)), l = ""), n.push(ln(o.href, o.text)), r = o.next;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "[") {
      if (D(e, "[[", r)) {
        if (D(e, "[[[", r)) {
          l += "[", r += 1;
          continue;
        }
        let o = Ne(e, "]]", r + 2);
        if (o != -1) {
          let c = e.slice(r + 2, o).trim();
          if (Ve(c, "|") == -1 && !pe(c, 10) && c != "") {
            l != "" && (n.push(R(l)), l = ""), n.push(rn(c)), r = o + 2;
            continue;
          }
        }
        l += "[", r += 1;
        continue;
      }
      let f = lt(e, r, t, u);
      if (f != null) {
        const o = f ?? new te(0, "", "", !1, null, "");
        l != "" && (n.push(R(l)), l = ""), o.loading ? (n.push(Fe(o.href, o.title, o.text, j(o.text, t), !0)), o.tail != "" && n.push(R(o.tail))) : n.push(Fe(o.href, o.title, o.text, j(o.text, t), !1)), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "$") {
      let f = Ne(e, "$", r + 1);
      if (f != -1) {
        let o = e.slice(r + 1, f), c = Number(o.length), g = 0;
        f + 1 < i && (g = e.charCodeAt(f + 1));
        let m = g >= 48 && g <= 57, a = c > 0 && o.charCodeAt(0) != 32 && !pe(o, 10), h = c > 0 && o.charCodeAt(c - 1) != 32 && !m;
        if (a && h) {
          l != "" && (n.push(R(l)), l = ""), n.push(un(o)), r = f + 1;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "\\" && r + 1 < Number(e.length)) {
      const f = e.charCodeAt(r + 1);
      if (At(f)) {
        f == 34 ? l += "" : f == 39 ? l += "" : l += e.slice(r + 1, r + 2), r += 2;
        continue;
      }
    }
    l += s, r += 1;
  }
  return l != "" && n.push(R(l)), t || En(n), n;
}
function En(e) {
  if (Number(e.length) > 0) {
    const t = e[Number(e.length) - 1];
    t.type == "text" && Wn(t, e);
  }
}
function Vn(e) {
  let t = -1, n = 0;
  for (; n < Number(e.length); )
    e.charCodeAt(n) == 62 && (t = n), n += 1;
  let l = t + 1;
  for (; l < Number(e.length); ) {
    if (e.charCodeAt(l) == 60) {
      let r = l + 1;
      if (r >= Number(e.length))
        return e;
      let u = e.charCodeAt(r);
      if (u == 47) {
        if (r += 1, r >= Number(e.length))
          return e;
        u = e.charCodeAt(r);
      }
      let i = !1;
      if (u == 33 ? i = !0 : (u >= 65 && u <= 90 && (i = !0), u >= 97 && u <= 122 && (i = !0)), !i)
        return e;
      let s = r + 1, f = !0;
      for (; s < Number(e.length); ) {
        if (e.charCodeAt(s) == 62) {
          f = !1;
          break;
        }
        s += 1;
      }
      if (!f)
        return e;
      let o = l;
      return l > 0 && e.charCodeAt(l - 1) == 32 && (o = l - 1), e.slice(0, o);
    }
    l += 1;
  }
  return e;
}
function Dn(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const l = e.charCodeAt(t - 1);
    if (l == 32)
      t -= 1;
    else if (l == 9)
      t -= 1;
    else if (l == 10)
      t -= 1;
    else if (l == 13)
      t -= 1;
    else
      break;
  }
  let n = t;
  for (; n > 0 && e.charCodeAt(n - 1) == 40; )
    n -= 1;
  return n == t ? e : e.slice(0, n);
}
function jn(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) || t == 0 ? e : e.charCodeAt(t - 1) == 42 ? e.slice(0, t - 1) : e;
}
function Un(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) ? e : e.slice(0, t);
}
function Wn(e, t) {
  let n = e.content ?? "", l = !1, r = Vn(n);
  r == n && y(n, "<") && (r = n.slice(0, Number(n.length) - 1)), r != n && (l = !0), n = r;
  let u = Dn(n);
  u != n && (l = !0), n = u;
  let i = jn(n);
  i == n && y(n, "*") && (y(n, "**") || (i = n.slice(0, Number(n.length) - 1))), i != n && (l = !0), n = i, l || (n = Un(n)), n.trim() == "|" && (n = ""), n == "" ? t.pop() : (t.pop(), t.push(mn(n)));
}
function R(e) {
  let t = $n(e);
  return t = t.split("").join('"'), t = t.split("").join("'"), new A("text", t, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function fe(e) {
  return e >= 48 && e <= 57 || e >= 65 && e <= 90 || e >= 97 && e <= 122 || e == 95;
}
function Qn(e) {
  return e == 41 || e == 93 || e == 125 || e == 44 || e == 46 || e == 59 || e == 58 || e == 33 || e == 63 || e == 8230 || e == 34 || e == 39 || e == 65289 || e == 65292 || e == 65294 || e == 12290 || e == 65307 || e == 65306 || e == 65301 || e == 65311 || e == 12301 || e == 12303 || e == 12313 || e == 12311;
}
function At(e) {
  return e >= 33 && e <= 47 || e >= 58 && e <= 64 || e >= 91 && e <= 96 || e >= 123 && e <= 126 || e == 161 || e == 167 || e == 171 || e == 182 || e == 183 || e == 191 || e >= 8208 && e <= 8286 || e >= 12288 && e <= 12351 || e >= 65281 && e <= 65380;
}
function Hn() {
  return "“";
}
function he() {
  return "”";
}
function yn() {
  return "‘";
}
function nt() {
  return "’";
}
function $n(e) {
  let t = "", n = 0;
  const l = Number(e.length);
  for (; n < l; ) {
    const r = e.slice(n, n + 1);
    if (r == '"') {
      let u = !1;
      if (t == "")
        u = !0;
      else {
        const s = t.slice(Number(t.length) - 1, Number(t.length));
        s == " " && (u = !0), s == `
` && (u = !0), s == "(" && (u = !0), s == "[" && (u = !0), s == "{" && (u = !0);
      }
      if (u) {
        t += Hn(), n += 1;
        continue;
      }
      if (n + 1 >= Number(e.length)) {
        t += he(), n += 1;
        continue;
      }
      const i = e.charCodeAt(n + 1);
      i == 32 || i == 10 || Qn(i) ? t += he() : t += r, n += 1;
      continue;
    }
    if (r == "'") {
      let u = !1;
      if (n > 0 && n + 1 < Number(e.length)) {
        const i = e.charCodeAt(n - 1), s = e.charCodeAt(n + 1), f = fe(i), o = fe(s);
        f && o && (u = !0);
      }
      if (u)
        t += nt();
      else {
        let i = !1;
        if (t == "")
          i = !0;
        else {
          const s = t.slice(Number(t.length) - 1, Number(t.length));
          s == " " && (i = !0), s == `
` && (i = !0), s == "(" && (i = !0), s == "[" && (i = !0);
        }
        i ? t += yn() : t += nt();
      }
      n += 1;
      continue;
    }
    t += r, n += 1;
  }
  return t;
}
function Gn(e) {
  return e == 42 ? !0 : e == 95;
}
function J(e, t, n, l, r) {
  if (n == "_" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (fe(o))
      return null;
  }
  if (n == "__" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (fe(o))
      return null;
  }
  if (n == "___" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (fe(o))
      return null;
  }
  const u = t + Number(n.length), i = e.slice(u);
  let s = Ve(i, n), f = i;
  if (s != -1 && (f = i.slice(0, s)), f == "")
    return null;
  if (At(f.charCodeAt(0))) {
    let o = !1;
    if (s != -1 && Gn(f.charCodeAt(0)) && (o = !0), !o)
      return null;
  }
  if (s != -1) {
    let o = u + s + Number(n.length);
    return new Q(o, f);
  }
  return !l && r || i == "" || i.charCodeAt(0) == 32 ? null : new Q(Number(e.length), i);
}
function Kn(e, t, n) {
  let l = t;
  for (; l < Number(e.length); ) {
    if (e.charCodeAt(l) != 96) {
      l += 1;
      continue;
    }
    let r = 0;
    for (; D(e, "`", l + r); )
      r += 1;
    if (r == n)
      return l;
    l += r;
  }
  return -1;
}
function zn(e) {
  let t = Number(e.length);
  for (; t > 0; ) {
    const n = e.charCodeAt(t - 1);
    if (n == 46)
      t -= 1;
    else if (n == 44)
      t -= 1;
    else if (n == 58)
      t -= 1;
    else if (n == 59)
      t -= 1;
    else if (n == 33)
      t -= 1;
    else if (n == 63)
      t -= 1;
    else if (n == 41)
      t -= 1;
    else
      break;
  }
  return t == Number(e.length) ? e : e.slice(0, t);
}
function Yn(e) {
  return !!(M(e, "http://") && Number(e.length) > 7 || M(e, "https://") && Number(e.length) > 8);
}
function Xn(e) {
  if (Number(e.length) == 0)
    return !1;
  let t = -1, n = 0;
  for (; n < Number(e.length); ) {
    const u = e.charCodeAt(n);
    let i = !1;
    if (u >= 48 && u <= 57 && (i = !0), u >= 65 && u <= 90 && (i = !0), u >= 97 && u <= 122 && (i = !0), u == 45 && (i = !0), u == 46 && (i = !0, t = n), !i)
      return !1;
    n += 1;
  }
  if (t <= 0)
    return !1;
  let l = 0, r = t + 1;
  for (; r < Number(e.length); ) {
    const u = e.charCodeAt(r);
    if (u >= 65)
      if (u <= 90)
        l += 1;
      else
        return !1;
    else if (u >= 97)
      if (u <= 122)
        l += 1;
      else
        return !1;
    else
      return !1;
    r += 1;
  }
  return !(l < 2);
}
function lt(e, t, n, l) {
  let r = Ne(e, "]", t);
  if (r == -1)
    return null;
  const u = e.slice(t + 1, r);
  let i = r + 1;
  if (!D(e, "(", i))
    return null;
  let s = Ne(e, ")", i);
  if (s == -1) {
    const m = e.slice(i + 1);
    if (Yn(m)) {
      let a = zn(m), h = m.slice(Number(a.length)), b = "";
      return l && (b = null), new te(Number(e.length), u, a, !0, b, h);
    }
    return Xn(m) ? new te(Number(e.length), u, "http://" + m, !0, null, "") : new te(Number(e.length), u, "", !0, null, "");
  }
  let f = e.slice(i + 1, s), o = f, c = null;
  const g = Ve(f, ' "');
  if (g != -1) {
    o = f.slice(0, g);
    const m = f.slice(g + 2);
    y(m, '"') && (c = m.slice(0, Number(m.length) - 1));
  } else
    s + 1 >= Number(e.length) || l || (c = "");
  return new te(s + 1, u, o, !1, c, "");
}
function _t(e) {
  const n = e.trim().split(" "), l = Number(n.length);
  if (l < 2)
    return "";
  const r = n[l - 1];
  if (Number(r.length) < 2 || r.slice(0, 1) != "^")
    return "";
  const u = r.slice(1), i = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-";
  for (let s = 0; s < Number(u.length); s++) {
    const f = u.slice(s, s + 1);
    let o = !1;
    for (let c = 0; c < Number(i.length); c++)
      i.slice(c, c + 1) == f && (o = !0);
    if (!o)
      return "";
  }
  return u;
}
function K(e, t) {
  let n = [];
  for (const l of e) {
    const r = l.type;
    if (r == "text" && n.push(G(l.content ?? "", t, [])), r == "strong")
      for (const u of K(l.children ?? B(), X(t, O.Strong)))
        n.push(u);
    if (r == "emphasis")
      for (const u of K(l.children ?? B(), X(t, O.Em)))
        n.push(u);
    if (r == "underline")
      for (const u of K(l.children ?? B(), X(t, O.Underline)))
        n.push(u);
    if (r == "strikethrough")
      for (const u of K(l.children ?? B(), X(t, O.Del)))
        n.push(u);
    if (r == "inline_code" && n.push(G(l.code ?? "", X(t, O.Code), [])), r == "hardbreak" && n.push(G(`
`, t, [])), r == "link") {
      let u = [];
      u = S(u, "href", k.Str(l.href ?? ""));
      const i = l.title;
      i != null && (u = S(u, "title", k.Str(i ?? "")));
      for (const s of K(l.children ?? B(), X(t, O.Link)))
        n.push(G(s.text, s.marks, u));
    }
    if (r == "image") {
      let u = [];
      u = S(u, "src", k.Str(l.src ?? "")), u = S(u, "alt", k.Str(l.alt ?? ""));
      const i = l.title;
      i != null && (u = S(u, "title", k.Str(i ?? ""))), n.push(G(l.alt ?? "", X(t, O.Image), u));
    }
    if (r == "wikilink") {
      let u = [];
      u = S(u, "wikilink", k.Str(l.title ?? "")), n.push(G(l.title ?? "", t, u));
    }
    if (r == "math_inline") {
      let u = [];
      u = S(u, "math_inline", k.Str(l.code ?? "")), n.push(G(l.code ?? "", t, u));
    }
  }
  return n;
}
function Jn(e, t) {
  let n = [];
  return n = S(n, "header", k.Bool(e.isHeader ?? !1)), n = S(n, "align", k.Str(e.align ?? "left")), I(t, p.TableCell, n, [], K(e.children ?? B(), []), v(0, 0));
}
function rt(e, t) {
  const n = e.cells ?? B();
  let l = [];
  for (let r = 0; r < Number(n.length); r++)
    l.push(Jn(n[r], t + "-c" + String(r)));
  return I(t, p.TableRow, [], l, [], v(0, 0));
}
function St(e, t) {
  const n = e.type;
  if (n == "heading") {
    let l = [];
    return l = S(l, "level", k.Int(e.level ?? 0)), I(t, p.Heading, l, [], K(e.children ?? B(), []), v(0, 0));
  }
  if (n == "code_block") {
    let l = [];
    return l = S(l, "language", k.Str(e.language ?? "")), l = S(l, "loading", k.Bool(e.loading ?? !1)), I(t, p.Fence, l, [], [Z(e.code ?? "")], v(0, 0));
  }
  if (n == "blockquote")
    return I(t, p.Blockquote, [], ie(e.children ?? B(), t), [], v(0, 0));
  if (n == "list") {
    let l = [];
    l = S(l, "ordered", k.Bool(e.ordered ?? !1));
    const r = e.start;
    return r != null && (l = S(l, "start", k.Int(r ?? 0))), I(t, p.ListBlock, l, ie(e.items ?? B(), t), [], v(0, 0));
  }
  if (n == "list_item") {
    let l = [];
    const r = e.checked;
    return r != null && (l = S(l, "checked", k.Bool(r ?? !1))), I(t, p.ListItem, l, ie(e.children ?? B(), t), [], v(0, 0));
  }
  if (n == "table") {
    let l = [];
    const r = e.header ?? B();
    Number(r.length) > 0 && l.push(rt(r[0], t + "-h"));
    const u = e.rows ?? B();
    for (let s = 0; s < Number(u.length); s++)
      l.push(rt(u[s], t + "-r" + String(s)));
    let i = [];
    return i = S(i, "loading", k.Bool(e.loading ?? !1)), I(t, p.Table, i, l, [], v(0, 0));
  }
  if (n == "thematic_break")
    return I(t, p.ThematicBreak, [], [], [], v(0, 0));
  if (n == "callout") {
    let l = [];
    return l = S(l, "type", k.Str(e.language ?? "")), l = S(l, "title", k.Str(e.title ?? "")), I(t, p.Callout, l, ie(e.children ?? B(), t), [], v(0, 0));
  }
  if (n == "details") {
    let l = [];
    return l = S(l, "summary", k.Str(e.text ?? "")), (e.loading ?? !1) && (l = S(l, "open", k.Bool(!0))), I(t, p.Details, l, ie(e.children ?? B(), t), [], v(0, 0));
  }
  if (n == "query") {
    let l = [];
    return l = S(l, "query", k.Str(e.content ?? "")), I(t, p.QueryBlock, l, [], [], v(0, 0));
  }
  if (n == "embed") {
    let l = [];
    return l = S(l, "src", k.Str(e.src ?? "")), I(t, p.BlockEmbed, l, [], [], v(0, 0));
  }
  return n == "math_block" ? I(t, p.MathBlock, [], [], [Z(e.code ?? "")], v(0, 0)) : n == "mermaid" ? I(t, p.Mermaid, [], [], [Z(e.code ?? "")], v(0, 0)) : I(t, p.Paragraph, [], [], K(e.children ?? B(), []), v(0, 0));
}
function ie(e, t) {
  let n = [];
  for (let l = 0; l < Number(e.length); l++)
    n.push(St(e[l], t + "-" + String(l)));
  return n;
}
function ut(e) {
  let t = [], n = 0;
  for (; n < Number(e.length); ) {
    const l = e[n];
    l == null ? t.push(k.Null()) : t.push(k.Int(l ?? 0)), n += 1;
  }
  return k.ListV(t);
}
function Zn(e, t) {
  let n = [], l = 0;
  for (const r of e)
    if (r.kind == p.Table)
      if (l < Number(t.length)) {
        const u = t[l], i = ut(u.cols), s = ut(u.rows);
        let f = [];
        f.push(Qe("cols", i)), f.push(Qe("rows", s)), n.push(I(r.id, r.kind, S(r.attrs, "ial", k.AttrsV(f)), r.children, r.inlines, r.source)), l += 1;
      } else
        n.push(r);
    else
      n.push(r);
  return n;
}
function y(e, t) {
  const n = Number(e.length), l = Number(t.length);
  return l > n ? !1 : e.slice(n - l, n) == t;
}
function Fn(e, t) {
  let n = [], l = 0;
  for (let r = 0; r < Number(e.length); r++) {
    const u = e[r].text, i = l, s = l + Number(u.length);
    if (l = s, !(i >= t)) if (s <= t)
      n.push(e[r]);
    else {
      const f = t - i;
      f > 0 && n.push(G(u.slice(0, f), e[r].marks, e[r].attrs));
    }
  }
  return n;
}
function el(e) {
  return e == p.Paragraph || e == p.Heading || e == p.ListItem;
}
function vt(e) {
  let t = [];
  for (let l = 0; l < Number(e.children.length); l++)
    t.push(vt(e.children[l]));
  let n = I(e.id, e.kind, e.attrs, t, e.inlines, e.source);
  if (el(e.kind) && Number(e.inlines.length) > 0) {
    const l = oe(e.inlines), r = _t(l);
    if (r != "") {
      const u = Number(l.length), i = Number(r.length) + 1, s = u - i - 1;
      if (s >= 0) {
        const f = l.slice(s, s + 1);
        if (f == " " || f == "	") {
          const o = Fn(e.inlines, s), c = I(e.id, e.kind, e.attrs, t, o, e.source);
          n = Dt(c, r);
        }
      }
    }
  }
  return n;
}
function tl(e) {
  const t = Number(e.length);
  if (t < 3)
    return e;
  const n = _t(e);
  if (n == "")
    return e;
  const l = "^" + n, r = Number(l.length);
  if (!y(e, l))
    return e;
  const u = t - r;
  if (u <= 0)
    return e;
  const i = e.slice(u - 1, u);
  return i != " " && i != "	" ? e : e.slice(0, u - 1);
}
function Bl(e) {
  let t = "", n = !1;
  const l = e.split(`
`), r = Number(l.length);
  for (let u = 0; u < r; u++) {
    let i = l[u];
    i.slice(0, 3) == "```" ? n = !n : n || (i = tl(i)), t = t + i, u < r - 1 && (t = t + `
`);
  }
  return t;
}
function Tl(e, t) {
  const n = Xt(e), l = n.md, r = kn(l, t);
  let u = [];
  for (let s = 0; s < Number(r.length); s++) {
    const f = "block-" + String(s);
    u.push(vt(St(r[s], f)));
  }
  const i = Zn(u, n.tableAttrs);
  return I("doc", p.Paragraph, [], i, [], v(0, 0));
}
function Lt(e, t) {
  let n = "";
  for (let l = 0; l < t; l++)
    n = n + e;
  return n;
}
function xt(e, t) {
  const n = Number(e.length), l = Number(t.length);
  return n < l ? !1 : e.slice(n - l, n) == t;
}
function nl(e) {
  for (let t = 0; t < Number(e.length); t++)
    if (e.slice(t, t + 1) == `
`)
      return !0;
  return !1;
}
function ll(e) {
  if (e.text == `
`)
    return `  
`;
  let t = e.text;
  const n = T(e.attrs, "wikilink", "");
  Number(n.length) > 0 && (t = "[[" + n + "]]");
  const l = T(e.attrs, "math_inline", "");
  if (Number(l.length) > 0 && (t = "$" + l + "$"), Y(e.marks, O.Code) && (t = "`" + t + "`"), Y(e.marks, O.Strong) && (t = "**" + t + "**"), Y(e.marks, O.Em) && (t = "*" + t + "*"), Y(e.marks, O.Underline) && (t = "__" + t + "__"), Y(e.marks, O.Del) && (t = "~~" + t + "~~"), Y(e.marks, O.Link)) {
    const r = T(e.attrs, "href", ""), u = T(e.attrs, "title", "");
    Number(u.length) > 0 ? t = "[" + t + "](" + r + ' "' + u + '")' : t = "[" + t + "](" + r + ")";
  }
  if (Y(e.marks, O.Image)) {
    const r = T(e.attrs, "src", ""), u = T(e.attrs, "title", "");
    Number(u.length) > 0 ? t = "![" + t + "](" + r + ' "' + u + '")' : t = "![" + t + "](" + r + ")";
  }
  return t;
}
function Se(e) {
  let t = "";
  for (const n of e)
    t = t + ll(n);
  return t;
}
function rl(e) {
  return e == "left" ? ":---" : e == "center" ? ":---:" : e == "right" ? "---:" : "---";
}
function qe(e) {
  let t = "|";
  for (const n of e.children)
    t = t + " " + Se(n.inlines) + " |";
  return t;
}
function ul(e) {
  let t = "|";
  for (const n of e.children)
    t = t + " " + rl(T(n.attrs, "align", "")) + " |";
  return t;
}
function il(e) {
  const t = e;
  return t._tag === "Null" ? [] : t._tag === "Str" ? (t.value, []) : t._tag === "Int" ? (t.value, []) : t._tag === "Bool" ? (t.value, []) : t._tag === "ListV" ? t.value : t._tag === "AttrsV" ? (t.value, []) : [];
}
function sl(e) {
  const t = e;
  return t._tag === "Null" ? [] : t._tag === "Str" ? (t.value, []) : t._tag === "Int" ? (t.value, []) : t._tag === "Bool" ? (t.value, []) : t._tag === "ListV" ? (t.value, []) : t._tag === "AttrsV" ? t.value : [];
}
function fl(e) {
  const t = e;
  return t._tag === "Null" ? null : t._tag === "Str" ? (t.value, null) : t._tag === "Int" ? t.value : t._tag === "Bool" || t._tag === "ListV" ? (t.value, null) : (t._tag === "AttrsV" && t.value, null);
}
function it(e) {
  const t = il(e);
  let n = [];
  for (let l = 0; l < Number(t.length); l++)
    n.push(fl(t[l]));
  return n;
}
function st(e) {
  for (let t = 0; t < Number(e.length); t++)
    if (e[t] != null)
      return !0;
  return !1;
}
function ft(e) {
  let t = "";
  for (let n = 0; n < Number(e.length); n++) {
    n > 0 && (t = t + ",");
    const l = e[n];
    l == null ? t = t + '"auto"' : t = t + String(l ?? 0);
  }
  return t;
}
function ol(e) {
  const t = ee(e, "cols"), n = ee(e, "rows"), l = it(t ?? k.Null()), r = it(n ?? k.Null());
  let u = [];
  st(l) && u.push("cols:[" + ft(l) + "]"), st(r) && u.push("rows:[" + ft(r) + "]");
  let i = "";
  for (let s = 0; s < Number(u.length); s++)
    s > 0 && (i = i + ", "), i = i + u[s];
  return i;
}
function cl(e) {
  if (Number(e.children.length) == 0)
    return "";
  let t = "";
  for (let l = 0; l < Number(e.children.length); l++) {
    l > 0 && (t = t + `
`);
    const r = e.children[l];
    l == 0 ? t = t + qe(r) + `
` + ul(r) : t = t + qe(r);
  }
  const n = ee(e.attrs, "ial");
  if (n != null) {
    const l = ol(sl(n ?? k.Null()));
    Number(l.length) > 0 && (t = t + `
{` + l + "}");
  }
  return t;
}
function ve(e, t) {
  let n = "";
  for (let l = 0; l < Number(e.length); l++)
    l > 0 && (e[l].kind == p.ListBlock ? n = n + `
` : n = n + `

`), n = n + Bt(e[l], t);
  return n;
}
function al(e, t) {
  const l = ve(e.children, t).split(`
`);
  let r = "";
  for (let u = 0; u < Number(l.length); u++)
    u > 0 && (r = r + `
`), Number(l[u].length) > 0 ? r = r + "> " + l[u] : r = r + ">";
  return r;
}
function hl(e, t) {
  const n = Te(e.attrs, "ordered", !1), l = ot(e.attrs, "start", 1);
  let r = "";
  for (let u = 0; u < Number(e.children.length); u++) {
    u > 0 && (r = r + `
`);
    let i = "- ", s = 2;
    if (n) {
      const g = l + u;
      i = String(g) + ". ", s = Number(i.length);
    } else {
      const g = e.children[u];
      ee(g.attrs, "checked") != null && (Te(g.attrs, "checked", !1) ? i = "- [x] " : i = "- [ ] ");
    }
    const o = ve(e.children[u].children, t).split(`
`), c = Lt(" ", s);
    for (let g = 0; g < Number(o.length); g++)
      g > 0 && (r = r + `
`), g == 0 ? r = r + i + o[g] : Number(o[g].length) > 0 && (r = r + c + o[g]);
  }
  return r;
}
function gl(e) {
  const t = T(e.attrs, "language", ""), n = oe(e.inlines);
  let l = "```" + t + `
` + n;
  return Number(n.length) > 0 && (xt(n, `
`) || (l = l + `
`)), l = l + "```", l;
}
function ml(e, t) {
  const n = ot(e.attrs, "level", 1);
  let l = Se(e.inlines);
  if (t && (l = Tt(l, T(e.attrs, "anchor", ""))), n <= 2 && nl(l)) {
    let r = "---";
    return n == 1 && (r = "==="), l + `
` + r;
  }
  return Lt("#", n) + " " + l;
}
function Ee(e, t) {
  return e + ': "' + t + '"';
}
function It(e, t, n, l) {
  return "$" + e + "(" + t + `) {
` + ve(n.children, l) + `
}`;
}
function bl(e, t) {
  let n = Ee("type", T(e.attrs, "type", ""));
  const l = T(e.attrs, "title", "");
  return Number(l.length) > 0 && (n = n + ", " + Ee("title", l)), It("callout", n, e, t);
}
function dl(e, t) {
  let n = Ee("summary", T(e.attrs, "summary", ""));
  return Te(e.attrs, "open", !1) && (n = n + ", open: true"), It("details", n, e, t);
}
function pl(e) {
  const t = T(e.attrs, "target", ""), n = T(e.attrs, "anchor", "");
  return Number(n.length) > 0 ? "[[" + t + "#" + n + "]]" : "[[" + t + "]]";
}
function Bt(e, t) {
  const n = e.kind;
  if (n == p.Heading)
    return ml(e, t);
  if (n == p.Fence)
    return gl(e);
  if (n == p.Blockquote)
    return al(e, t);
  if (n == p.ListBlock)
    return hl(e, t);
  if (n == p.ListItem)
    return ve(e.children, t);
  if (n == p.Table)
    return cl(e);
  if (n == p.TableRow)
    return qe(e);
  if (n == p.TableCell)
    return Se(e.inlines);
  if (n == p.ThematicBreak)
    return "---";
  if (n == p.Callout)
    return bl(e, t);
  if (n == p.Details)
    return dl(e, t);
  if (n == p.WikilinkBlock)
    return pl(e);
  if (n == p.QueryBlock)
    return "$query(" + T(e.attrs, "query", "") + ")";
  if (n == p.BlockEmbed)
    return '$embed(src: "' + T(e.attrs, "src", "") + '")';
  if (n == p.Mermaid)
    return "```mermaid\n" + oe(e.inlines) + "\n```";
  if (n == p.MathBlock)
    return `%{
` + oe(e.inlines) + `
}%`;
  const l = Se(e.inlines);
  return t ? Tt(l, T(e.attrs, "anchor", "")) : l;
}
function Tt(e, t) {
  const n = "^" + t;
  return Number(t.length) == 0 || xt(e, n) ? e : e + " " + n;
}
function Nl(e, t) {
  let n = "";
  for (let l = 0; l < Number(e.length); l++) {
    l > 0 && (n = n + `

`);
    const r = e[l];
    n = n + Bt(r, t);
  }
  return n;
}
function Rl(e, t) {
  const n = Nl(e.children, t);
  return Number(n.length) == 0 ? "" : n + `
`;
}
export {
  wl as $,
  ln as A,
  p as B,
  Fe as C,
  Ze as D,
  mn as E,
  tn as F,
  Je as G,
  ce as H,
  Xe as I,
  Al as J,
  $e as K,
  vl as L,
  O as M,
  ct as N,
  xl as O,
  Et as P,
  S as Q,
  Ll as R,
  Ut as S,
  de as T,
  qt as U,
  k as V,
  A as W,
  le as X,
  X as Y,
  kl as Z,
  Ot as _,
  Z as a,
  re as a0,
  me as a1,
  be as a2,
  Qe as a3,
  Be as a4,
  _l as a5,
  Sl as a6,
  Tl as a7,
  Se as a8,
  Cl as a9,
  jt as aa,
  Il as ab,
  Rl as ac,
  Nl as ad,
  Le as b,
  Ft as c,
  T as d,
  oe as e,
  W as f,
  dt as g,
  Te as h,
  ee as i,
  en as j,
  ot as k,
  xe as l,
  ze as m,
  Ye as n,
  Re as o,
  kn as p,
  Zt as q,
  $ as r,
  Bl as s,
  Jt as t,
  Y as u,
  Pe as v,
  He as w,
  nn as x,
  rn as y,
  un as z
};
