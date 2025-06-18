import { f, u as _, E as $ } from "./lit-CWlWuEHk.mjs";
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const T = (t) => (e, s) => {
  s !== void 0 ? s.addInitializer(() => {
    customElements.define(t, e);
  }) : customElements.define(t, e);
};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const A = { attribute: !0, type: String, converter: _, reflect: !1, hasChanged: f }, p = (t = A, e, s) => {
  const { kind: i, metadata: n } = s;
  let r = globalThis.litPropertyMetadata.get(n);
  if (r === void 0 && globalThis.litPropertyMetadata.set(n, r = /* @__PURE__ */ new Map()), i === "setter" && ((t = Object.create(t)).wrapped = !0), r.set(s.name, t), i === "accessor") {
    const { name: o } = s;
    return { set(h) {
      const l = e.get.call(this);
      e.set.call(this, h), this.requestUpdate(o, l, t);
    }, init(h) {
      return h !== void 0 && this.C(o, void 0, t, h), h;
    } };
  }
  if (i === "setter") {
    const { name: o } = s;
    return function(h) {
      const l = this[o];
      e.call(this, h), this.requestUpdate(o, l, t);
    };
  }
  throw Error("Unsupported decorator location: " + i);
};
function O(t) {
  return (e, s) => typeof s == "object" ? p(t, e, s) : ((i, n, r) => {
    const o = n.hasOwnProperty(r);
    return n.constructor.createProperty(r, i), o ? Object.getOwnPropertyDescriptor(n, r) : void 0;
  })(t, e, s);
}
/**
 * @license
 * Copyright 2020 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const v = (t) => t.strings === void 0;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const g = { CHILD: 2 }, C = (t) => (...e) => ({ _$litDirective$: t, values: e });
class m {
  constructor(e) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(e, s, i) {
    this._$Ct = e, this._$AM = s, this._$Ci = i;
  }
  _$AS(e, s) {
    return this.update(e, s);
  }
  update(e, s) {
    return this.render(...s);
  }
}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const c = (t, e) => {
  var i;
  const s = t._$AN;
  if (s === void 0) return !1;
  for (const n of s) (i = n._$AO) == null || i.call(n, e, !1), c(n, e);
  return !0;
}, a = (t) => {
  let e, s;
  do {
    if ((e = t._$AM) === void 0) break;
    s = e._$AN, s.delete(t), t = e;
  } while ((s == null ? void 0 : s.size) === 0);
}, u = (t) => {
  for (let e; e = t._$AM; t = e) {
    let s = e._$AN;
    if (s === void 0) e._$AN = s = /* @__PURE__ */ new Set();
    else if (s.has(t)) break;
    s.add(t), M(e);
  }
};
function y(t) {
  this._$AN !== void 0 ? (a(this), this._$AM = t, u(this)) : this._$AM = t;
}
function G(t, e = !1, s = 0) {
  const i = this._$AH, n = this._$AN;
  if (n !== void 0 && n.size !== 0) if (e) if (Array.isArray(i)) for (let r = s; r < i.length; r++) c(i[r], !1), a(i[r]);
  else i != null && (c(i, !1), a(i));
  else c(this, t);
}
const M = (t) => {
  t.type == g.CHILD && (t._$AP ?? (t._$AP = G), t._$AQ ?? (t._$AQ = y));
};
class b extends m {
  constructor() {
    super(...arguments), this._$AN = void 0;
  }
  _$AT(e, s, i) {
    super._$AT(e, s, i), u(this), this.isConnected = e._$AU;
  }
  _$AO(e, s = !0) {
    var i, n;
    e !== this.isConnected && (this.isConnected = e, e ? (i = this.reconnected) == null || i.call(this) : (n = this.disconnected) == null || n.call(this)), s && (c(this, e), a(this));
  }
  setValue(e) {
    if (v(this._$Ct)) this._$Ct._$AI(e, this);
    else {
      const s = [...this._$Ct._$AH];
      s[this._$Ci] = e, this._$Ct._$AI(s, this, 0);
    }
  }
  disconnected() {
  }
  reconnected() {
  }
}
/**
 * @license
 * Copyright 2020 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const P = () => new w();
class w {
}
const d = /* @__PURE__ */ new WeakMap(), U = C(class extends b {
  render(t) {
    return $;
  }
  update(t, [e]) {
    var i;
    const s = e !== this.G;
    return s && this.G !== void 0 && this.rt(void 0), (s || this.lt !== this.ct) && (this.G = e, this.ht = (i = t.options) == null ? void 0 : i.host, this.rt(this.ct = t.element)), $;
  }
  rt(t) {
    if (this.isConnected || (t = void 0), typeof this.G == "function") {
      const e = this.ht ?? globalThis;
      let s = d.get(e);
      s === void 0 && (s = /* @__PURE__ */ new WeakMap(), d.set(e, s)), s.get(this.G) !== void 0 && this.G.call(this.ht, void 0), s.set(this.G, t), t !== void 0 && this.G.call(this.ht, t);
    } else this.G.value = t;
  }
  get lt() {
    var t, e;
    return typeof this.G == "function" ? (t = d.get(this.ht ?? globalThis)) == null ? void 0 : t.get(this.G) : (e = this.G) == null ? void 0 : e.value;
  }
  disconnected() {
    this.lt === this.ct && this.rt(void 0);
  }
  reconnected() {
    this.rt(this.ct);
  }
});
export {
  O as a,
  P as e,
  U as n,
  T as t
};
