import { f, u } from "./lit-DkXrt_Iv.mjs";
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const t = (t2) => (e, o2) => {
  void 0 !== o2 ? o2.addInitializer((() => {
    customElements.define(t2, e);
  })) : customElements.define(t2, e);
};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const o = { attribute: true, type: String, converter: u, reflect: false, hasChanged: f }, r = (t2 = o, e, r2) => {
  const { kind: n2, metadata: i } = r2;
  let s = globalThis.litPropertyMetadata.get(i);
  if (void 0 === s && globalThis.litPropertyMetadata.set(i, s = /* @__PURE__ */ new Map()), "setter" === n2 && ((t2 = Object.create(t2)).wrapped = true), s.set(r2.name, t2), "accessor" === n2) {
    const { name: o2 } = r2;
    return { set(r3) {
      const n3 = e.get.call(this);
      e.set.call(this, r3), this.requestUpdate(o2, n3, t2);
    }, init(e2) {
      return void 0 !== e2 && this.C(o2, void 0, t2, e2), e2;
    } };
  }
  if ("setter" === n2) {
    const { name: o2 } = r2;
    return function(r3) {
      const n3 = this[o2];
      e.call(this, r3), this.requestUpdate(o2, n3, t2);
    };
  }
  throw Error("Unsupported decorator location: " + n2);
};
function n(t2) {
  return (e, o2) => "object" == typeof o2 ? r(t2, e, o2) : ((t3, e2, o3) => {
    const r2 = e2.hasOwnProperty(o3);
    return e2.constructor.createProperty(o3, t3), r2 ? Object.getOwnPropertyDescriptor(e2, o3) : void 0;
  })(t2, e, o2);
}
export {
  n,
  t
};
