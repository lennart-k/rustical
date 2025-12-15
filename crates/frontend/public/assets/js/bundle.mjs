const t$3 = globalThis, e$4 = t$3.ShadowRoot && (void 0 === t$3.ShadyCSS || t$3.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype, s$3 = /* @__PURE__ */ Symbol(), o$6 = /* @__PURE__ */ new WeakMap();
let n$5 = class n {
  constructor(t2, e2, o2) {
    if (this._$cssResult$ = true, o2 !== s$3) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t2, this.t = e2;
  }
  get styleSheet() {
    let t2 = this.o;
    const s2 = this.t;
    if (e$4 && void 0 === t2) {
      const e2 = void 0 !== s2 && 1 === s2.length;
      e2 && (t2 = o$6.get(s2)), void 0 === t2 && ((this.o = t2 = new CSSStyleSheet()).replaceSync(this.cssText), e2 && o$6.set(s2, t2));
    }
    return t2;
  }
  toString() {
    return this.cssText;
  }
};
const r$4 = (t2) => new n$5("string" == typeof t2 ? t2 : t2 + "", void 0, s$3), S$1 = (s2, o2) => {
  if (e$4) s2.adoptedStyleSheets = o2.map(((t2) => t2 instanceof CSSStyleSheet ? t2 : t2.styleSheet));
  else for (const e2 of o2) {
    const o3 = document.createElement("style"), n3 = t$3.litNonce;
    void 0 !== n3 && o3.setAttribute("nonce", n3), o3.textContent = e2.cssText, s2.appendChild(o3);
  }
}, c$3 = e$4 ? (t2) => t2 : (t2) => t2 instanceof CSSStyleSheet ? ((t3) => {
  let e2 = "";
  for (const s2 of t3.cssRules) e2 += s2.cssText;
  return r$4(e2);
})(t2) : t2;
const { is: i$3, defineProperty: e$3, getOwnPropertyDescriptor: h$3, getOwnPropertyNames: r$3, getOwnPropertySymbols: o$5, getPrototypeOf: n$4 } = Object, a$1 = globalThis, c$2 = a$1.trustedTypes, l$1 = c$2 ? c$2.emptyScript : "", p$1 = a$1.reactiveElementPolyfillSupport, d$1 = (t2, s2) => t2, u$1 = { toAttribute(t2, s2) {
  switch (s2) {
    case Boolean:
      t2 = t2 ? l$1 : null;
      break;
    case Object:
    case Array:
      t2 = null == t2 ? t2 : JSON.stringify(t2);
  }
  return t2;
}, fromAttribute(t2, s2) {
  let i3 = t2;
  switch (s2) {
    case Boolean:
      i3 = null !== t2;
      break;
    case Number:
      i3 = null === t2 ? null : Number(t2);
      break;
    case Object:
    case Array:
      try {
        i3 = JSON.parse(t2);
      } catch (t3) {
        i3 = null;
      }
  }
  return i3;
} }, f$3 = (t2, s2) => !i$3(t2, s2), b$1 = { attribute: true, type: String, converter: u$1, reflect: false, useDefault: false, hasChanged: f$3 };
Symbol.metadata ??= /* @__PURE__ */ Symbol("metadata"), a$1.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
let y$1 = class y extends HTMLElement {
  static addInitializer(t2) {
    this._$Ei(), (this.l ??= []).push(t2);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t2, s2 = b$1) {
    if (s2.state && (s2.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t2) && ((s2 = Object.create(s2)).wrapped = true), this.elementProperties.set(t2, s2), !s2.noAccessor) {
      const i3 = /* @__PURE__ */ Symbol(), h2 = this.getPropertyDescriptor(t2, i3, s2);
      void 0 !== h2 && e$3(this.prototype, t2, h2);
    }
  }
  static getPropertyDescriptor(t2, s2, i3) {
    const { get: e2, set: r2 } = h$3(this.prototype, t2) ?? { get() {
      return this[s2];
    }, set(t3) {
      this[s2] = t3;
    } };
    return { get: e2, set(s3) {
      const h2 = e2?.call(this);
      r2?.call(this, s3), this.requestUpdate(t2, h2, i3);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t2) {
    return this.elementProperties.get(t2) ?? b$1;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d$1("elementProperties"))) return;
    const t2 = n$4(this);
    t2.finalize(), void 0 !== t2.l && (this.l = [...t2.l]), this.elementProperties = new Map(t2.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d$1("finalized"))) return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d$1("properties"))) {
      const t3 = this.properties, s2 = [...r$3(t3), ...o$5(t3)];
      for (const i3 of s2) this.createProperty(i3, t3[i3]);
    }
    const t2 = this[Symbol.metadata];
    if (null !== t2) {
      const s2 = litPropertyMetadata.get(t2);
      if (void 0 !== s2) for (const [t3, i3] of s2) this.elementProperties.set(t3, i3);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t3, s2] of this.elementProperties) {
      const i3 = this._$Eu(t3, s2);
      void 0 !== i3 && this._$Eh.set(i3, t3);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s2) {
    const i3 = [];
    if (Array.isArray(s2)) {
      const e2 = new Set(s2.flat(1 / 0).reverse());
      for (const s3 of e2) i3.unshift(c$3(s3));
    } else void 0 !== s2 && i3.push(c$3(s2));
    return i3;
  }
  static _$Eu(t2, s2) {
    const i3 = s2.attribute;
    return false === i3 ? void 0 : "string" == typeof i3 ? i3 : "string" == typeof t2 ? t2.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise(((t2) => this.enableUpdating = t2)), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach(((t2) => t2(this)));
  }
  addController(t2) {
    (this._$EO ??= /* @__PURE__ */ new Set()).add(t2), void 0 !== this.renderRoot && this.isConnected && t2.hostConnected?.();
  }
  removeController(t2) {
    this._$EO?.delete(t2);
  }
  _$E_() {
    const t2 = /* @__PURE__ */ new Map(), s2 = this.constructor.elementProperties;
    for (const i3 of s2.keys()) this.hasOwnProperty(i3) && (t2.set(i3, this[i3]), delete this[i3]);
    t2.size > 0 && (this._$Ep = t2);
  }
  createRenderRoot() {
    const t2 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S$1(t2, this.constructor.elementStyles), t2;
  }
  connectedCallback() {
    this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(true), this._$EO?.forEach(((t2) => t2.hostConnected?.()));
  }
  enableUpdating(t2) {
  }
  disconnectedCallback() {
    this._$EO?.forEach(((t2) => t2.hostDisconnected?.()));
  }
  attributeChangedCallback(t2, s2, i3) {
    this._$AK(t2, i3);
  }
  _$ET(t2, s2) {
    const i3 = this.constructor.elementProperties.get(t2), e2 = this.constructor._$Eu(t2, i3);
    if (void 0 !== e2 && true === i3.reflect) {
      const h2 = (void 0 !== i3.converter?.toAttribute ? i3.converter : u$1).toAttribute(s2, i3.type);
      this._$Em = t2, null == h2 ? this.removeAttribute(e2) : this.setAttribute(e2, h2), this._$Em = null;
    }
  }
  _$AK(t2, s2) {
    const i3 = this.constructor, e2 = i3._$Eh.get(t2);
    if (void 0 !== e2 && this._$Em !== e2) {
      const t3 = i3.getPropertyOptions(e2), h2 = "function" == typeof t3.converter ? { fromAttribute: t3.converter } : void 0 !== t3.converter?.fromAttribute ? t3.converter : u$1;
      this._$Em = e2;
      const r2 = h2.fromAttribute(s2, t3.type);
      this[e2] = r2 ?? this._$Ej?.get(e2) ?? r2, this._$Em = null;
    }
  }
  requestUpdate(t2, s2, i3) {
    if (void 0 !== t2) {
      const e2 = this.constructor, h2 = this[t2];
      if (i3 ??= e2.getPropertyOptions(t2), !((i3.hasChanged ?? f$3)(h2, s2) || i3.useDefault && i3.reflect && h2 === this._$Ej?.get(t2) && !this.hasAttribute(e2._$Eu(t2, i3)))) return;
      this.C(t2, s2, i3);
    }
    false === this.isUpdatePending && (this._$ES = this._$EP());
  }
  C(t2, s2, { useDefault: i3, reflect: e2, wrapped: h2 }, r2) {
    i3 && !(this._$Ej ??= /* @__PURE__ */ new Map()).has(t2) && (this._$Ej.set(t2, r2 ?? s2 ?? this[t2]), true !== h2 || void 0 !== r2) || (this._$AL.has(t2) || (this.hasUpdated || i3 || (s2 = void 0), this._$AL.set(t2, s2)), true === e2 && this._$Em !== t2 && (this._$Eq ??= /* @__PURE__ */ new Set()).add(t2));
  }
  async _$EP() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t3) {
      Promise.reject(t3);
    }
    const t2 = this.scheduleUpdate();
    return null != t2 && await t2, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending) return;
    if (!this.hasUpdated) {
      if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
        for (const [t4, s3] of this._$Ep) this[t4] = s3;
        this._$Ep = void 0;
      }
      const t3 = this.constructor.elementProperties;
      if (t3.size > 0) for (const [s3, i3] of t3) {
        const { wrapped: t4 } = i3, e2 = this[s3];
        true !== t4 || this._$AL.has(s3) || void 0 === e2 || this.C(s3, void 0, i3, e2);
      }
    }
    let t2 = false;
    const s2 = this._$AL;
    try {
      t2 = this.shouldUpdate(s2), t2 ? (this.willUpdate(s2), this._$EO?.forEach(((t3) => t3.hostUpdate?.())), this.update(s2)) : this._$EM();
    } catch (s3) {
      throw t2 = false, this._$EM(), s3;
    }
    t2 && this._$AE(s2);
  }
  willUpdate(t2) {
  }
  _$AE(t2) {
    this._$EO?.forEach(((t3) => t3.hostUpdated?.())), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t2)), this.updated(t2);
  }
  _$EM() {
    this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = false;
  }
  get updateComplete() {
    return this.getUpdateComplete();
  }
  getUpdateComplete() {
    return this._$ES;
  }
  shouldUpdate(t2) {
    return true;
  }
  update(t2) {
    this._$Eq &&= this._$Eq.forEach(((t3) => this._$ET(t3, this[t3]))), this._$EM();
  }
  updated(t2) {
  }
  firstUpdated(t2) {
  }
};
y$1.elementStyles = [], y$1.shadowRootOptions = { mode: "open" }, y$1[d$1("elementProperties")] = /* @__PURE__ */ new Map(), y$1[d$1("finalized")] = /* @__PURE__ */ new Map(), p$1?.({ ReactiveElement: y$1 }), (a$1.reactiveElementVersions ??= []).push("2.1.1");
const t$2 = globalThis, i$2 = t$2.trustedTypes, s$2 = i$2 ? i$2.createPolicy("lit-html", { createHTML: (t2) => t2 }) : void 0, e$2 = "$lit$", h$2 = `lit$${Math.random().toFixed(9).slice(2)}$`, o$4 = "?" + h$2, n$3 = `<${o$4}>`, r$2 = document, l = () => r$2.createComment(""), c$1 = (t2) => null === t2 || "object" != typeof t2 && "function" != typeof t2, a = Array.isArray, u = (t2) => a(t2) || "function" == typeof t2?.[Symbol.iterator], d = "[ 	\n\f\r]", f$2 = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g, v = /-->/g, _ = />/g, m = RegExp(`>|${d}(?:([^\\s"'>=/]+)(${d}*=${d}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g"), p = /'/g, g = /"/g, $ = /^(?:script|style|textarea|title)$/i, y2 = (t2) => (i3, ...s2) => ({ _$litType$: t2, strings: i3, values: s2 }), x = y2(1), b = y2(2), T = /* @__PURE__ */ Symbol.for("lit-noChange"), E = /* @__PURE__ */ Symbol.for("lit-nothing"), A = /* @__PURE__ */ new WeakMap(), C = r$2.createTreeWalker(r$2, 129);
function P(t2, i3) {
  if (!a(t2) || !t2.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return void 0 !== s$2 ? s$2.createHTML(i3) : i3;
}
const V = (t2, i3) => {
  const s2 = t2.length - 1, o2 = [];
  let r2, l2 = 2 === i3 ? "<svg>" : 3 === i3 ? "<math>" : "", c2 = f$2;
  for (let i4 = 0; i4 < s2; i4++) {
    const s3 = t2[i4];
    let a2, u2, d2 = -1, y3 = 0;
    for (; y3 < s3.length && (c2.lastIndex = y3, u2 = c2.exec(s3), null !== u2); ) y3 = c2.lastIndex, c2 === f$2 ? "!--" === u2[1] ? c2 = v : void 0 !== u2[1] ? c2 = _ : void 0 !== u2[2] ? ($.test(u2[2]) && (r2 = RegExp("</" + u2[2], "g")), c2 = m) : void 0 !== u2[3] && (c2 = m) : c2 === m ? ">" === u2[0] ? (c2 = r2 ?? f$2, d2 = -1) : void 0 === u2[1] ? d2 = -2 : (d2 = c2.lastIndex - u2[2].length, a2 = u2[1], c2 = void 0 === u2[3] ? m : '"' === u2[3] ? g : p) : c2 === g || c2 === p ? c2 = m : c2 === v || c2 === _ ? c2 = f$2 : (c2 = m, r2 = void 0);
    const x2 = c2 === m && t2[i4 + 1].startsWith("/>") ? " " : "";
    l2 += c2 === f$2 ? s3 + n$3 : d2 >= 0 ? (o2.push(a2), s3.slice(0, d2) + e$2 + s3.slice(d2) + h$2 + x2) : s3 + h$2 + (-2 === d2 ? i4 : x2);
  }
  return [P(t2, l2 + (t2[s2] || "<?>") + (2 === i3 ? "</svg>" : 3 === i3 ? "</math>" : "")), o2];
};
class N {
  constructor({ strings: t2, _$litType$: s2 }, n3) {
    let r2;
    this.parts = [];
    let c2 = 0, a2 = 0;
    const u2 = t2.length - 1, d2 = this.parts, [f2, v2] = V(t2, s2);
    if (this.el = N.createElement(f2, n3), C.currentNode = this.el.content, 2 === s2 || 3 === s2) {
      const t3 = this.el.content.firstChild;
      t3.replaceWith(...t3.childNodes);
    }
    for (; null !== (r2 = C.nextNode()) && d2.length < u2; ) {
      if (1 === r2.nodeType) {
        if (r2.hasAttributes()) for (const t3 of r2.getAttributeNames()) if (t3.endsWith(e$2)) {
          const i3 = v2[a2++], s3 = r2.getAttribute(t3).split(h$2), e2 = /([.?@])?(.*)/.exec(i3);
          d2.push({ type: 1, index: c2, name: e2[2], strings: s3, ctor: "." === e2[1] ? H : "?" === e2[1] ? I : "@" === e2[1] ? L : k }), r2.removeAttribute(t3);
        } else t3.startsWith(h$2) && (d2.push({ type: 6, index: c2 }), r2.removeAttribute(t3));
        if ($.test(r2.tagName)) {
          const t3 = r2.textContent.split(h$2), s3 = t3.length - 1;
          if (s3 > 0) {
            r2.textContent = i$2 ? i$2.emptyScript : "";
            for (let i3 = 0; i3 < s3; i3++) r2.append(t3[i3], l()), C.nextNode(), d2.push({ type: 2, index: ++c2 });
            r2.append(t3[s3], l());
          }
        }
      } else if (8 === r2.nodeType) if (r2.data === o$4) d2.push({ type: 2, index: c2 });
      else {
        let t3 = -1;
        for (; -1 !== (t3 = r2.data.indexOf(h$2, t3 + 1)); ) d2.push({ type: 7, index: c2 }), t3 += h$2.length - 1;
      }
      c2++;
    }
  }
  static createElement(t2, i3) {
    const s2 = r$2.createElement("template");
    return s2.innerHTML = t2, s2;
  }
}
function S(t2, i3, s2 = t2, e2) {
  if (i3 === T) return i3;
  let h2 = void 0 !== e2 ? s2._$Co?.[e2] : s2._$Cl;
  const o2 = c$1(i3) ? void 0 : i3._$litDirective$;
  return h2?.constructor !== o2 && (h2?._$AO?.(false), void 0 === o2 ? h2 = void 0 : (h2 = new o2(t2), h2._$AT(t2, s2, e2)), void 0 !== e2 ? (s2._$Co ??= [])[e2] = h2 : s2._$Cl = h2), void 0 !== h2 && (i3 = S(t2, h2._$AS(t2, i3.values), h2, e2)), i3;
}
class M {
  constructor(t2, i3) {
    this._$AV = [], this._$AN = void 0, this._$AD = t2, this._$AM = i3;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t2) {
    const { el: { content: i3 }, parts: s2 } = this._$AD, e2 = (t2?.creationScope ?? r$2).importNode(i3, true);
    C.currentNode = e2;
    let h2 = C.nextNode(), o2 = 0, n3 = 0, l2 = s2[0];
    for (; void 0 !== l2; ) {
      if (o2 === l2.index) {
        let i4;
        2 === l2.type ? i4 = new R(h2, h2.nextSibling, this, t2) : 1 === l2.type ? i4 = new l2.ctor(h2, l2.name, l2.strings, this, t2) : 6 === l2.type && (i4 = new z(h2, this, t2)), this._$AV.push(i4), l2 = s2[++n3];
      }
      o2 !== l2?.index && (h2 = C.nextNode(), o2++);
    }
    return C.currentNode = r$2, e2;
  }
  p(t2) {
    let i3 = 0;
    for (const s2 of this._$AV) void 0 !== s2 && (void 0 !== s2.strings ? (s2._$AI(t2, s2, i3), i3 += s2.strings.length - 2) : s2._$AI(t2[i3])), i3++;
  }
}
class R {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t2, i3, s2, e2) {
    this.type = 2, this._$AH = E, this._$AN = void 0, this._$AA = t2, this._$AB = i3, this._$AM = s2, this.options = e2, this._$Cv = e2?.isConnected ?? true;
  }
  get parentNode() {
    let t2 = this._$AA.parentNode;
    const i3 = this._$AM;
    return void 0 !== i3 && 11 === t2?.nodeType && (t2 = i3.parentNode), t2;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t2, i3 = this) {
    t2 = S(this, t2, i3), c$1(t2) ? t2 === E || null == t2 || "" === t2 ? (this._$AH !== E && this._$AR(), this._$AH = E) : t2 !== this._$AH && t2 !== T && this._(t2) : void 0 !== t2._$litType$ ? this.$(t2) : void 0 !== t2.nodeType ? this.T(t2) : u(t2) ? this.k(t2) : this._(t2);
  }
  O(t2) {
    return this._$AA.parentNode.insertBefore(t2, this._$AB);
  }
  T(t2) {
    this._$AH !== t2 && (this._$AR(), this._$AH = this.O(t2));
  }
  _(t2) {
    this._$AH !== E && c$1(this._$AH) ? this._$AA.nextSibling.data = t2 : this.T(r$2.createTextNode(t2)), this._$AH = t2;
  }
  $(t2) {
    const { values: i3, _$litType$: s2 } = t2, e2 = "number" == typeof s2 ? this._$AC(t2) : (void 0 === s2.el && (s2.el = N.createElement(P(s2.h, s2.h[0]), this.options)), s2);
    if (this._$AH?._$AD === e2) this._$AH.p(i3);
    else {
      const t3 = new M(e2, this), s3 = t3.u(this.options);
      t3.p(i3), this.T(s3), this._$AH = t3;
    }
  }
  _$AC(t2) {
    let i3 = A.get(t2.strings);
    return void 0 === i3 && A.set(t2.strings, i3 = new N(t2)), i3;
  }
  k(t2) {
    a(this._$AH) || (this._$AH = [], this._$AR());
    const i3 = this._$AH;
    let s2, e2 = 0;
    for (const h2 of t2) e2 === i3.length ? i3.push(s2 = new R(this.O(l()), this.O(l()), this, this.options)) : s2 = i3[e2], s2._$AI(h2), e2++;
    e2 < i3.length && (this._$AR(s2 && s2._$AB.nextSibling, e2), i3.length = e2);
  }
  _$AR(t2 = this._$AA.nextSibling, i3) {
    for (this._$AP?.(false, true, i3); t2 !== this._$AB; ) {
      const i4 = t2.nextSibling;
      t2.remove(), t2 = i4;
    }
  }
  setConnected(t2) {
    void 0 === this._$AM && (this._$Cv = t2, this._$AP?.(t2));
  }
}
class k {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t2, i3, s2, e2, h2) {
    this.type = 1, this._$AH = E, this._$AN = void 0, this.element = t2, this.name = i3, this._$AM = e2, this.options = h2, s2.length > 2 || "" !== s2[0] || "" !== s2[1] ? (this._$AH = Array(s2.length - 1).fill(new String()), this.strings = s2) : this._$AH = E;
  }
  _$AI(t2, i3 = this, s2, e2) {
    const h2 = this.strings;
    let o2 = false;
    if (void 0 === h2) t2 = S(this, t2, i3, 0), o2 = !c$1(t2) || t2 !== this._$AH && t2 !== T, o2 && (this._$AH = t2);
    else {
      const e3 = t2;
      let n3, r2;
      for (t2 = h2[0], n3 = 0; n3 < h2.length - 1; n3++) r2 = S(this, e3[s2 + n3], i3, n3), r2 === T && (r2 = this._$AH[n3]), o2 ||= !c$1(r2) || r2 !== this._$AH[n3], r2 === E ? t2 = E : t2 !== E && (t2 += (r2 ?? "") + h2[n3 + 1]), this._$AH[n3] = r2;
    }
    o2 && !e2 && this.j(t2);
  }
  j(t2) {
    t2 === E ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t2 ?? "");
  }
}
class H extends k {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t2) {
    this.element[this.name] = t2 === E ? void 0 : t2;
  }
}
class I extends k {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t2) {
    this.element.toggleAttribute(this.name, !!t2 && t2 !== E);
  }
}
class L extends k {
  constructor(t2, i3, s2, e2, h2) {
    super(t2, i3, s2, e2, h2), this.type = 5;
  }
  _$AI(t2, i3 = this) {
    if ((t2 = S(this, t2, i3, 0) ?? E) === T) return;
    const s2 = this._$AH, e2 = t2 === E && s2 !== E || t2.capture !== s2.capture || t2.once !== s2.once || t2.passive !== s2.passive, h2 = t2 !== E && (s2 === E || e2);
    e2 && this.element.removeEventListener(this.name, this, s2), h2 && this.element.addEventListener(this.name, this, t2), this._$AH = t2;
  }
  handleEvent(t2) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t2) : this._$AH.handleEvent(t2);
  }
}
class z {
  constructor(t2, i3, s2) {
    this.element = t2, this.type = 6, this._$AN = void 0, this._$AM = i3, this.options = s2;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t2) {
    S(this, t2);
  }
}
const j = t$2.litHtmlPolyfillSupport;
j?.(N, R), (t$2.litHtmlVersions ??= []).push("3.3.1");
const B = (t2, i3, s2) => {
  const e2 = s2?.renderBefore ?? i3;
  let h2 = e2._$litPart$;
  if (void 0 === h2) {
    const t3 = s2?.renderBefore ?? null;
    e2._$litPart$ = h2 = new R(i3.insertBefore(l(), t3), t3, void 0, s2 ?? {});
  }
  return h2._$AI(t2), h2;
};
const s$1 = globalThis;
let i$1 = class i extends y$1 {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    const t2 = super.createRenderRoot();
    return this.renderOptions.renderBefore ??= t2.firstChild, t2;
  }
  update(t2) {
    const r2 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t2), this._$Do = B(r2, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this._$Do?.setConnected(true);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this._$Do?.setConnected(false);
  }
  render() {
    return T;
  }
};
i$1._$litElement$ = true, i$1["finalized"] = true, s$1.litElementHydrateSupport?.({ LitElement: i$1 });
const o$3 = s$1.litElementPolyfillSupport;
o$3?.({ LitElement: i$1 });
(s$1.litElementVersions ??= []).push("4.2.1");
const t$1 = (t2) => (e2, o2) => {
  void 0 !== o2 ? o2.addInitializer((() => {
    customElements.define(t2, e2);
  })) : customElements.define(t2, e2);
};
const o$2 = { attribute: true, type: String, converter: u$1, reflect: false, hasChanged: f$3 }, r$1 = (t2 = o$2, e2, r2) => {
  const { kind: n3, metadata: i3 } = r2;
  let s2 = globalThis.litPropertyMetadata.get(i3);
  if (void 0 === s2 && globalThis.litPropertyMetadata.set(i3, s2 = /* @__PURE__ */ new Map()), "setter" === n3 && ((t2 = Object.create(t2)).wrapped = true), s2.set(r2.name, t2), "accessor" === n3) {
    const { name: o2 } = r2;
    return { set(r3) {
      const n4 = e2.get.call(this);
      e2.set.call(this, r3), this.requestUpdate(o2, n4, t2);
    }, init(e3) {
      return void 0 !== e3 && this.C(o2, void 0, t2, e3), e3;
    } };
  }
  if ("setter" === n3) {
    const { name: o2 } = r2;
    return function(r3) {
      const n4 = this[o2];
      e2.call(this, r3), this.requestUpdate(o2, n4, t2);
    };
  }
  throw Error("Unsupported decorator location: " + n3);
};
function n$2(t2) {
  return (e2, o2) => "object" == typeof o2 ? r$1(t2, e2, o2) : ((t3, e3, o3) => {
    const r2 = e3.hasOwnProperty(o3);
    return e3.constructor.createProperty(o3, t3), r2 ? Object.getOwnPropertyDescriptor(e3, o3) : void 0;
  })(t2, e2, o2);
}
const f$1 = (o2) => void 0 === o2.strings;
const t = { CHILD: 2 }, e$1 = (t2) => (...e2) => ({ _$litDirective$: t2, values: e2 });
class i2 {
  constructor(t2) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(t2, e2, i3) {
    this._$Ct = t2, this._$AM = e2, this._$Ci = i3;
  }
  _$AS(t2, e2) {
    return this.update(t2, e2);
  }
  update(t2, e2) {
    return this.render(...e2);
  }
}
const s = (i3, t2) => {
  const e2 = i3._$AN;
  if (void 0 === e2) return false;
  for (const i4 of e2) i4._$AO?.(t2, false), s(i4, t2);
  return true;
}, o$1 = (i3) => {
  let t2, e2;
  do {
    if (void 0 === (t2 = i3._$AM)) break;
    e2 = t2._$AN, e2.delete(i3), i3 = t2;
  } while (0 === e2?.size);
}, r = (i3) => {
  for (let t2; t2 = i3._$AM; i3 = t2) {
    let e2 = t2._$AN;
    if (void 0 === e2) t2._$AN = e2 = /* @__PURE__ */ new Set();
    else if (e2.has(i3)) break;
    e2.add(i3), c(t2);
  }
};
function h$1(i3) {
  void 0 !== this._$AN ? (o$1(this), this._$AM = i3, r(this)) : this._$AM = i3;
}
function n$1(i3, t2 = false, e2 = 0) {
  const r2 = this._$AH, h2 = this._$AN;
  if (void 0 !== h2 && 0 !== h2.size) if (t2) if (Array.isArray(r2)) for (let i4 = e2; i4 < r2.length; i4++) s(r2[i4], false), o$1(r2[i4]);
  else null != r2 && (s(r2, false), o$1(r2));
  else s(this, i3);
}
const c = (i3) => {
  i3.type == t.CHILD && (i3._$AP ??= n$1, i3._$AQ ??= h$1);
};
class f extends i2 {
  constructor() {
    super(...arguments), this._$AN = void 0;
  }
  _$AT(i3, t2, e2) {
    super._$AT(i3, t2, e2), r(this), this.isConnected = i3._$AU;
  }
  _$AO(i3, t2 = true) {
    i3 !== this.isConnected && (this.isConnected = i3, i3 ? this.reconnected?.() : this.disconnected?.()), t2 && (s(this, i3), o$1(this));
  }
  setValue(t2) {
    if (f$1(this._$Ct)) this._$Ct._$AI(t2, this);
    else {
      const i3 = [...this._$Ct._$AH];
      i3[this._$Ci] = t2, this._$Ct._$AI(i3, this, 0);
    }
  }
  disconnected() {
  }
  reconnected() {
  }
}
const e = () => new h();
class h {
}
const o = /* @__PURE__ */ new WeakMap(), n2 = e$1(class extends f {
  render(i3) {
    return E;
  }
  update(i3, [s2]) {
    const e2 = s2 !== this.G;
    return e2 && void 0 !== this.G && this.rt(void 0), (e2 || this.lt !== this.ct) && (this.G = s2, this.ht = i3.options?.host, this.rt(this.ct = i3.element)), E;
  }
  rt(t2) {
    if (this.isConnected || (t2 = void 0), "function" == typeof this.G) {
      const i3 = this.ht ?? globalThis;
      let s2 = o.get(i3);
      void 0 === s2 && (s2 = /* @__PURE__ */ new WeakMap(), o.set(i3, s2)), void 0 !== s2.get(this.G) && this.G.call(this.ht, void 0), s2.set(this.G, t2), void 0 !== t2 && this.G.call(this.ht, t2);
    } else this.G.value = t2;
  }
  get lt() {
    return "function" == typeof this.G ? o.get(this.ht ?? globalThis)?.get(this.G) : this.G?.value;
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
const SVG_ICON_CALENDAR = b`<!-- Adapted from https://iconoir.com/ -->
<svg width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon">
  <path d="M15 4V2M15 4V6M15 4H10.5M3 10V19C3 20.1046 3.89543 21 5 21H19C20.1046 21 21 20.1046 21 19V10H3Z" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M3 10V6C3 4.89543 3.89543 4 5 4H7" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M7 2V6" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M21 10V6C21 4.89543 20.1046 4 19 4H18.5" stroke-linecap="round" stroke-linejoin="round"></path>
</svg>
`;
const SVG_ICON_INTERNET = b`<!-- Adapted from https://iconoir.com/ -->
<svg class="icon" width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M13 2.04932C13 2.04932 16 5.99994 16 11.9999" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M11 21.9506C11 21.9506 8 17.9999 8 11.9999C8 5.99994 11 2.04932 11 2.04932" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 15.5H12" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 8.5H21.3704" stroke-linecap="round" stroke-linejoin="round"></path>
  <path fill-rule="evenodd" clip-rule="evenodd" d="M21.8789 17.9174C22.3727 18.2211 22.3423 18.9604 21.8337 19.0181L19.2671 19.309L18.1159 21.6213C17.8878 22.0795 17.1827 21.8552 17.0661 21.2873L15.8108 15.1713C15.7123 14.6913 16.1437 14.3892 16.561 14.646L21.8789 17.9174Z"></path>
</svg>
`;
var __defProp$7 = Object.defineProperty;
var __getOwnPropDesc$7 = Object.getOwnPropertyDescriptor;
var __decorateClass$7 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$7(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$7(target, key, result);
  return result;
};
let CreateAddressbookForm = class extends i$1 {
  constructor() {
    super();
    this.user = "";
    this.principal = "";
    this.addr_id = self.crypto.randomUUID();
    this.displayname = "";
    this.description = "";
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Create addressbook</button>
      <dialog ${n2(this.dialog)}>
        <h3>Create addressbook</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            principal (for group addressbooks)
            <select .value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" .value=${this.addr_id} @change=${(e2) => this.addr_id = e2.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
        </form>
      </dialog>
    `;
  }
  async submit(e2) {
    console.log(this.displayname);
    e2.preventDefault();
    if (!this.addr_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    let response = await fetch(`/carddav/principal/${this.principal || this.user}/${this.addr_id}`, {
      method: "MKCOL",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <mkcol xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CARD:addressbook-description>${escapeXml(this.description)}</CARD:addressbook-description>` : ""}
          </prop>
        </set>
      </mkcol>
      `
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$7([
  n$2()
], CreateAddressbookForm.prototype, "user", 2);
__decorateClass$7([
  n$2()
], CreateAddressbookForm.prototype, "principal", 2);
__decorateClass$7([
  n$2()
], CreateAddressbookForm.prototype, "addr_id", 2);
__decorateClass$7([
  n$2()
], CreateAddressbookForm.prototype, "displayname", 2);
__decorateClass$7([
  n$2()
], CreateAddressbookForm.prototype, "description", 2);
CreateAddressbookForm = __decorateClass$7([
  t$1("create-addressbook-form")
], CreateAddressbookForm);
var __defProp$6 = Object.defineProperty;
var __getOwnPropDesc$6 = Object.getOwnPropertyDescriptor;
var __decorateClass$6 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$6(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$6(target, key, result);
  return result;
};
let CreateBirthdayCalendarForm = class extends i$1 {
  constructor() {
    super(...arguments);
    this.principal = "";
    this.addr_id = "";
    this.displayname = "";
    this.description = "";
    this.color = "";
    this.dialog = e();
    this.form = e();
    this.timezones = [];
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Create birthday calendar</button>
      <dialog ${n2(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} required @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e2) => this.color = e2.target.value} />
          </label>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `;
  }
  async submit(e2) {
    e2.preventDefault();
    if (!this.addr_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    let response = await fetch(`/caldav/principal/${this.principal}/_birthdays_${this.addr_id}`, {
      method: "MKCOL",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
            <CAL:supported-calendar-component-set>
              <CAL:comp name="VEVENT" />
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
      </mkcol>
      `
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "principal", 2);
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "addr_id", 2);
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "displayname", 2);
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "description", 2);
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "color", 2);
__decorateClass$6([
  n$2()
], CreateBirthdayCalendarForm.prototype, "timezones", 2);
CreateBirthdayCalendarForm = __decorateClass$6([
  t$1("create-birthday-calendar-form")
], CreateBirthdayCalendarForm);
let timezonesPromise = null;
async function getTimezones() {
  timezonesPromise ||= new Promise(async (resolve, reject) => {
    try {
      let response = await fetch("/frontend/_timezones.json");
      resolve(await response.json());
    } catch (e2) {
      reject(e2);
    }
  });
  return await timezonesPromise;
}
var __defProp$5 = Object.defineProperty;
var __getOwnPropDesc$5 = Object.getOwnPropertyDescriptor;
var __decorateClass$5 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$5(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$5(target, key, result);
  return result;
};
let CreateCalendarForm = class extends i$1 {
  constructor() {
    super();
    this.user = "";
    this.dialog = e();
    this.form = e();
    this.timezones = [];
    this.resetForm();
    this.fetchTimezones();
  }
  resetForm() {
    this.form.value?.reset();
    this.principal = this.user;
    this.cal_id = self.crypto.randomUUID();
    this.displayname = "";
    this.description = "";
    this.timezone_id = "";
    this.color = "";
    this.isSubscription = false;
    this.subscriptionUrl = null;
    this.components = /* @__PURE__ */ new Set(["VEVENT", "VTODO"]);
  }
  async fetchTimezones() {
    this.timezones = await getTimezones();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${(e2) => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${n2(this.dialog)} @close=${(e2) => this.resetForm()}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            principal (for group calendars)
            <select required value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${(e2) => this.cal_id = e2.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${(e2) => this.timezone_id = e2.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map((timezone) => x`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e2) => this.color = e2.target.value} />
          </label>
          <br>
          <br>
          <label>Type</label>
          <div class="tab-radio">
            <label>
              <input type="radio" name="type" .checked=${!this.isSubscription} @change=${(e2) => this.isSubscription = false}></input>
              ${SVG_ICON_CALENDAR}
              Calendar
            </label>
            <label>
              <input type="radio" name="type" .checked=${this.isSubscription} @change=${(e2) => this.isSubscription = true}></input>
              ${SVG_ICON_INTERNET}
              webCal Subscription
            </label>
          </div>
          <br>
          ${this.isSubscription ? x`
            <label>
              Subscription URL
              <input type="text" pattern="https://.*" .required=${this.isSubscription} .value=${this.subscriptionUrl} @change=${(e2) => this.subscriptionUrl = e2.target.value}  />
            </label>
            <br>
            <br>
          ` : x``}

          <label>Components</label>
          <div>
            ${["VEVENT", "VTODO", "VJOURNAL"].map((comp) => x`
              <label>
                Support ${comp}
                <input type="checkbox" .value=${comp} @change=${(e2) => e2.target.checked ? this.components.add(e2.target.value) : this.components.delete(e2.target.value)} .checked=${this.components.has(comp)} />
              </label>
              <br>
            `)}
          </div>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
    }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `;
  }
  async submit(e2) {
    e2.preventDefault();
    if (!this.cal_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    if (!this.components.size) {
      alert("No calendar components selected");
      return;
    }
    if (this.isSubscription && !this.subscriptionUrl) {
      alert("Invalid subscription url");
      return;
    }
    let response = await fetch(`/caldav/principal/${this.principal || this.user}/${this.cal_id}`, {
      method: "MKCOL",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ""}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
            ${this.isSubscription && this.subscriptionUrl ? `<CS:source><href>${escapeXml(this.subscriptionUrl)}</href></CS:source>` : ""}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map((comp) => `<CAL:comp name="${escapeXml(comp)}" />`).join("\n")}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
      </mkcol>
      `
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "user", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "principal", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "cal_id", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "displayname", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "description", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "timezone_id", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "color", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "isSubscription", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "subscriptionUrl", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "components", 2);
__decorateClass$5([
  n$2()
], CreateCalendarForm.prototype, "timezones", 2);
CreateCalendarForm = __decorateClass$5([
  t$1("create-calendar-form")
], CreateCalendarForm);
var __defProp$4 = Object.defineProperty;
var __getOwnPropDesc$4 = Object.getOwnPropertyDescriptor;
var __decorateClass$4 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$4(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$4(target, key, result);
  return result;
};
let DeleteButton = class extends i$1 {
  constructor() {
    super();
    this.trash = false;
  }
  createRenderRoot() {
    return this;
  }
  render() {
    let text = this.trash ? "Trash" : "Delete";
    return x`<button class="delete" @click=${(e2) => this._onClick(e2)}>${text}</button>`;
  }
  async _onClick(event) {
    event.preventDefault();
    if (!this.trash && !confirm("Do you want to delete this collection permanently?")) {
      return;
    }
    let response = await fetch(this.href, {
      method: "DELETE",
      headers: {
        "X-No-Trashbin": this.trash ? "0" : "1"
      }
    });
    if (response.status < 200 || response.status >= 300) {
      alert("An error occured, look into the console");
      console.error(response);
      return;
    }
    window.location.reload();
  }
};
__decorateClass$4([
  n$2({ type: Boolean })
], DeleteButton.prototype, "trash", 2);
__decorateClass$4([
  n$2()
], DeleteButton.prototype, "href", 2);
DeleteButton = __decorateClass$4([
  t$1("delete-button")
], DeleteButton);
var __defProp$3 = Object.defineProperty;
var __getOwnPropDesc$3 = Object.getOwnPropertyDescriptor;
var __decorateClass$3 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$3(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$3(target, key, result);
  return result;
};
let EditAddressbookForm = class extends i$1 {
  constructor() {
    super();
    this.principal = "";
    this.addr_id = "";
    this.displayname = "";
    this.description = "";
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${n2(this.dialog)}>
        <h3>Edit addressbook</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <button type="submit">Submit</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
        </form>
      </dialog>
    `;
  }
  async submit(e2) {
    e2.preventDefault();
    if (!this.principal) {
      alert("Empty principal");
      return;
    }
    if (!this.addr_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    let response = await fetch(`/carddav/principal/${this.principal}/${this.addr_id}`, {
      method: "PROPPATCH",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CARD:addressbook-description>${escapeXml(this.description)}</CARD:addressbook-description>` : ""}
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.description ? "<CARD:calendar-description />" : ""}
          </prop>
        </remove>
      </propertyupdate>
      `
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$3([
  n$2()
], EditAddressbookForm.prototype, "principal", 2);
__decorateClass$3([
  n$2()
], EditAddressbookForm.prototype, "addr_id", 2);
__decorateClass$3([
  n$2()
], EditAddressbookForm.prototype, "displayname", 2);
__decorateClass$3([
  n$2()
], EditAddressbookForm.prototype, "description", 2);
EditAddressbookForm = __decorateClass$3([
  t$1("edit-addressbook-form")
], EditAddressbookForm);
var __defProp$2 = Object.defineProperty;
var __getOwnPropDesc$2 = Object.getOwnPropertyDescriptor;
var __decorateClass$2 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$2(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$2(target, key, result);
  return result;
};
let EditCalendarForm = class extends i$1 {
  constructor() {
    super();
    this.displayname = "";
    this.description = "";
    this.timezone_id = "";
    this.color = "";
    this.components = /* @__PURE__ */ new Set();
    this.dialog = e();
    this.form = e();
    this.timezones = [];
    this.fetchTimezones();
  }
  async fetchTimezones() {
    this.timezones = await getTimezones();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${n2(this.dialog)}>
        <h3>Edit calendar</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${(e2) => this.timezone_id = e2.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map((timezone) => x`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e2) => this.color = e2.target.value} />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map((comp) => x`
            <label>
              Support ${comp}
              <input type="checkbox" .value=${comp} ?checked=${this.components.has(comp)} @change=${(e2) => e2.target.checked ? this.components.add(e2.target.value) : this.components.delete(e2.target.value)} />
            </label>
            <br>
          `)}
          <br>
          <button type="submit">Submit</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `;
  }
  async submit(e2) {
    e2.preventDefault();
    if (!this.principal) {
      alert("Empty principal");
      return;
    }
    if (!this.cal_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    if (!this.components.size) {
      alert("No calendar components selected");
      return;
    }
    let response = await fetch(`/caldav/principal/${this.principal}/${this.cal_id}`, {
      method: "PROPPATCH",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ""}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map((comp) => `<CAL:comp name="${escapeXml(comp)}" />`).join("\n")}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.timezone_id ? `<CAL:calendar-timezone-id />` : ""}
            ${!this.description ? "<CAL:calendar-description />" : ""}
            ${!this.color ? "<ICAL:calendar-color />" : ""}
          </prop>
        </remove>
      </propertyupdate>
      `
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "principal", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "cal_id", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "displayname", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "description", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "timezone_id", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "color", 2);
__decorateClass$2([
  n$2({
    converter: {
      fromAttribute: (value, _type) => new Set(value ? JSON.parse(value) : []),
      toAttribute: (value, _type) => JSON.stringify(value)
    }
  })
], EditCalendarForm.prototype, "components", 2);
__decorateClass$2([
  n$2()
], EditCalendarForm.prototype, "timezones", 2);
EditCalendarForm = __decorateClass$2([
  t$1("edit-calendar-form")
], EditCalendarForm);
var __defProp$1 = Object.defineProperty;
var __getOwnPropDesc$1 = Object.getOwnPropertyDescriptor;
var __decorateClass$1 = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc$1(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp$1(target, key, result);
  return result;
};
let ImportAddressbookForm = class extends i$1 {
  constructor() {
    super();
    this.user = "";
    this.addressbook_id = self.crypto.randomUUID();
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Import addressbook</button>
      <dialog ${n2(this.dialog)}>
        <h3>Import addressbook</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            principal (for group addressbook)
            <select name="principal" required .value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.addressbook_id} @change=${(e2) => this.addressbook_id = e2.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" accept="text/vcard" required @change=${(e2) => this.file = e2.target.files[0]} />
          </label>
          <br>
          <br>
          <button type="submit">Import</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `;
  }
  async submit(e2) {
    e2.preventDefault();
    this.principal ||= this.user;
    if (!this.principal) {
      alert("Empty principal");
      return;
    }
    if (!this.addressbook_id) {
      alert("Empty id");
      return;
    }
    let response = await fetch(`/carddav/principal/${this.principal}/${this.addressbook_id}`, {
      method: "IMPORT",
      headers: {
        "Content-Type": "text/vcard"
      },
      body: this.file
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass$1([
  n$2()
], ImportAddressbookForm.prototype, "user", 2);
__decorateClass$1([
  n$2()
], ImportAddressbookForm.prototype, "principal", 2);
__decorateClass$1([
  n$2()
], ImportAddressbookForm.prototype, "addressbook_id", 2);
ImportAddressbookForm = __decorateClass$1([
  t$1("import-addressbook-form")
], ImportAddressbookForm);
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i3 = decorators.length - 1, decorator; i3 >= 0; i3--)
    if (decorator = decorators[i3])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
let ImportCalendarForm = class extends i$1 {
  constructor() {
    super();
    this.user = "";
    this.cal_id = self.crypto.randomUUID();
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Import calendar</button>
      <dialog ${n2(this.dialog)}>
        <h3>Import calendar</h3>
        <form @submit=${this.submit} ${n2(this.form)}>
          <label>
            principal (for group calendars)
            <select name="principal" required .value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${(e2) => this.cal_id = e2.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" required accept="text/calendar" @change=${(e2) => this.file = e2.target.files[0]} />
          </label>
          <br>
          <br>
          <button type="submit">Import</button>
          <button type="submit" @click=${(event) => {
      event.preventDefault();
      this.dialog.value.close();
      this.form.value.reset();
    }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `;
  }
  async submit(e2) {
    e2.preventDefault();
    this.principal ||= this.user;
    if (!this.principal) {
      alert("Empty principal");
      return;
    }
    if (!this.cal_id) {
      alert("Empty id");
      return;
    }
    let response = await fetch(`/caldav/principal/${this.principal}/${this.cal_id}`, {
      method: "IMPORT",
      headers: {
        "Content-Type": "text/calendar"
      },
      body: this.file
    });
    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`);
      return null;
    }
    window.location.reload();
    return null;
  }
};
__decorateClass([
  n$2()
], ImportCalendarForm.prototype, "user", 2);
__decorateClass([
  n$2()
], ImportCalendarForm.prototype, "principal", 2);
__decorateClass([
  n$2()
], ImportCalendarForm.prototype, "cal_id", 2);
ImportCalendarForm = __decorateClass([
  t$1("import-calendar-form")
], ImportCalendarForm);
export {
  CreateAddressbookForm,
  CreateBirthdayCalendarForm,
  CreateCalendarForm,
  DeleteButton,
  EditAddressbookForm,
  EditCalendarForm,
  ImportAddressbookForm,
  ImportCalendarForm
};
