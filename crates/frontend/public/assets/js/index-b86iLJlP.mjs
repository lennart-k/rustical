import { E } from "./lit-z6_uA4GX.mjs";
/**
 * @license
 * Copyright 2020 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const f$1 = (o2) => void 0 === o2.strings;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const t = { CHILD: 2 }, e$1 = (t2) => (...e2) => ({ _$litDirective$: t2, values: e2 });
class i {
  constructor(t2) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(t2, e2, i2) {
    this._$Ct = t2, this._$AM = e2, this._$Ci = i2;
  }
  _$AS(t2, e2) {
    return this.update(t2, e2);
  }
  update(t2, e2) {
    return this.render(...e2);
  }
}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const s = (i2, t2) => {
  var _a;
  const e2 = i2._$AN;
  if (void 0 === e2) return false;
  for (const i3 of e2) (_a = i3._$AO) == null ? void 0 : _a.call(i3, t2, false), s(i3, t2);
  return true;
}, o$1 = (i2) => {
  let t2, e2;
  do {
    if (void 0 === (t2 = i2._$AM)) break;
    e2 = t2._$AN, e2.delete(i2), i2 = t2;
  } while (0 === (e2 == null ? void 0 : e2.size));
}, r = (i2) => {
  for (let t2; t2 = i2._$AM; i2 = t2) {
    let e2 = t2._$AN;
    if (void 0 === e2) t2._$AN = e2 = /* @__PURE__ */ new Set();
    else if (e2.has(i2)) break;
    e2.add(i2), c(t2);
  }
};
function h$1(i2) {
  void 0 !== this._$AN ? (o$1(this), this._$AM = i2, r(this)) : this._$AM = i2;
}
function n$1(i2, t2 = false, e2 = 0) {
  const r2 = this._$AH, h2 = this._$AN;
  if (void 0 !== h2 && 0 !== h2.size) if (t2) if (Array.isArray(r2)) for (let i3 = e2; i3 < r2.length; i3++) s(r2[i3], false), o$1(r2[i3]);
  else null != r2 && (s(r2, false), o$1(r2));
  else s(this, i2);
}
const c = (i2) => {
  i2.type == t.CHILD && (i2._$AP ?? (i2._$AP = n$1), i2._$AQ ?? (i2._$AQ = h$1));
};
class f extends i {
  constructor() {
    super(...arguments), this._$AN = void 0;
  }
  _$AT(i2, t2, e2) {
    super._$AT(i2, t2, e2), r(this), this.isConnected = i2._$AU;
  }
  _$AO(i2, t2 = true) {
    var _a, _b;
    i2 !== this.isConnected && (this.isConnected = i2, i2 ? (_a = this.reconnected) == null ? void 0 : _a.call(this) : (_b = this.disconnected) == null ? void 0 : _b.call(this)), t2 && (s(this, i2), o$1(this));
  }
  setValue(t2) {
    if (f$1(this._$Ct)) this._$Ct._$AI(t2, this);
    else {
      const i2 = [...this._$Ct._$AH];
      i2[this._$Ci] = t2, this._$Ct._$AI(i2, this, 0);
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
const e = () => new h();
class h {
}
const o = /* @__PURE__ */ new WeakMap(), n = e$1(class extends f {
  render(i2) {
    return E;
  }
  update(i2, [s2]) {
    var _a;
    const e2 = s2 !== this.G;
    return e2 && void 0 !== this.G && this.rt(void 0), (e2 || this.lt !== this.ct) && (this.G = s2, this.ht = (_a = i2.options) == null ? void 0 : _a.host, this.rt(this.ct = i2.element)), E;
  }
  rt(t2) {
    if (this.isConnected || (t2 = void 0), "function" == typeof this.G) {
      const i2 = this.ht ?? globalThis;
      let s2 = o.get(i2);
      void 0 === s2 && (s2 = /* @__PURE__ */ new WeakMap(), o.set(i2, s2)), void 0 !== s2.get(this.G) && this.G.call(this.ht, void 0), s2.set(this.G, t2), void 0 !== t2 && this.G.call(this.ht, t2);
    } else this.G.value = t2;
  }
  get lt() {
    var _a, _b;
    return "function" == typeof this.G ? (_a = o.get(this.ht ?? globalThis)) == null ? void 0 : _a.get(this.G) : (_b = this.G) == null ? void 0 : _b.value;
  }
  disconnected() {
    this.lt === this.ct && this.rt(void 0);
  }
  reconnected() {
    this.rt(this.ct);
  }
});
function escapeXml(unsafe) {
  return unsafe.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");
}
export {
  escapeXml as a,
  e,
  n
};
