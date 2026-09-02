class bt {
  constructor(t, n) {
    this.start = t, this.end = n;
  }
}
function S(e, t) {
  return new bt(e, t);
}
const w = {
  Null: () => ({ _tag: "Null" }),
  Str: (e) => ({ _tag: "Str", value: e }),
  Int: (e) => ({ _tag: "Int", value: e }),
  Bool: (e) => ({ _tag: "Bool", value: e }),
  ListV: (e) => ({ _tag: "ListV", value: e }),
  AttrsV: (e) => ({ _tag: "AttrsV", value: e })
};
class _e {
  constructor(t, n) {
    this.key = t, this.value = n;
  }
}
function ve(e, t) {
  for (let n = 0; n < Number(e.length); n++)
    if (e[n].key == t)
      return e[n].value;
  return null;
}
function gt(e, t, n) {
  const u = ve(e, t) ?? w.Str(n);
  return u._tag === "Null" ? n : u._tag === "Str" ? u.value : u._tag === "Int" || u._tag === "Bool" || u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function Vn(e, t, n) {
  const u = ve(e, t) ?? w.Int(n);
  return u._tag === "Null" ? n : u._tag === "Str" ? (u.value, n) : u._tag === "Int" ? u.value : u._tag === "Bool" || u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function Wn(e, t, n) {
  const u = ve(e, t) ?? w.Bool(n);
  return u._tag === "Null" ? n : u._tag === "Str" || u._tag === "Int" ? (u.value, n) : u._tag === "Bool" ? u.value : u._tag === "ListV" ? (u.value, n) : (u._tag === "AttrsV" && u.value, n);
}
function _(e, t, n) {
  let l = [], r = -1;
  for (let u = 0; u < Number(e.length); u++)
    e[u].key == t && (r = u), l.push(e[u]);
  return r >= 0 ? l[r] = new _e(t, n) : l.push(new _e(t, n)), l;
}
function ie(e) {
  let t = [];
  for (const n of e)
    t.push(n);
  return t;
}
var G = /* @__PURE__ */ ((e) => (e[e.Strong = 0] = "Strong", e[e.Em = 1] = "Em", e[e.Code = 2] = "Code", e[e.Link = 3] = "Link", e[e.Image = 4] = "Image", e[e.Del = 5] = "Del", e[e.Underline = 6] = "Underline", e))(G || {});
function Qn(e, t) {
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
class ee {
  constructor(t, n, l) {
    this.text = t, this.marks = n, this.attrs = l;
  }
}
function F(e) {
  return new ee(e, [], []);
}
function K(e, t, n) {
  return new ee(e, t, n);
}
function ye(e) {
  let t = "";
  for (const n of e)
    t = t + n.text;
  return t;
}
function Oe(e, t, n) {
  let l = [], r = 0, u = !1;
  for (const i of e)
    if (u)
      l.push(i);
    else {
      const s = Number(i.text.length);
      if (t <= r + s) {
        const f = t - r, o = i.text.slice(0, f) + n + i.text.slice(f);
        l.push(new ee(o, i.marks, i.attrs)), u = !0;
      } else
        l.push(i);
      r = r + s;
    }
  return u || l.push(F(n)), l;
}
function pt(e, t, n) {
  let l = [], r = 0;
  for (const u of e) {
    const i = Number(u.text.length), s = r + i;
    let f = !0;
    if (s <= t && (f = !1), r >= n && (f = !1), f) {
      let o = "";
      t > r && (o = u.text.slice(0, t - r)), n < s && (o = o + u.text.slice(n - r)), Number(o.length) > 0 && l.push(new ee(o, u.marks, u.attrs));
    } else
      l.push(u);
    r = s;
  }
  return l;
}
class Nt {
  constructor(t, n) {
    this.before = t, this.after = n;
  }
}
function wt(e, t) {
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
        Number(o.length) > 0 && n.push(new ee(o, i.marks, i.attrs)), Number(c.length) > 0 && l.push(new ee(c, i.marks, i.attrs)), u = !0;
      }
      r = f;
    }
  return new Nt(n, l);
}
var x = /* @__PURE__ */ ((e) => (e[e.Heading = 0] = "Heading", e[e.Paragraph = 1] = "Paragraph", e[e.Fence = 2] = "Fence", e[e.Blockquote = 3] = "Blockquote", e[e.ListBlock = 4] = "ListBlock", e[e.ListItem = 5] = "ListItem", e[e.Table = 6] = "Table", e[e.TableRow = 7] = "TableRow", e[e.TableCell = 8] = "TableCell", e[e.ThematicBreak = 9] = "ThematicBreak", e[e.Callout = 10] = "Callout", e[e.Details = 11] = "Details", e[e.WikilinkBlock = 12] = "WikilinkBlock", e[e.QueryBlock = 13] = "QueryBlock", e[e.BlockEmbed = 14] = "BlockEmbed", e[e.Mermaid = 15] = "Mermaid", e[e.MathBlock = 16] = "MathBlock", e))(x || {});
class R {
  constructor(t, n, l, r, u, i) {
    this.id = t, this.kind = n, this.attrs = l, this.children = r, this.inlines = u, this.source = i;
  }
}
function kt(e, t) {
  return new R(e, t, [], [], [], S(0, 0));
}
function v(e, t, n, l, r, u) {
  return new R(e, t, n, l, r, u);
}
function Ee(e, t) {
  return new _e(e, t);
}
function Hn(e, t, n) {
  return new R(e, t, [], [], [F(n)], S(0, Number(n.length)));
}
function ke(e) {
  return ye(e.inlines);
}
function je(e, t) {
  return new R(e.id, e.kind, e.attrs, e.children, t, e.source);
}
function Ct(e, t) {
  return new R(e.id, t, e.attrs, e.children, e.inlines, e.source);
}
function he(e, t) {
  return new R(e.id, e.kind, e.attrs, t, e.inlines, e.source);
}
function qe(e) {
  let t = [];
  for (const n of e)
    t.push(n);
  return t;
}
function Mn(e) {
  return gt(e.attrs, "anchor", "");
}
function At(e, t) {
  return new R(t, e.kind, _(e.attrs, "anchor", w.Str(t)), e.children, e.inlines, e.source);
}
function _t(e, t) {
  return new R(t, e.kind, _(e.attrs, "anchor", w.Str(t)), e.children, e.inlines, e.source);
}
function Fe(e, t) {
  if (e.id == t)
    return !0;
  for (let n = 0; n < Number(e.children.length); n++)
    if (Fe(e.children[n], t))
      return !0;
  return !1;
}
function St(e, t, n) {
  if (!Fe(e, t))
    return e;
  if (e.id == t)
    return At(e, n);
  if (Number(e.children.length) == 0)
    return e;
  let l = [];
  for (let r = 0; r < Number(e.children.length); r++)
    l.push(St(e.children[r], t, n));
  return new R(e.id, e.kind, e.attrs, l, e.inlines, e.source);
}
function V(e, t) {
  if (e.id == t)
    return e;
  for (let n = 0; n < Number(e.children.length); n++) {
    const l = V(e.children[n], t);
    if (l != null)
      return l;
  }
  return null;
}
function ce(e, t) {
  for (let n = 0; n < Number(e.children.length); n++) {
    if (e.children[n].id == t)
      return e;
    const l = ce(e.children[n], t);
    if (l != null)
      return l;
  }
  return null;
}
function ae(e, t) {
  for (let n = 0; n < Number(e.children.length); n++)
    if (e.children[n].id == t)
      return n;
  return -1;
}
function et(e, t, n) {
  const l = ae(e, t);
  if (l >= 0) {
    let u = [];
    for (let i = 0; i < Number(e.children.length); i++)
      if (i == l)
        for (const s of n)
          u.push(s);
      else
        u.push(e.children[i]);
    return he(e, u);
  }
  let r = [];
  for (const u of e.children)
    r.push(et(u, t, n));
  return he(e, r);
}
function M(e, t, n) {
  return e.id == t ? Number(n.length) > 0 ? n[0] : e : et(e, t, n);
}
function tt(e, t, n, l, r) {
  if (e.id == t) {
    let i = [];
    for (let s = 0; s < Number(e.children.length); s++) {
      if (s < n && i.push(e.children[s]), s == n)
        for (const f of r)
          i.push(f);
      s >= l && i.push(e.children[s]);
    }
    return he(e, i);
  }
  let u = [];
  for (const i of e.children)
    u.push(tt(i, t, n, l, r));
  return he(e, u);
}
class De {
  constructor(t, n) {
    this.blockId = t, this.offset = n;
  }
}
class xt {
  constructor(t, n) {
    this.anchor = t, this.head = n;
  }
}
function te(e, t) {
  return new xt(new De(e, t), new De(e, t));
}
class Gn {
  constructor(t, n) {
    this.pos = t, this.text = n;
  }
}
class Kn {
  constructor(t, n) {
    this.pos = t, this.newId = n;
  }
}
class Yn {
  constructor(t, n) {
    this.aId = t, this.bId = n;
  }
}
class zn {
  constructor(t, n) {
    this.id = t, this.kind = n;
  }
}
class Xn {
  constructor(t, n) {
    this.sel = t, this.text = n;
  }
}
const $n = {
  InsertText: (e) => ({ _tag: "InsertText", value: e }),
  SplitBlock: (e) => ({ _tag: "SplitBlock", value: e }),
  MergeBlocks: (e) => ({ _tag: "MergeBlocks", value: e }),
  SetBlockType: (e) => ({ _tag: "SetBlockType", value: e }),
  LiftBlock: (e) => ({ _tag: "LiftBlock", value: e }),
  WrapBlock: (e) => ({ _tag: "WrapBlock", value: e }),
  ReplaceRange: (e) => ({ _tag: "ReplaceRange", value: e })
};
class A {
  constructor(t, n) {
    this.tree = t, this.selection = n;
  }
}
function j() {
  return kt(
    "",
    1
    /* Paragraph */
  );
}
function Jn(e, t, n) {
  const l = n;
  if (l._tag === "InsertText") {
    const r = l.value, i = V(e, r.pos.blockId) ?? j();
    if (i.id == "")
      return new A(e, t);
    const s = Oe(i.inlines, r.pos.offset, r.text), f = M(e, i.id, [je(i, s)]);
    return new A(f, te(r.pos.blockId, r.pos.offset + Number(r.text.length)));
  } else if (l._tag === "SplitBlock") {
    const r = l.value, i = V(e, r.pos.blockId) ?? j();
    if (i.id == "")
      return new A(e, t);
    const s = wt(i.inlines, r.pos.offset), f = new R(i.id, i.kind, ie(i.attrs), qe(i.children), s.before, S(i.source.start, i.source.start + r.pos.offset)), o = new R(r.newId, i.kind, ie(i.attrs), qe(i.children), s.after, S(i.source.start + r.pos.offset, i.source.end)), c = M(e, i.id, [f, o]);
    return new A(c, te(r.newId, 0));
  } else if (l._tag === "MergeBlocks") {
    const r = l.value, i = V(e, r.aId) ?? j(), f = V(e, r.bId) ?? j();
    if (i.id == "")
      return new A(e, t);
    if (f.id == "")
      return new A(e, t);
    const o = Number(ke(i).length);
    let c = [];
    for (const b of i.inlines)
      c.push(b);
    for (const b of f.inlines)
      c.push(b);
    let m = [];
    for (const b of i.children)
      m.push(b);
    for (const b of f.children)
      m.push(b);
    const d = new R(i.id, i.kind, i.attrs, m, c, S(i.source.start, f.source.end)), a = M(e, r.bId, []), h = M(a, r.aId, [d]);
    return new A(h, te(r.aId, o));
  } else if (l._tag === "SetBlockType") {
    const r = l.value, i = V(e, r.id) ?? j();
    if (i.id == "")
      return new A(e, t);
    const s = M(e, r.id, [Ct(i, r.kind)]);
    return new A(s, t);
  } else if (l._tag === "LiftBlock") {
    const r = l.value, i = V(e, r.id) ?? j();
    if (i.id == "")
      return new A(e, t);
    const f = ce(e, r.id) ?? j();
    if (f.id == "")
      return new A(e, t);
    if (f.id == e.id)
      return new A(e, t);
    const o = ae(f, r.id);
    let c = [], m = [];
    for (let h = 0; h < Number(f.children.length); h++)
      h < o && c.push(f.children[h]), h > o && m.push(f.children[h]);
    let d = [];
    Number(c.length) > 0 && d.push(new R(f.id, f.kind, ie(f.attrs), c, f.inlines, f.source)), d.push(i), Number(m.length) > 0 && d.push(new R(f.id + "-l", f.kind, ie(f.attrs), m, f.inlines, f.source));
    const a = M(e, f.id, d);
    return new A(a, t);
  } else if (l._tag === "WrapBlock") {
    const r = l.value, i = V(e, r.id) ?? j();
    if (i.id == "")
      return new A(e, t);
    const s = new R(r.newId, r.kind, [], [i], [], S(i.source.start, i.source.end)), f = M(e, r.id, [s]);
    return new A(f, t);
  } else if (l._tag === "ReplaceRange") {
    const r = l.value, u = r.sel;
    if (u.anchor.blockId == u.head.blockId) {
      const L = V(e, u.anchor.blockId) ?? j();
      if (L.id == "")
        return new A(e, t);
      let g = u.anchor.offset, N = u.head.offset;
      if (g > N) {
        const z = g;
        g = N, N = z;
      }
      const U = Oe(pt(L.inlines, g, N), g, r.text), O = M(e, L.id, [je(L, U)]);
      return new A(O, te(u.anchor.blockId, g + Number(r.text.length)));
    }
    const s = ce(e, u.anchor.blockId) ?? j(), o = ce(e, u.head.blockId) ?? j();
    if (s.id == "")
      return new A(e, t);
    if (o.id == "")
      return new A(e, t);
    if (s.id != o.id)
      return new A(e, t);
    const c = ae(s, u.anchor.blockId), m = ae(s, u.head.blockId);
    if (c < 0)
      return new A(e, t);
    if (m < 0)
      return new A(e, t);
    if (c > m)
      return new A(e, t);
    const d = s.children[c], a = s.children[m], h = ke(d).slice(0, u.anchor.offset) + r.text + ke(a).slice(u.head.offset);
    let b = [];
    for (const I of d.children)
      b.push(I);
    for (const I of a.children)
      b.push(I);
    const p = new R(d.id, d.kind, d.attrs, b, [F(h)], S(d.source.start, a.source.end)), k = tt(e, s.id, c, m + 1, [p]);
    return new A(k, te(u.anchor.blockId, u.anchor.offset + Number(r.text.length)));
  }
  return new A(e, t);
}
class nt {
  constructor(t, n) {
    this.cols = t, this.rows = n;
  }
}
class It {
  constructor(t, n) {
    this.md = t, this.tableAttrs = n;
  }
}
function T(e, t) {
  return Number(e.length) < Number(t.length) ? !1 : e.slice(0, Number(t.length)) == t;
}
function q(e, t, n) {
  return n < 0 || n + Number(t.length) > Number(e.length) ? !1 : e.slice(n, n + Number(t.length)) == t;
}
function Lt(e, t) {
  return Number(e.length) < Number(t.length) ? !1 : e.slice(Number(e.length) - Number(t.length), Number(e.length)) == t;
}
function re(e) {
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
function vt(e) {
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
function de(e, t) {
  let n = 0;
  for (; n < Number(e.length); ) {
    if (e.charCodeAt(n) == t)
      return !0;
    n += 1;
  }
  return !1;
}
function Be(e, t) {
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
function me(e, t, n) {
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
function lt(e, t) {
  let n = Number(e.length) - 1;
  for (; n >= 0; ) {
    if (e.charCodeAt(n) == t)
      return n;
    n -= 1;
  }
  return -1;
}
function rt(e) {
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
function Bt(e) {
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
function Pt(e) {
  const t = Bt(e);
  return t == "auto" ? null : rt(t);
}
function Ue(e) {
  const t = e.split(",");
  let n = [], l = 0;
  for (; l < Number(t.length); ) {
    const r = t[l];
    n.push(Pt(r)), l += 1;
  }
  return n;
}
function Ve(e) {
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
function Rt(e) {
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
function Tt(e) {
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
  if (!T(n, "{cols:[") || !Lt(n, "}"))
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
  const i = Ue(n.slice(7, r));
  let s = [];
  const f = n.slice(r + 1, l);
  if (f != "") {
    if (!T(f, ","))
      return null;
    let o = f.slice(1), c = !0;
    for (; c; )
      if (c = !1, Number(o.length) > 0) {
        const h = o.charCodeAt(0);
        (h == 32 || h == 9 || h == 10 || h == 13) && (o = o.slice(1), c = !0);
      }
    if (!T(o, "rows:["))
      return null;
    const m = o.slice(6);
    let d = -1, a = 0;
    for (; a < Number(m.length); ) {
      if (m.charCodeAt(a) == 93) {
        d = a;
        break;
      }
      a += 1;
    }
    if (d == -1 || d + 1 != Number(m.length))
      return null;
    s = Ue(m.slice(0, d));
  }
  return new nt(i, s);
}
function Ot(e) {
  const t = e.split(`
`);
  let n = [], l = [], r = 0;
  for (; r < Number(t.length); ) {
    let u = !1;
    if (r + 1 < Number(t.length) && Ve(t[r]) && Rt(t[r + 1])) {
      let i = r + 2, s = 0, f = !0;
      for (; f; )
        f = !1, i < Number(t.length) && Ve(t[i]) && (i += 1, s += 1, f = !0);
      if (s >= 1 && i < Number(t.length)) {
        const o = Tt(t[i]);
        if (o != null) {
          n.push(o ?? new nt([], []));
          let c = r;
          for (; c < i; )
            l.push(t[c]), c += 1;
          r = i + 1, u = !0;
        }
      }
    }
    u || (l.push(t[r]), r += 1);
  }
  return new It(l.join(`
`), n);
}
class C {
  constructor(t, n, l, r, u, i, s, f, o, c, m, d, a, h, b, p, k, I, L, g, N) {
    this.type = t, this.content = n, this.level = l, this.language = r, this.code = u, this.loading = i, this.children = s, this.ordered = f, this.start = o, this.items = c, this.cells = m, this.header = d, this.rows = a, this.isHeader = h, this.align = b, this.href = p, this.title = k, this.text = I, this.src = L, this.alt = g, this.checked = N;
  }
}
function B() {
  return [];
}
function We(e, t, n) {
  return new C("code_block", null, null, e, t, n, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Qe(e, t) {
  return new C("heading", null, e, null, null, null, t, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Et() {
  return new C("thematic_break", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function jt(e) {
  return new C("blockquote", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function qt(e) {
  return new C("paragraph", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function ut(e, t, n) {
  return new C("table", null, null, null, null, n, null, null, null, null, null, e, t, null, null, null, null, null, null, null, null);
}
function Se(e) {
  return new C("table_row", null, null, null, null, null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null);
}
function xe(e, t, n) {
  return new C("table_cell", null, null, null, null, null, t, null, null, null, null, null, null, e, n, null, null, null, null, null, null);
}
function Ce(e, t, n) {
  return new C("list", null, null, null, null, null, null, e, t, n, null, null, null, null, null, null, null, null, null, null, null);
}
function Dt(e, t) {
  return new C("list_item", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, t);
}
function He(e) {
  return new C("strong", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function se(e) {
  return new C("emphasis", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Me(e) {
  return new C("underline", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Ut(e) {
  return new C("strikethrough", null, null, null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Ge(e) {
  return new C("inline_code", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Vt() {
  return new C("hardbreak", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Wt(e, t) {
  return new C("image", null, null, null, null, !1, null, null, null, null, null, null, null, null, null, null, null, null, e, t, null);
}
function Ke(e, t, n, l, r) {
  return new C("link", null, null, null, null, r, l, null, null, null, null, null, null, null, null, e, t, n, null, null, null);
}
function Qt(e) {
  return new C("wikilink", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, e, null, null, null, null);
}
function Ht(e) {
  return new C("math_inline", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Mt(e, t, n) {
  return new C("callout", null, null, e, null, null, n, null, null, null, null, null, null, null, null, null, t, null, null, null, null);
}
function Gt(e, t, n) {
  return new C("details", null, null, null, null, t, n, null, null, null, null, null, null, null, null, null, null, e, null, null, null);
}
function Kt(e) {
  return new C("query", e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Yt(e) {
  return new C("embed", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, e, null, null);
}
function zt(e) {
  return new C("math_block", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function Xt(e) {
  return new C("mermaid", null, null, null, e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function it(e) {
  return E(e) >= 4 ? !1 : e.trim() == "%{";
}
function $t(e) {
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
function Jt(e) {
  return new C("text", e, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
class W {
  constructor(t, n) {
    this.next = t, this.inner = n;
  }
}
class Z {
  constructor(t, n, l, r, u, i) {
    this.next = t, this.text = n, this.href = l, this.loading = r, this.title = u, this.tail = i;
  }
}
function Zt(e) {
  return e.split(`\r
`).join(`
`).split("\r").join(`
`);
}
function yt(e) {
  const t = lt(e, 10);
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
function Ft(e) {
  const t = lt(e, 10);
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
function en(e, t) {
  if (t)
    return e;
  let n = e, l = !0;
  for (; l; ) {
    l = !1;
    const r = yt(n);
    r != n && (n = r, l = !0);
    const u = Ft(n);
    u != n && (n = u, l = !0);
  }
  return n;
}
function Q(e) {
  return e.trim() == "";
}
function ne(e) {
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
function tn(e, t) {
  const n = Zt(e), r = en(n, t).split(`
`);
  return Ne(r, t);
}
function st(e) {
  if (E(e) >= 4)
    return "";
  const t = e.trim();
  return T(t, "```") ? "`" : T(t, "~~~") ? "~" : "";
}
function nn(e) {
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
function ln(e, t, n) {
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
function ft(e) {
  if (E(e) >= 4)
    return 0;
  const t = e.trim();
  let n = 0;
  for (; n < Number(t.length) && t.charCodeAt(n) == 35; )
    n += 1;
  return n == 0 || n > 6 ? 0 : n == Number(t.length) ? n : t.charCodeAt(n) != 32 ? 0 : n;
}
function rn(e) {
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
  return vt(r);
}
function be(e) {
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
  return (i == Number(e.length) || e.charCodeAt(i) == 32) && (s = !0), s ? rt(e.slice(t, l)) ?? -1 : -1;
}
function ge(e) {
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
function pe(e) {
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
function ot(e) {
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
function un(e) {
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
function Ie(e) {
  return E(e) >= 4 ? !1 : T(re(e), ">");
}
function sn(e) {
  const n = re(e).slice(1);
  return T(n, " ") ? n.slice(1) : n;
}
function Le(e) {
  return Q(e) ? !1 : de(e, 124);
}
function fn(e) {
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
function on(e) {
  if (Q(e) || !de(e, 45))
    return !1;
  const t = J(e);
  if (Number(t.length) == 0)
    return !1;
  for (const n of t) {
    const l = n.trim();
    if (!fn(l))
      return !1;
  }
  return !0;
}
function J(e) {
  let t = e.trim();
  T(t, "|") && (t = t.slice(1)), H(t, "|") && (t = t.slice(0, Number(t.length) - 1));
  const n = t.split("|");
  let l = [], r = 0;
  for (; r < Number(n.length); )
    l.push(n[r]), r += 1;
  return l;
}
function cn(e) {
  const t = e.trim(), n = T(t, ":"), l = H(t, ":");
  return n ? l ? "center" : "left" : l ? "right" : "left";
}
function an(e) {
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
function hn(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) || t == 0 ? e : e.charCodeAt(t - 1) == 10 ? e.slice(0, t) : e;
}
function dn(e) {
  let t = 0;
  for (; t < 4 && t < Number(e.length); )
    if (e.charCodeAt(t) == 32)
      t += 1;
    else
      break;
  return t == 0 ? e : e.slice(t);
}
class Pe {
  constructor(t, n, l) {
    this.name = t, this.argstr = n, this.afterParen = l;
  }
}
function mn(e) {
  return e >= 97 && e <= 122 || e >= 65 && e <= 90 || e >= 48 && e <= 57 || e == 95;
}
function Re(e) {
  let t = 0, n = 0;
  for (; t < Number(e.length) && e.charCodeAt(t) == 32; )
    n += 1, t += 1;
  if (n > 3 || t >= Number(e.length) || e.charCodeAt(t) != 36)
    return null;
  let l = t + 1;
  const r = l;
  for (; l < Number(e.length) && mn(e.charCodeAt(l)); )
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
  return new Pe(s, f, u);
}
function Ye(e) {
  const t = Re(e);
  if (t == null)
    return !1;
  const n = t ?? new Pe("", "", 0);
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
function ze(e, t) {
  let n = t + 1;
  for (; n < Number(e.length); ) {
    if (e.charCodeAt(n) != 32)
      return !1;
    n += 1;
  }
  return !0;
}
function bn(e) {
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
function ct(e, t) {
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
function fe(e, t) {
  const n = ct(e, t);
  if (n == -1 || e.charCodeAt(n) != 34)
    return null;
  let l = n + 1;
  const r = Number(e.length);
  for (; l < r && e.charCodeAt(l) != 34; )
    l += 1;
  return l >= r ? null : e.slice(n + 1, l);
}
function gn(e, t) {
  const n = ct(e, t);
  return n == -1 ? null : e.slice(n, n + 4) == "true" ? !0 : e.slice(n, n + 5) == "false" ? !1 : null;
}
function Ne(e, t) {
  let n = [], l = 0;
  for (; l < Number(e.length); ) {
    const r = e[l];
    if (Q(r)) {
      l += 1;
      continue;
    }
    const u = st(r);
    if (u != "") {
      const a = nn(r), b = r.trim().slice(Number(a.length)).trim();
      let p = [], k = l + 1, I = !1;
      for (; k < Number(e.length); ) {
        if (ln(e[k], u, a)) {
          I = !0;
          break;
        }
        p.push(e[k]), k += 1;
      }
      let L = p.join(`
`);
      if (I)
        if (Number(p.length) > 0 ? L += `
` : L = "", b == "mermaid") {
          const g = p.join(`
`);
          n.push(Xt(g));
        } else
          n.push(We(b, L, !1));
      else {
        for (; Number(p.length) > 0; ) {
          const N = p[Number(p.length) - 1].trim();
          if (N == "" || !an(N))
            break;
          p.pop();
        }
        let g = p.join(`
`);
        g = hn(g), n.push(We(b, g, !t));
      }
      l = k + 1;
      continue;
    }
    if (it(r)) {
      let a = [], h = l + 1, b = !1;
      for (; h < Number(e.length); ) {
        if ($t(e[h])) {
          b = !0;
          break;
        }
        a.push(e[h]), h += 1;
      }
      if (b) {
        const p = a.join(`
`);
        n.push(zt(p)), l = h + 1;
        continue;
      }
    }
    const i = ft(r);
    if (i > 0) {
      const h = r.trim().slice(i).trim(), b = rn(h);
      let p = y(b, t);
      b == "" && (p = B()), n.push(Qe(i, p)), l += 1;
      continue;
    }
    if (ot(r)) {
      n.push(Et()), l += 1;
      continue;
    }
    const s = Re(r);
    if (s != null) {
      const a = s ?? new Pe("", "", 0);
      if (Ye(r)) {
        let h = [], b = l + 1, p = 1, k = !1;
        for (; b < Number(e.length); ) {
          if (bn(e[b])) {
            if (p -= 1, p == 0) {
              k = !0;
              break;
            }
          } else
            Ye(e[b]) && (p += 1);
          h.push(e[b]), b += 1;
        }
        if (k) {
          const I = Ne(h, t);
          if (a.name == "callout") {
            const L = fe(a.argstr, "title") ?? "";
            n.push(Mt(fe(a.argstr, "type") ?? "", L, I));
          } else {
            const L = gn(a.argstr, "open") ?? !1;
            n.push(Gt(fe(a.argstr, "summary") ?? "", L, I));
          }
          l = b + 1;
          continue;
        }
      } else {
        if (a.name == "query" && ze(r, a.afterParen)) {
          n.push(Kt(a.argstr.trim())), l += 1;
          continue;
        }
        if (a.name == "embed" && ze(r, a.afterParen)) {
          n.push(Yt(fe(a.argstr, "src") ?? "")), l += 1;
          continue;
        }
      }
    }
    if (Ie(r)) {
      let a = [], h = l;
      for (; h < Number(e.length); )
        if (Ie(e[h]))
          a.push(sn(e[h])), h += 1;
        else {
          if (Q(e[h]))
            break;
          if (pn(e[h]))
            a.push(e[h]), h += 1;
          else
            break;
        }
      const b = Ne(a, t);
      n.push(jt(b)), l = h;
      continue;
    }
    if (Le(r) && l + 1 < Number(e.length) && on(e[l + 1])) {
      const a = Number(J(r).length), h = Number(J(e[l + 1]).length);
      if (a == h) {
        l = Nn(e, l, n, t);
        continue;
      }
    }
    const f = be(r), o = ge(r);
    if (f >= 0) {
      l = Ae(e, l, !0, f, n, t);
      continue;
    }
    if (o != "") {
      l = Ae(e, l, !1, 0, n, t);
      continue;
    }
    if (pe(r)) {
      l = Ae(e, l, !1, 0, n, t);
      continue;
    }
    let c = [], m = l, d = 0;
    for (; m < Number(e.length); ) {
      const a = e[m];
      if (Q(a))
        break;
      if (Number(c.length) > 0) {
        const h = un(a);
        if (h > 0) {
          d = h, m += 1;
          break;
        }
      }
      if (we(a, e, m))
        break;
      c.push(dn(a)), m += 1;
    }
    if (d > 0) {
      const a = c.join(`
`), h = y(a, t);
      n.push(Qe(d, h)), l = m;
      continue;
    }
    if (Number(c.length) > 0) {
      if (!t) {
        let b = !1;
        if (Number(c.length) >= 2 && (b = !0), m < Number(e.length) && (b = !0), b) {
          const p = c[0];
          if (Le(p) && H(p.trim(), "|")) {
            const k = J(p);
            if (Number(k.length) >= 2) {
              let I = !0, L = 0;
              for (; L < Number(c.length); ) {
                const g = c[L].trim();
                T(g, "|") || (I = !1), L += 1;
              }
              if (I) {
                let g = [];
                for (const U of k) {
                  const O = U.trim(), z = y(O, t);
                  g.push(xe(!0, z, "left"));
                }
                const N = Se(g);
                n.push(ut([N], B(), !0)), l = m;
                continue;
              }
            }
          }
        }
      }
      const a = c.join(`
`), h = y(a, t);
      n.push(qt(h)), l = m;
      continue;
    }
    l += 1;
  }
  return n;
}
function we(e, t, n) {
  return n == 0 ? !1 : !!(st(e) != "" || ft(e) > 0 || ot(e) || Ie(e) || ge(e) != "" || pe(e) || be(e) >= 0 || Re(e) != null || it(e));
}
function pn(e) {
  return !we(e, [], 0);
}
function Ae(e, t, n, l, r, u) {
  let i = [], s = t, f = null;
  for (n && l != 1 && (f = l); s < Number(e.length); ) {
    const o = e[s];
    if (Q(o)) {
      let g = s + 1;
      for (; g < Number(e.length) && Q(e[g]); )
        g += 1;
      if (g < Number(e.length)) {
        if (ge(e[g]) != "") {
          s = g;
          continue;
        }
        if (pe(e[g])) {
          s = g;
          continue;
        }
        if (be(e[g]) >= 0) {
          s = g;
          continue;
        }
      }
      break;
    }
    let c = 0, m = 0, d = !1, a = !1;
    const h = ge(o), b = be(o);
    if (h != "")
      c = ne(o), m = 2 + ne(o), d = !0, a = !1;
    else if (pe(o))
      c = ne(o), m = 1 + ne(o), d = !0, a = !1;
    else if (b >= 0) {
      c = ne(o);
      const g = re(o);
      let N = 0;
      for (; N < Number(g.length); ) {
        const Te = g.charCodeAt(N);
        if (Te >= 48)
          if (Te <= 57)
            N += 1;
          else
            break;
        else
          break;
      }
      let U = N + 1, O = 0, z = N + 1;
      for (; z < Number(g.length) && g.charCodeAt(z) == 32; )
        O += 1, z += 1;
      O > 4 && (O = 1), O < 1 && (O = 1), m = c + U + O, d = !0, a = !0;
    }
    if (!d || a != n)
      break;
    if (!n) {
      const N = re(o).slice(0, 1), O = re(e[t]).slice(0, 1);
      if (N != O)
        break;
    }
    let p = [], k = o.slice(m), I = null;
    for (a || (T(k, "[ ] ") ? (k = k.slice(4), I = !1) : (T(k, "[x] ") || T(k, "[X] ")) && (k = k.slice(4), I = !0)), p.push(k), s += 1; s < Number(e.length); ) {
      const g = e[s];
      if (Q(g)) {
        let N = s + 1;
        for (; N < Number(e.length) && Q(e[N]); )
          N += 1;
        if (N < Number(e.length) && E(e[N]) >= m) {
          we(e[N], e, N);
          let U = s;
          for (; U < N; )
            p.push(""), U += 1;
          s = N;
          continue;
        }
        break;
      }
      if (E(g) >= m) {
        let N = g.slice(m);
        p.push(N), s += 1;
        continue;
      }
      if (!we(g, e, s)) {
        p.push(g), s += 1;
        continue;
      }
      break;
    }
    const L = Ne(p, u);
    i.push(Dt(L, I));
  }
  return n ? f != null ? r.push(Ce(!0, f, i)) : r.push(Ce(!0, null, i)) : r.push(Ce(!1, null, i)), s;
}
function Nn(e, t, n, l) {
  const r = J(e[t]), u = [], i = J(e[t + 1]);
  for (const d of i)
    u.push(cn(d));
  let s = [], f = t + 2;
  for (; f < Number(e.length) && Le(e[f]); ) {
    const d = J(e[f]);
    let a = [], h = 0;
    for (; h < Number(r.length); ) {
      let b = "";
      h < Number(d.length) && (b = d[h].trim());
      let p = "left";
      h < Number(u.length) && (p = u[h]);
      let k = y(b, l);
      b == "" && (k = B()), a.push(xe(!1, k, p)), h += 1;
    }
    s.push(Se(a)), f += 1;
  }
  let o = [], c = 0;
  for (; c < Number(r.length); ) {
    const d = r[c].trim();
    let a = "left";
    c < Number(u.length) && (a = u[c]);
    const h = y(d, l);
    o.push(xe(!0, h, a)), c += 1;
  }
  const m = Se(o);
  return n.push(ut([m], s, !1)), f;
}
function y(e, t) {
  return e == "" ? B() : D(e, t);
}
function D(e, t) {
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
        o != "" && n.push(P(o)), n.push(Vt()), l = "";
      } else
        f > 0 && (l = l.slice(0, Number(l.length) - f)), l += `
`;
      r += 1;
      continue;
    }
    if (s == "*") {
      if (q(e, "***", r)) {
        let o = $(e, r, "***", !1, t);
        if (o != null) {
          const c = o ?? new W(0, "");
          l != "" && (n.push(P(l)), l = ""), n.push(He([se(D(c.inner, t))])), r = c.next;
          continue;
        }
      }
      if (q(e, "**", r)) {
        let o = $(e, r, "**", !0, t);
        if (o != null) {
          const c = o ?? new W(0, "");
          l != "" && (n.push(P(l)), l = ""), n.push(He(D(c.inner, t))), r = c.next;
          continue;
        }
      }
      let f = $(e, r, "*", !1, t);
      if (f != null) {
        const o = f ?? new W(0, "");
        l != "" && (n.push(P(l)), l = ""), n.push(se(D(o.inner, t))), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "_") {
      if (q(e, "___", r)) {
        let o = $(e, r, "___", !1, t);
        if (o != null) {
          const c = o ?? new W(0, "");
          l != "" && (n.push(P(l)), l = ""), n.push(Me([se(D(c.inner, t))])), r = c.next;
          continue;
        }
      }
      if (q(e, "__", r)) {
        let o = $(e, r, "__", !1, t);
        if (o != null) {
          const c = o ?? new W(0, "");
          l != "" && (n.push(P(l)), l = ""), n.push(Me(D(c.inner, t))), r = c.next;
          continue;
        }
      }
      let f = $(e, r, "_", !1, t);
      if (f != null) {
        const o = f ?? new W(0, "");
        l != "" && (n.push(P(l)), l = ""), n.push(se(D(o.inner, t))), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "~") {
      if (q(e, "~~", r)) {
        let f = $(e, r, "~~", !1, t);
        if (f != null) {
          const o = f ?? new W(0, "");
          l != "" && (n.push(P(l)), l = ""), n.push(Ut(D(o.inner, t))), r = o.next;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "`") {
      let f = 0;
      for (; q(e, "`", r + f); )
        f += 1;
      let o = Pn(e, r + f, f);
      if (o != -1) {
        let c = e.slice(r + f, o);
        T(c, " ") && H(c, " ") && c.trim() != "" && (c = c.slice(1, Number(c.length) - 1)), l != "" && (n.push(P(l)), l = ""), n.push(Ge(c)), u = !0, r = o + f;
        continue;
      }
      if (!t && f == 1 && e.slice(r + f).trim() == "" && l == "") {
        r = Number(e.length);
        continue;
      }
      if (f == 1 && !t) {
        const c = e.slice(r + 1);
        l != "" && (n.push(P(l)), l = ""), n.push(Ge(c)), u = !0, r = Number(e.length);
        continue;
      }
      l += e.slice(r, r + f), r += f;
      continue;
    }
    if (s == "!") {
      if (q(e, "![", r)) {
        let f = $e(e, r + 1, t, u);
        if (f != null) {
          const o = f ?? new Z(0, "", "", !1, null, "");
          l != "" && (n.push(P(l)), l = ""), n.push(Wt(o.href, o.text)), r = o.next;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "[") {
      if (q(e, "[[", r)) {
        if (q(e, "[[[", r)) {
          l += "[", r += 1;
          continue;
        }
        let o = me(e, "]]", r + 2);
        if (o != -1) {
          let c = e.slice(r + 2, o).trim();
          if (Be(c, "|") == -1 && !de(c, 10) && c != "") {
            l != "" && (n.push(P(l)), l = ""), n.push(Qt(c)), r = o + 2;
            continue;
          }
        }
        l += "[", r += 1;
        continue;
      }
      let f = $e(e, r, t, u);
      if (f != null) {
        const o = f ?? new Z(0, "", "", !1, null, "");
        l != "" && (n.push(P(l)), l = ""), o.loading ? (n.push(Ke(o.href, o.title, o.text, D(o.text, t), !0)), o.tail != "" && n.push(P(o.tail))) : n.push(Ke(o.href, o.title, o.text, D(o.text, t), !1)), r = o.next;
        continue;
      }
      l += s, r += 1;
      continue;
    }
    if (s == "$") {
      let f = me(e, "$", r + 1);
      if (f != -1) {
        let o = e.slice(r + 1, f), c = Number(o.length), m = 0;
        f + 1 < i && (m = e.charCodeAt(f + 1));
        let d = m >= 48 && m <= 57, a = c > 0 && o.charCodeAt(0) != 32 && !de(o, 10), h = c > 0 && o.charCodeAt(c - 1) != 32 && !d;
        if (a && h) {
          l != "" && (n.push(P(l)), l = ""), n.push(Ht(o)), r = f + 1;
          continue;
        }
      }
      l += s, r += 1;
      continue;
    }
    if (s == "\\" && r + 1 < Number(e.length)) {
      const f = e.charCodeAt(r + 1);
      if (at(f)) {
        f == 34 ? l += "" : f == 39 ? l += "" : l += e.slice(r + 1, r + 2), r += 2;
        continue;
      }
    }
    l += s, r += 1;
  }
  return l != "" && n.push(P(l)), t || wn(n), n;
}
function wn(e) {
  if (Number(e.length) > 0) {
    const t = e[Number(e.length) - 1];
    t.type == "text" && Sn(t, e);
  }
}
function kn(e) {
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
function Cn(e) {
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
function An(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) || t == 0 ? e : e.charCodeAt(t - 1) == 42 ? e.slice(0, t - 1) : e;
}
function _n(e) {
  let t = Number(e.length);
  for (; t > 0 && e.charCodeAt(t - 1) == 32; )
    t -= 1;
  return t == Number(e.length) ? e : e.slice(0, t);
}
function Sn(e, t) {
  let n = e.content ?? "", l = !1, r = kn(n);
  r == n && H(n, "<") && (r = n.slice(0, Number(n.length) - 1)), r != n && (l = !0), n = r;
  let u = Cn(n);
  u != n && (l = !0), n = u;
  let i = An(n);
  i == n && H(n, "*") && (H(n, "**") || (i = n.slice(0, Number(n.length) - 1))), i != n && (l = !0), n = i, l || (n = _n(n)), n.trim() == "|" && (n = ""), n == "" ? t.pop() : (t.pop(), t.push(Jt(n)));
}
function P(e) {
  let t = vn(e);
  return t = t.split("").join('"'), t = t.split("").join("'"), new C("text", t, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null);
}
function ue(e) {
  return e >= 48 && e <= 57 || e >= 65 && e <= 90 || e >= 97 && e <= 122 || e == 95;
}
function xn(e) {
  return e == 41 || e == 93 || e == 125 || e == 44 || e == 46 || e == 59 || e == 58 || e == 33 || e == 63 || e == 8230 || e == 34 || e == 39 || e == 65289 || e == 65292 || e == 65294 || e == 12290 || e == 65307 || e == 65306 || e == 65301 || e == 65311 || e == 12301 || e == 12303 || e == 12313 || e == 12311;
}
function at(e) {
  return e >= 33 && e <= 47 || e >= 58 && e <= 64 || e >= 91 && e <= 96 || e >= 123 && e <= 126 || e == 161 || e == 167 || e == 171 || e == 182 || e == 183 || e == 191 || e >= 8208 && e <= 8286 || e >= 12288 && e <= 12351 || e >= 65281 && e <= 65380;
}
function In() {
  return "“";
}
function oe() {
  return "”";
}
function Ln() {
  return "‘";
}
function Xe() {
  return "’";
}
function vn(e) {
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
        t += In(), n += 1;
        continue;
      }
      if (n + 1 >= Number(e.length)) {
        t += oe(), n += 1;
        continue;
      }
      const i = e.charCodeAt(n + 1);
      i == 32 || i == 10 || xn(i) ? t += oe() : t += r, n += 1;
      continue;
    }
    if (r == "'") {
      let u = !1;
      if (n > 0 && n + 1 < Number(e.length)) {
        const i = e.charCodeAt(n - 1), s = e.charCodeAt(n + 1), f = ue(i), o = ue(s);
        f && o && (u = !0);
      }
      if (u)
        t += Xe();
      else {
        let i = !1;
        if (t == "")
          i = !0;
        else {
          const s = t.slice(Number(t.length) - 1, Number(t.length));
          s == " " && (i = !0), s == `
` && (i = !0), s == "(" && (i = !0), s == "[" && (i = !0);
        }
        i ? t += Ln() : t += Xe();
      }
      n += 1;
      continue;
    }
    t += r, n += 1;
  }
  return t;
}
function Bn(e) {
  return e == 42 ? !0 : e == 95;
}
function $(e, t, n, l, r) {
  if (n == "_" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (ue(o))
      return null;
  }
  if (n == "__" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (ue(o))
      return null;
  }
  if (n == "___" && t > 0) {
    const o = e.charCodeAt(t - 1);
    if (ue(o))
      return null;
  }
  const u = t + Number(n.length), i = e.slice(u);
  let s = Be(i, n), f = i;
  if (s != -1 && (f = i.slice(0, s)), f == "")
    return null;
  if (at(f.charCodeAt(0))) {
    let o = !1;
    if (s != -1 && Bn(f.charCodeAt(0)) && (o = !0), !o)
      return null;
  }
  if (s != -1) {
    let o = u + s + Number(n.length);
    return new W(o, f);
  }
  return !l && r || i == "" || i.charCodeAt(0) == 32 ? null : new W(Number(e.length), i);
}
function Pn(e, t, n) {
  let l = t;
  for (; l < Number(e.length); ) {
    if (e.charCodeAt(l) != 96) {
      l += 1;
      continue;
    }
    let r = 0;
    for (; q(e, "`", l + r); )
      r += 1;
    if (r == n)
      return l;
    l += r;
  }
  return -1;
}
function Rn(e) {
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
function Tn(e) {
  return !!(T(e, "http://") && Number(e.length) > 7 || T(e, "https://") && Number(e.length) > 8);
}
function On(e) {
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
function $e(e, t, n, l) {
  let r = me(e, "]", t);
  if (r == -1)
    return null;
  const u = e.slice(t + 1, r);
  let i = r + 1;
  if (!q(e, "(", i))
    return null;
  let s = me(e, ")", i);
  if (s == -1) {
    const d = e.slice(i + 1);
    if (Tn(d)) {
      let a = Rn(d), h = d.slice(Number(a.length)), b = "";
      return l && (b = null), new Z(Number(e.length), u, a, !0, b, h);
    }
    return On(d) ? new Z(Number(e.length), u, "http://" + d, !0, null, "") : new Z(Number(e.length), u, "", !0, null, "");
  }
  let f = e.slice(i + 1, s), o = f, c = null;
  const m = Be(f, ' "');
  if (m != -1) {
    o = f.slice(0, m);
    const d = f.slice(m + 2);
    H(d, '"') && (c = d.slice(0, Number(d.length) - 1));
  } else
    s + 1 >= Number(e.length) || l || (c = "");
  return new Z(s + 1, u, o, !1, c, "");
}
function ht(e) {
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
function Y(e, t) {
  let n = [];
  for (const l of e) {
    const r = l.type;
    if (r == "text" && n.push(K(l.content ?? "", t, [])), r == "strong")
      for (const u of Y(l.children ?? B(), X(t, G.Strong)))
        n.push(u);
    if (r == "emphasis")
      for (const u of Y(l.children ?? B(), X(t, G.Em)))
        n.push(u);
    if (r == "underline")
      for (const u of Y(l.children ?? B(), X(t, G.Underline)))
        n.push(u);
    if (r == "strikethrough")
      for (const u of Y(l.children ?? B(), X(t, G.Del)))
        n.push(u);
    if (r == "inline_code" && n.push(K(l.code ?? "", X(t, G.Code), [])), r == "hardbreak" && n.push(K(`
`, t, [])), r == "link") {
      let u = [];
      u = _(u, "href", w.Str(l.href ?? ""));
      const i = l.title;
      i != null && (u = _(u, "title", w.Str(i ?? "")));
      for (const s of Y(l.children ?? B(), X(t, G.Link)))
        n.push(K(s.text, s.marks, u));
    }
    if (r == "image") {
      let u = [];
      u = _(u, "src", w.Str(l.src ?? "")), u = _(u, "alt", w.Str(l.alt ?? ""));
      const i = l.title;
      i != null && (u = _(u, "title", w.Str(i ?? ""))), n.push(K(l.alt ?? "", X(t, G.Image), u));
    }
    if (r == "wikilink") {
      let u = [];
      u = _(u, "wikilink", w.Str(l.title ?? "")), n.push(K(l.title ?? "", t, u));
    }
    if (r == "math_inline") {
      let u = [];
      u = _(u, "math_inline", w.Str(l.code ?? "")), n.push(K(l.code ?? "", t, u));
    }
  }
  return n;
}
function En(e, t) {
  let n = [];
  return n = _(n, "header", w.Bool(e.isHeader ?? !1)), n = _(n, "align", w.Str(e.align ?? "left")), v(t, x.TableCell, n, [], Y(e.children ?? B(), []), S(0, 0));
}
function Je(e, t) {
  const n = e.cells ?? B();
  let l = [];
  for (let r = 0; r < Number(n.length); r++)
    l.push(En(n[r], t + "-c" + String(r)));
  return v(t, x.TableRow, [], l, [], S(0, 0));
}
function dt(e, t) {
  const n = e.type;
  if (n == "heading") {
    let l = [];
    return l = _(l, "level", w.Int(e.level ?? 0)), v(t, x.Heading, l, [], Y(e.children ?? B(), []), S(0, 0));
  }
  if (n == "code_block") {
    let l = [];
    return l = _(l, "language", w.Str(e.language ?? "")), l = _(l, "loading", w.Bool(e.loading ?? !1)), v(t, x.Fence, l, [], [F(e.code ?? "")], S(0, 0));
  }
  if (n == "blockquote")
    return v(t, x.Blockquote, [], le(e.children ?? B(), t), [], S(0, 0));
  if (n == "list") {
    let l = [];
    l = _(l, "ordered", w.Bool(e.ordered ?? !1));
    const r = e.start;
    return r != null && (l = _(l, "start", w.Int(r ?? 0))), v(t, x.ListBlock, l, le(e.items ?? B(), t), [], S(0, 0));
  }
  if (n == "list_item") {
    let l = [];
    const r = e.checked;
    return r != null && (l = _(l, "checked", w.Bool(r ?? !1))), v(t, x.ListItem, l, le(e.children ?? B(), t), [], S(0, 0));
  }
  if (n == "table") {
    let l = [];
    const r = e.header ?? B();
    Number(r.length) > 0 && l.push(Je(r[0], t + "-h"));
    const u = e.rows ?? B();
    for (let s = 0; s < Number(u.length); s++)
      l.push(Je(u[s], t + "-r" + String(s)));
    let i = [];
    return i = _(i, "loading", w.Bool(e.loading ?? !1)), v(t, x.Table, i, l, [], S(0, 0));
  }
  if (n == "thematic_break")
    return v(t, x.ThematicBreak, [], [], [], S(0, 0));
  if (n == "callout") {
    let l = [];
    return l = _(l, "type", w.Str(e.language ?? "")), l = _(l, "title", w.Str(e.title ?? "")), v(t, x.Callout, l, le(e.children ?? B(), t), [], S(0, 0));
  }
  if (n == "details") {
    let l = [];
    return l = _(l, "summary", w.Str(e.text ?? "")), (e.loading ?? !1) && (l = _(l, "open", w.Bool(!0))), v(t, x.Details, l, le(e.children ?? B(), t), [], S(0, 0));
  }
  if (n == "query") {
    let l = [];
    return l = _(l, "query", w.Str(e.content ?? "")), v(t, x.QueryBlock, l, [], [], S(0, 0));
  }
  if (n == "embed") {
    let l = [];
    return l = _(l, "src", w.Str(e.src ?? "")), v(t, x.BlockEmbed, l, [], [], S(0, 0));
  }
  return n == "math_block" ? v(t, x.MathBlock, [], [], [F(e.code ?? "")], S(0, 0)) : n == "mermaid" ? v(t, x.Mermaid, [], [], [F(e.code ?? "")], S(0, 0)) : v(t, x.Paragraph, [], [], Y(e.children ?? B(), []), S(0, 0));
}
function le(e, t) {
  let n = [];
  for (let l = 0; l < Number(e.length); l++)
    n.push(dt(e[l], t + "-" + String(l)));
  return n;
}
function Ze(e) {
  let t = [], n = 0;
  for (; n < Number(e.length); ) {
    const l = e[n];
    l == null ? t.push(w.Null()) : t.push(w.Int(l ?? 0)), n += 1;
  }
  return w.ListV(t);
}
function jn(e, t) {
  let n = [], l = 0;
  for (const r of e)
    if (r.kind == x.Table)
      if (l < Number(t.length)) {
        const u = t[l], i = Ze(u.cols), s = Ze(u.rows);
        let f = [];
        f.push(Ee("cols", i)), f.push(Ee("rows", s)), n.push(v(r.id, r.kind, _(r.attrs, "ial", w.AttrsV(f)), r.children, r.inlines, r.source)), l += 1;
      } else
        n.push(r);
    else
      n.push(r);
  return n;
}
function H(e, t) {
  const n = Number(e.length), l = Number(t.length);
  return l > n ? !1 : e.slice(n - l, n) == t;
}
function qn(e, t) {
  let n = [], l = 0;
  for (let r = 0; r < Number(e.length); r++) {
    const u = e[r].text, i = l, s = l + Number(u.length);
    if (l = s, !(i >= t)) if (s <= t)
      n.push(e[r]);
    else {
      const f = t - i;
      f > 0 && n.push(K(u.slice(0, f), e[r].marks, e[r].attrs));
    }
  }
  return n;
}
function Dn(e) {
  return e == x.Paragraph || e == x.Heading || e == x.ListItem;
}
function mt(e) {
  let t = [];
  for (let l = 0; l < Number(e.children.length); l++)
    t.push(mt(e.children[l]));
  let n = v(e.id, e.kind, e.attrs, t, e.inlines, e.source);
  if (Dn(e.kind) && Number(e.inlines.length) > 0) {
    const l = ye(e.inlines), r = ht(l);
    if (r != "") {
      const u = Number(l.length), i = Number(r.length) + 1, s = u - i - 1;
      if (s >= 0) {
        const f = l.slice(s, s + 1);
        if (f == " " || f == "	") {
          const o = qn(e.inlines, s), c = v(e.id, e.kind, e.attrs, t, o, e.source);
          n = _t(c, r);
        }
      }
    }
  }
  return n;
}
function Un(e) {
  const t = Number(e.length);
  if (t < 3)
    return e;
  const n = ht(e);
  if (n == "")
    return e;
  const l = "^" + n, r = Number(l.length);
  if (!H(e, l))
    return e;
  const u = t - r;
  if (u <= 0)
    return e;
  const i = e.slice(u - 1, u);
  return i != " " && i != "	" ? e : e.slice(0, u - 1);
}
function Zn(e) {
  let t = "", n = !1;
  const l = e.split(`
`), r = Number(l.length);
  for (let u = 0; u < r; u++) {
    let i = l[u];
    i.slice(0, 3) == "```" ? n = !n : n || (i = Un(i)), t = t + i, u < r - 1 && (t = t + `
`);
  }
  return t;
}
function yn(e, t) {
  const n = Ot(e), l = n.md, r = tn(l, t);
  let u = [];
  for (let s = 0; s < Number(r.length); s++) {
    const f = "block-" + String(s);
    u.push(mt(dt(r[s], f)));
  }
  const i = jn(u, n.tableAttrs);
  return v("doc", x.Paragraph, [], i, [], S(0, 0));
}
export {
  Xn as $,
  Ce as A,
  De as B,
  jt as C,
  We as D,
  Qe as E,
  Se as F,
  xe as G,
  Vt as H,
  ee as I,
  Qt as J,
  Ht as K,
  Wt as L,
  G as M,
  Ke as N,
  Ge as O,
  Jt as P,
  Ut as Q,
  Me as R,
  xt as S,
  se as T,
  He as U,
  w as V,
  C as W,
  te as X,
  $n as Y,
  Gn as Z,
  Jn as _,
  F as a,
  zn as a0,
  Fe as a1,
  Ct as a2,
  wt as a3,
  Ee as a4,
  X as a5,
  _e as a6,
  Kn as a7,
  Yn as a8,
  yn as a9,
  ke as b,
  gt as c,
  Wn as d,
  ye as e,
  V as f,
  _ as g,
  Qn as h,
  x as i,
  ce as j,
  ae as k,
  Hn as l,
  he as m,
  kt as n,
  Mn as o,
  tn as p,
  St as q,
  M as r,
  Zn as s,
  Vn as t,
  qt as u,
  Et as v,
  je as w,
  ut as x,
  ve as y,
  Dt as z
};
