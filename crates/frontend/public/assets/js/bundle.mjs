//#region node_modules/.deno/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/css-tag.js
/**
* @license
* Copyright 2019 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/
var t$4 = globalThis, e$5 = t$4.ShadowRoot && (void 0 === t$4.ShadyCSS || t$4.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype, s$4 = Symbol(), o$6 = /* @__PURE__ */ new WeakMap();
var n$6 = class {
	constructor(t, e, o) {
		if (this._$cssResult$ = !0, o !== s$4) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
		this.cssText = t, this.t = e;
	}
	get styleSheet() {
		let t = this.o;
		const s = this.t;
		if (e$5 && void 0 === t) {
			const e = void 0 !== s && 1 === s.length;
			e && (t = o$6.get(s)), void 0 === t && ((this.o = t = new CSSStyleSheet()).replaceSync(this.cssText), e && o$6.set(s, t));
		}
		return t;
	}
	toString() {
		return this.cssText;
	}
}, r$5 = (t) => new n$6("string" == typeof t ? t : t + "", void 0, s$4), S$1 = (s, o) => {
	if (e$5) s.adoptedStyleSheets = o.map((t) => t instanceof CSSStyleSheet ? t : t.styleSheet);
	else for (const e of o) {
		const o = document.createElement("style"), n = t$4.litNonce;
		void 0 !== n && o.setAttribute("nonce", n), o.textContent = e.cssText, s.appendChild(o);
	}
}, c$4 = e$5 ? (t) => t : (t) => t instanceof CSSStyleSheet ? ((t) => {
	let e = "";
	for (const s of t.cssRules) e += s.cssText;
	return r$5(e);
})(t) : t;
//#endregion
//#region node_modules/.deno/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/reactive-element.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var { is: i$4, defineProperty: e$4, getOwnPropertyDescriptor: h$4, getOwnPropertyNames: r$4, getOwnPropertySymbols: o$5, getPrototypeOf: n$5 } = Object, a$1 = globalThis, c$3 = a$1.trustedTypes, l$2 = c$3 ? c$3.emptyScript : "", p$2 = a$1.reactiveElementPolyfillSupport, d$2 = (t, s) => t, u$2 = {
	toAttribute(t, s) {
		switch (s) {
			case Boolean:
				t = t ? l$2 : null;
				break;
			case Object:
			case Array: t = null == t ? t : JSON.stringify(t);
		}
		return t;
	},
	fromAttribute(t, s) {
		let i = t;
		switch (s) {
			case Boolean:
				i = null !== t;
				break;
			case Number:
				i = null === t ? null : Number(t);
				break;
			case Object:
			case Array: try {
				i = JSON.parse(t);
			} catch (t) {
				i = null;
			}
		}
		return i;
	}
}, f$3 = (t, s) => !i$4(t, s), b$1 = {
	attribute: !0,
	type: String,
	converter: u$2,
	reflect: !1,
	useDefault: !1,
	hasChanged: f$3
};
Symbol.metadata ??= Symbol("metadata"), a$1.litPropertyMetadata ??= /* @__PURE__ */ new WeakMap();
var y$1 = class extends HTMLElement {
	static addInitializer(t) {
		this._$Ei(), (this.l ??= []).push(t);
	}
	static get observedAttributes() {
		return this.finalize(), this._$Eh && [...this._$Eh.keys()];
	}
	static createProperty(t, s = b$1) {
		if (s.state && (s.attribute = !1), this._$Ei(), this.prototype.hasOwnProperty(t) && ((s = Object.create(s)).wrapped = !0), this.elementProperties.set(t, s), !s.noAccessor) {
			const i = Symbol(), h = this.getPropertyDescriptor(t, i, s);
			void 0 !== h && e$4(this.prototype, t, h);
		}
	}
	static getPropertyDescriptor(t, s, i) {
		const { get: e, set: r } = h$4(this.prototype, t) ?? {
			get() {
				return this[s];
			},
			set(t) {
				this[s] = t;
			}
		};
		return {
			get: e,
			set(s) {
				const h = e?.call(this);
				r?.call(this, s), this.requestUpdate(t, h, i);
			},
			configurable: !0,
			enumerable: !0
		};
	}
	static getPropertyOptions(t) {
		return this.elementProperties.get(t) ?? b$1;
	}
	static _$Ei() {
		if (this.hasOwnProperty(d$2("elementProperties"))) return;
		const t = n$5(this);
		t.finalize(), void 0 !== t.l && (this.l = [...t.l]), this.elementProperties = new Map(t.elementProperties);
	}
	static finalize() {
		if (this.hasOwnProperty(d$2("finalized"))) return;
		if (this.finalized = !0, this._$Ei(), this.hasOwnProperty(d$2("properties"))) {
			const t = this.properties, s = [...r$4(t), ...o$5(t)];
			for (const i of s) this.createProperty(i, t[i]);
		}
		const t = this[Symbol.metadata];
		if (null !== t) {
			const s = litPropertyMetadata.get(t);
			if (void 0 !== s) for (const [t, i] of s) this.elementProperties.set(t, i);
		}
		this._$Eh = /* @__PURE__ */ new Map();
		for (const [t, s] of this.elementProperties) {
			const i = this._$Eu(t, s);
			void 0 !== i && this._$Eh.set(i, t);
		}
		this.elementStyles = this.finalizeStyles(this.styles);
	}
	static finalizeStyles(s) {
		const i = [];
		if (Array.isArray(s)) {
			const e = new Set(s.flat(Infinity).reverse());
			for (const s of e) i.unshift(c$4(s));
		} else void 0 !== s && i.push(c$4(s));
		return i;
	}
	static _$Eu(t, s) {
		const i = s.attribute;
		return !1 === i ? void 0 : "string" == typeof i ? i : "string" == typeof t ? t.toLowerCase() : void 0;
	}
	constructor() {
		super(), this._$Ep = void 0, this.isUpdatePending = !1, this.hasUpdated = !1, this._$Em = null, this._$Ev();
	}
	_$Ev() {
		this._$ES = new Promise((t) => this.enableUpdating = t), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t) => t(this));
	}
	addController(t) {
		(this._$EO ??= /* @__PURE__ */ new Set()).add(t), void 0 !== this.renderRoot && this.isConnected && t.hostConnected?.();
	}
	removeController(t) {
		this._$EO?.delete(t);
	}
	_$E_() {
		const t = /* @__PURE__ */ new Map(), s = this.constructor.elementProperties;
		for (const i of s.keys()) this.hasOwnProperty(i) && (t.set(i, this[i]), delete this[i]);
		t.size > 0 && (this._$Ep = t);
	}
	createRenderRoot() {
		const t = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
		return S$1(t, this.constructor.elementStyles), t;
	}
	connectedCallback() {
		this.renderRoot ??= this.createRenderRoot(), this.enableUpdating(!0), this._$EO?.forEach((t) => t.hostConnected?.());
	}
	enableUpdating(t) {}
	disconnectedCallback() {
		this._$EO?.forEach((t) => t.hostDisconnected?.());
	}
	attributeChangedCallback(t, s, i) {
		this._$AK(t, i);
	}
	_$ET(t, s) {
		const i = this.constructor.elementProperties.get(t), e = this.constructor._$Eu(t, i);
		if (void 0 !== e && !0 === i.reflect) {
			const h = (void 0 !== i.converter?.toAttribute ? i.converter : u$2).toAttribute(s, i.type);
			this._$Em = t, null == h ? this.removeAttribute(e) : this.setAttribute(e, h), this._$Em = null;
		}
	}
	_$AK(t, s) {
		const i = this.constructor, e = i._$Eh.get(t);
		if (void 0 !== e && this._$Em !== e) {
			const t = i.getPropertyOptions(e), h = "function" == typeof t.converter ? { fromAttribute: t.converter } : void 0 !== t.converter?.fromAttribute ? t.converter : u$2;
			this._$Em = e;
			const r = h.fromAttribute(s, t.type);
			this[e] = r ?? this._$Ej?.get(e) ?? r, this._$Em = null;
		}
	}
	requestUpdate(t, s, i, e = !1, h) {
		if (void 0 !== t) {
			const r = this.constructor;
			if (!1 === e && (h = this[t]), i ??= r.getPropertyOptions(t), !((i.hasChanged ?? f$3)(h, s) || i.useDefault && i.reflect && h === this._$Ej?.get(t) && !this.hasAttribute(r._$Eu(t, i)))) return;
			this.C(t, s, i);
		}
		!1 === this.isUpdatePending && (this._$ES = this._$EP());
	}
	C(t, s, { useDefault: i, reflect: e, wrapped: h }, r) {
		i && !(this._$Ej ??= /* @__PURE__ */ new Map()).has(t) && (this._$Ej.set(t, r ?? s ?? this[t]), !0 !== h || void 0 !== r) || (this._$AL.has(t) || (this.hasUpdated || i || (s = void 0), this._$AL.set(t, s)), !0 === e && this._$Em !== t && (this._$Eq ??= /* @__PURE__ */ new Set()).add(t));
	}
	async _$EP() {
		this.isUpdatePending = !0;
		try {
			await this._$ES;
		} catch (t) {
			Promise.reject(t);
		}
		const t = this.scheduleUpdate();
		return null != t && await t, !this.isUpdatePending;
	}
	scheduleUpdate() {
		return this.performUpdate();
	}
	performUpdate() {
		if (!this.isUpdatePending) return;
		if (!this.hasUpdated) {
			if (this.renderRoot ??= this.createRenderRoot(), this._$Ep) {
				for (const [t, s] of this._$Ep) this[t] = s;
				this._$Ep = void 0;
			}
			const t = this.constructor.elementProperties;
			if (t.size > 0) for (const [s, i] of t) {
				const { wrapped: t } = i, e = this[s];
				!0 !== t || this._$AL.has(s) || void 0 === e || this.C(s, void 0, i, e);
			}
		}
		let t = !1;
		const s = this._$AL;
		try {
			t = this.shouldUpdate(s), t ? (this.willUpdate(s), this._$EO?.forEach((t) => t.hostUpdate?.()), this.update(s)) : this._$EM();
		} catch (s) {
			throw t = !1, this._$EM(), s;
		}
		t && this._$AE(s);
	}
	willUpdate(t) {}
	_$AE(t) {
		this._$EO?.forEach((t) => t.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = !0, this.firstUpdated(t)), this.updated(t);
	}
	_$EM() {
		this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = !1;
	}
	get updateComplete() {
		return this.getUpdateComplete();
	}
	getUpdateComplete() {
		return this._$ES;
	}
	shouldUpdate(t) {
		return !0;
	}
	update(t) {
		this._$Eq &&= this._$Eq.forEach((t) => this._$ET(t, this[t])), this._$EM();
	}
	updated(t) {}
	firstUpdated(t) {}
};
y$1.elementStyles = [], y$1.shadowRootOptions = { mode: "open" }, y$1[d$2("elementProperties")] = /* @__PURE__ */ new Map(), y$1[d$2("finalized")] = /* @__PURE__ */ new Map(), p$2?.({ ReactiveElement: y$1 }), (a$1.reactiveElementVersions ??= []).push("2.1.2");
//#endregion
//#region node_modules/.deno/lit-html@3.3.2/node_modules/lit-html/lit-html.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/
var t$3 = globalThis, i$3 = (t) => t, s$3 = t$3.trustedTypes, e$3 = s$3 ? s$3.createPolicy("lit-html", { createHTML: (t) => t }) : void 0, h$3 = "$lit$", o$4 = `lit$${Math.random().toFixed(9).slice(2)}$`, n$4 = "?" + o$4, r$3 = `<${n$4}>`, l$1 = document, c$2 = () => l$1.createComment(""), a = (t) => null === t || "object" != typeof t && "function" != typeof t, u$1 = Array.isArray, d$1 = (t) => u$1(t) || "function" == typeof t?.[Symbol.iterator], f$2 = "[ 	\n\f\r]", v$1 = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g, _ = /-->/g, m$1 = />/g, p$1 = RegExp(`>|${f$2}(?:([^\\s"'>=/]+)(${f$2}*=${f$2}*(?:[^ \t\n\f\r"'\`<>=]|("|')|))|$)`, "g"), g = /'/g, $ = /"/g, y = /^(?:script|style|textarea|title)$/i, x = (t) => (i, ...s) => ({
	_$litType$: t,
	strings: i,
	values: s
}), b = x(1), w = x(2);
x(3);
var E = Symbol.for("lit-noChange"), A = Symbol.for("lit-nothing"), C = /* @__PURE__ */ new WeakMap(), P = l$1.createTreeWalker(l$1, 129);
function V(t, i) {
	if (!u$1(t) || !t.hasOwnProperty("raw")) throw Error("invalid template strings array");
	return void 0 !== e$3 ? e$3.createHTML(i) : i;
}
var N = (t, i) => {
	const s = t.length - 1, e = [];
	let n, l = 2 === i ? "<svg>" : 3 === i ? "<math>" : "", c = v$1;
	for (let i = 0; i < s; i++) {
		const s = t[i];
		let a, u, d = -1, f = 0;
		for (; f < s.length && (c.lastIndex = f, u = c.exec(s), null !== u);) f = c.lastIndex, c === v$1 ? "!--" === u[1] ? c = _ : void 0 !== u[1] ? c = m$1 : void 0 !== u[2] ? (y.test(u[2]) && (n = RegExp("</" + u[2], "g")), c = p$1) : void 0 !== u[3] && (c = p$1) : c === p$1 ? ">" === u[0] ? (c = n ?? v$1, d = -1) : void 0 === u[1] ? d = -2 : (d = c.lastIndex - u[2].length, a = u[1], c = void 0 === u[3] ? p$1 : "\"" === u[3] ? $ : g) : c === $ || c === g ? c = p$1 : c === _ || c === m$1 ? c = v$1 : (c = p$1, n = void 0);
		const x = c === p$1 && t[i + 1].startsWith("/>") ? " " : "";
		l += c === v$1 ? s + r$3 : d >= 0 ? (e.push(a), s.slice(0, d) + h$3 + s.slice(d) + o$4 + x) : s + o$4 + (-2 === d ? i : x);
	}
	return [V(t, l + (t[s] || "<?>") + (2 === i ? "</svg>" : 3 === i ? "</math>" : "")), e];
};
var S = class S {
	constructor({ strings: t, _$litType$: i }, e) {
		let r;
		this.parts = [];
		let l = 0, a = 0;
		const u = t.length - 1, d = this.parts, [f, v] = N(t, i);
		if (this.el = S.createElement(f, e), P.currentNode = this.el.content, 2 === i || 3 === i) {
			const t = this.el.content.firstChild;
			t.replaceWith(...t.childNodes);
		}
		for (; null !== (r = P.nextNode()) && d.length < u;) {
			if (1 === r.nodeType) {
				if (r.hasAttributes()) for (const t of r.getAttributeNames()) if (t.endsWith(h$3)) {
					const i = v[a++], s = r.getAttribute(t).split(o$4), e = /([.?@])?(.*)/.exec(i);
					d.push({
						type: 1,
						index: l,
						name: e[2],
						strings: s,
						ctor: "." === e[1] ? I : "?" === e[1] ? L : "@" === e[1] ? z : H
					}), r.removeAttribute(t);
				} else t.startsWith(o$4) && (d.push({
					type: 6,
					index: l
				}), r.removeAttribute(t));
				if (y.test(r.tagName)) {
					const t = r.textContent.split(o$4), i = t.length - 1;
					if (i > 0) {
						r.textContent = s$3 ? s$3.emptyScript : "";
						for (let s = 0; s < i; s++) r.append(t[s], c$2()), P.nextNode(), d.push({
							type: 2,
							index: ++l
						});
						r.append(t[i], c$2());
					}
				}
			} else if (8 === r.nodeType) if (r.data === n$4) d.push({
				type: 2,
				index: l
			});
			else {
				let t = -1;
				for (; -1 !== (t = r.data.indexOf(o$4, t + 1));) d.push({
					type: 7,
					index: l
				}), t += o$4.length - 1;
			}
			l++;
		}
	}
	static createElement(t, i) {
		const s = l$1.createElement("template");
		return s.innerHTML = t, s;
	}
};
function M$1(t, i, s = t, e) {
	if (i === E) return i;
	let h = void 0 !== e ? s._$Co?.[e] : s._$Cl;
	const o = a(i) ? void 0 : i._$litDirective$;
	return h?.constructor !== o && (h?._$AO?.(!1), void 0 === o ? h = void 0 : (h = new o(t), h._$AT(t, s, e)), void 0 !== e ? (s._$Co ??= [])[e] = h : s._$Cl = h), void 0 !== h && (i = M$1(t, h._$AS(t, i.values), h, e)), i;
}
var R = class {
	constructor(t, i) {
		this._$AV = [], this._$AN = void 0, this._$AD = t, this._$AM = i;
	}
	get parentNode() {
		return this._$AM.parentNode;
	}
	get _$AU() {
		return this._$AM._$AU;
	}
	u(t) {
		const { el: { content: i }, parts: s } = this._$AD, e = (t?.creationScope ?? l$1).importNode(i, !0);
		P.currentNode = e;
		let h = P.nextNode(), o = 0, n = 0, r = s[0];
		for (; void 0 !== r;) {
			if (o === r.index) {
				let i;
				2 === r.type ? i = new k(h, h.nextSibling, this, t) : 1 === r.type ? i = new r.ctor(h, r.name, r.strings, this, t) : 6 === r.type && (i = new Z(h, this, t)), this._$AV.push(i), r = s[++n];
			}
			o !== r?.index && (h = P.nextNode(), o++);
		}
		return P.currentNode = l$1, e;
	}
	p(t) {
		let i = 0;
		for (const s of this._$AV) void 0 !== s && (void 0 !== s.strings ? (s._$AI(t, s, i), i += s.strings.length - 2) : s._$AI(t[i])), i++;
	}
};
var k = class k {
	get _$AU() {
		return this._$AM?._$AU ?? this._$Cv;
	}
	constructor(t, i, s, e) {
		this.type = 2, this._$AH = A, this._$AN = void 0, this._$AA = t, this._$AB = i, this._$AM = s, this.options = e, this._$Cv = e?.isConnected ?? !0;
	}
	get parentNode() {
		let t = this._$AA.parentNode;
		const i = this._$AM;
		return void 0 !== i && 11 === t?.nodeType && (t = i.parentNode), t;
	}
	get startNode() {
		return this._$AA;
	}
	get endNode() {
		return this._$AB;
	}
	_$AI(t, i = this) {
		t = M$1(this, t, i), a(t) ? t === A || null == t || "" === t ? (this._$AH !== A && this._$AR(), this._$AH = A) : t !== this._$AH && t !== E && this._(t) : void 0 !== t._$litType$ ? this.$(t) : void 0 !== t.nodeType ? this.T(t) : d$1(t) ? this.k(t) : this._(t);
	}
	O(t) {
		return this._$AA.parentNode.insertBefore(t, this._$AB);
	}
	T(t) {
		this._$AH !== t && (this._$AR(), this._$AH = this.O(t));
	}
	_(t) {
		this._$AH !== A && a(this._$AH) ? this._$AA.nextSibling.data = t : this.T(l$1.createTextNode(t)), this._$AH = t;
	}
	$(t) {
		const { values: i, _$litType$: s } = t, e = "number" == typeof s ? this._$AC(t) : (void 0 === s.el && (s.el = S.createElement(V(s.h, s.h[0]), this.options)), s);
		if (this._$AH?._$AD === e) this._$AH.p(i);
		else {
			const t = new R(e, this), s = t.u(this.options);
			t.p(i), this.T(s), this._$AH = t;
		}
	}
	_$AC(t) {
		let i = C.get(t.strings);
		return void 0 === i && C.set(t.strings, i = new S(t)), i;
	}
	k(t) {
		u$1(this._$AH) || (this._$AH = [], this._$AR());
		const i = this._$AH;
		let s, e = 0;
		for (const h of t) e === i.length ? i.push(s = new k(this.O(c$2()), this.O(c$2()), this, this.options)) : s = i[e], s._$AI(h), e++;
		e < i.length && (this._$AR(s && s._$AB.nextSibling, e), i.length = e);
	}
	_$AR(t = this._$AA.nextSibling, s) {
		for (this._$AP?.(!1, !0, s); t !== this._$AB;) {
			const s = i$3(t).nextSibling;
			i$3(t).remove(), t = s;
		}
	}
	setConnected(t) {
		void 0 === this._$AM && (this._$Cv = t, this._$AP?.(t));
	}
};
var H = class {
	get tagName() {
		return this.element.tagName;
	}
	get _$AU() {
		return this._$AM._$AU;
	}
	constructor(t, i, s, e, h) {
		this.type = 1, this._$AH = A, this._$AN = void 0, this.element = t, this.name = i, this._$AM = e, this.options = h, s.length > 2 || "" !== s[0] || "" !== s[1] ? (this._$AH = Array(s.length - 1).fill(/* @__PURE__ */ new String()), this.strings = s) : this._$AH = A;
	}
	_$AI(t, i = this, s, e) {
		const h = this.strings;
		let o = !1;
		if (void 0 === h) t = M$1(this, t, i, 0), o = !a(t) || t !== this._$AH && t !== E, o && (this._$AH = t);
		else {
			const e = t;
			let n, r;
			for (t = h[0], n = 0; n < h.length - 1; n++) r = M$1(this, e[s + n], i, n), r === E && (r = this._$AH[n]), o ||= !a(r) || r !== this._$AH[n], r === A ? t = A : t !== A && (t += (r ?? "") + h[n + 1]), this._$AH[n] = r;
		}
		o && !e && this.j(t);
	}
	j(t) {
		t === A ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t ?? "");
	}
};
var I = class extends H {
	constructor() {
		super(...arguments), this.type = 3;
	}
	j(t) {
		this.element[this.name] = t === A ? void 0 : t;
	}
};
var L = class extends H {
	constructor() {
		super(...arguments), this.type = 4;
	}
	j(t) {
		this.element.toggleAttribute(this.name, !!t && t !== A);
	}
};
var z = class extends H {
	constructor(t, i, s, e, h) {
		super(t, i, s, e, h), this.type = 5;
	}
	_$AI(t, i = this) {
		if ((t = M$1(this, t, i, 0) ?? A) === E) return;
		const s = this._$AH, e = t === A && s !== A || t.capture !== s.capture || t.once !== s.once || t.passive !== s.passive, h = t !== A && (s === A || e);
		e && this.element.removeEventListener(this.name, this, s), h && this.element.addEventListener(this.name, this, t), this._$AH = t;
	}
	handleEvent(t) {
		"function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t) : this._$AH.handleEvent(t);
	}
};
var Z = class {
	constructor(t, i, s) {
		this.element = t, this.type = 6, this._$AN = void 0, this._$AM = i, this.options = s;
	}
	get _$AU() {
		return this._$AM._$AU;
	}
	_$AI(t) {
		M$1(this, t);
	}
};
var j$1 = {
	M: h$3,
	P: o$4,
	A: n$4,
	C: 1,
	L: N,
	R,
	D: d$1,
	V: M$1,
	I: k,
	H,
	N: L,
	U: z,
	B: I,
	F: Z
}, B = t$3.litHtmlPolyfillSupport;
B?.(S, k), (t$3.litHtmlVersions ??= []).push("3.3.2");
var D = (t, i, s) => {
	const e = s?.renderBefore ?? i;
	let h = e._$litPart$;
	if (void 0 === h) {
		const t = s?.renderBefore ?? null;
		e._$litPart$ = h = new k(i.insertBefore(c$2(), t), t, void 0, s ?? {});
	}
	return h._$AI(t), h;
};
//#endregion
//#region node_modules/.deno/lit-element@4.2.2/node_modules/lit-element/lit-element.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var s$2 = globalThis;
var i$2 = class extends y$1 {
	constructor() {
		super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
	}
	createRenderRoot() {
		const t = super.createRenderRoot();
		return this.renderOptions.renderBefore ??= t.firstChild, t;
	}
	update(t) {
		const r = this.render();
		this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t), this._$Do = D(r, this.renderRoot, this.renderOptions);
	}
	connectedCallback() {
		super.connectedCallback(), this._$Do?.setConnected(!0);
	}
	disconnectedCallback() {
		super.disconnectedCallback(), this._$Do?.setConnected(!1);
	}
	render() {
		return E;
	}
};
i$2._$litElement$ = !0, i$2["finalized"] = !0, s$2.litElementHydrateSupport?.({ LitElement: i$2 });
var o$3 = s$2.litElementPolyfillSupport;
o$3?.({ LitElement: i$2 });
(s$2.litElementVersions ??= []).push("4.2.2");
//#endregion
//#region node_modules/.deno/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/decorators/custom-element.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/
var t$2 = (t) => (e, o) => {
	void 0 !== o ? o.addInitializer(() => {
		customElements.define(t, e);
	}) : customElements.define(t, e);
};
//#endregion
//#region node_modules/.deno/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/decorators/property.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var o$2 = {
	attribute: !0,
	type: String,
	converter: u$2,
	reflect: !1,
	hasChanged: f$3
}, r$2 = (t = o$2, e, r) => {
	const { kind: n, metadata: i } = r;
	let s = globalThis.litPropertyMetadata.get(i);
	if (void 0 === s && globalThis.litPropertyMetadata.set(i, s = /* @__PURE__ */ new Map()), "setter" === n && ((t = Object.create(t)).wrapped = !0), s.set(r.name, t), "accessor" === n) {
		const { name: o } = r;
		return {
			set(r) {
				const n = e.get.call(this);
				e.set.call(this, r), this.requestUpdate(o, n, t, !0, r);
			},
			init(e) {
				return void 0 !== e && this.C(o, void 0, t, e), e;
			}
		};
	}
	if ("setter" === n) {
		const { name: o } = r;
		return function(r) {
			const n = this[o];
			e.call(this, r), this.requestUpdate(o, n, t, !0, r);
		};
	}
	throw Error("Unsupported decorator location: " + n);
};
function n$3(t) {
	return (e, o) => "object" == typeof o ? r$2(t, e, o) : ((t, e, o) => {
		const r = e.hasOwnProperty(o);
		return e.constructor.createProperty(o, t), r ? Object.getOwnPropertyDescriptor(e, o) : void 0;
	})(t, e, o);
}
//#endregion
//#region node_modules/.deno/lit-html@3.3.2/node_modules/lit-html/directive-helpers.js
/**
* @license
* Copyright 2020 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var { I: t$1 } = j$1, r$1 = (o) => void 0 === o.strings;
//#endregion
//#region node_modules/.deno/lit-html@3.3.2/node_modules/lit-html/directive.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/
var t = {
	ATTRIBUTE: 1,
	CHILD: 2,
	PROPERTY: 3,
	BOOLEAN_ATTRIBUTE: 4,
	EVENT: 5,
	ELEMENT: 6
}, e$1 = (t) => (...e) => ({
	_$litDirective$: t,
	values: e
});
var i = class {
	constructor(t) {}
	get _$AU() {
		return this._$AM._$AU;
	}
	_$AT(t, e, i) {
		this._$Ct = t, this._$AM = e, this._$Ci = i;
	}
	_$AS(t, e) {
		return this.update(t, e);
	}
	update(t, e) {
		return this.render(...e);
	}
};
//#endregion
//#region node_modules/.deno/lit-html@3.3.2/node_modules/lit-html/async-directive.js
/**
* @license
* Copyright 2017 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var s = (i, t) => {
	const e = i._$AN;
	if (void 0 === e) return !1;
	for (const i of e) i._$AO?.(t, !1), s(i, t);
	return !0;
}, o$1 = (i) => {
	let t, e;
	do {
		if (void 0 === (t = i._$AM)) break;
		e = t._$AN, e.delete(i), i = t;
	} while (0 === e?.size);
}, r = (i) => {
	for (let t; t = i._$AM; i = t) {
		let e = t._$AN;
		if (void 0 === e) t._$AN = e = /* @__PURE__ */ new Set();
		else if (e.has(i)) break;
		e.add(i), c(t);
	}
};
function h$1(i) {
	void 0 !== this._$AN ? (o$1(this), this._$AM = i, r(this)) : this._$AM = i;
}
function n$1(i, t = !1, e = 0) {
	const r = this._$AH, h = this._$AN;
	if (void 0 !== h && 0 !== h.size) if (t) if (Array.isArray(r)) for (let i = e; i < r.length; i++) s(r[i], !1), o$1(r[i]);
	else null != r && (s(r, !1), o$1(r));
	else s(this, i);
}
var c = (i) => {
	i.type == t.CHILD && (i._$AP ??= n$1, i._$AQ ??= h$1);
};
var f = class extends i {
	constructor() {
		super(...arguments), this._$AN = void 0;
	}
	_$AT(i, t, e) {
		super._$AT(i, t, e), r(this), this.isConnected = i._$AU;
	}
	_$AO(i, t = !0) {
		i !== this.isConnected && (this.isConnected = i, i ? this.reconnected?.() : this.disconnected?.()), t && (s(this, i), o$1(this));
	}
	setValue(t) {
		if (r$1(this._$Ct)) this._$Ct._$AI(t, this);
		else {
			const i = [...this._$Ct._$AH];
			i[this._$Ci] = t, this._$Ct._$AI(i, this, 0);
		}
	}
	disconnected() {}
	reconnected() {}
};
//#endregion
//#region node_modules/.deno/lit-html@3.3.2/node_modules/lit-html/directives/ref.js
/**
* @license
* Copyright 2020 Google LLC
* SPDX-License-Identifier: BSD-3-Clause
*/ var e = () => new h();
var h = class {};
var o = /* @__PURE__ */ new WeakMap(), n = e$1(class extends f {
	render(i) {
		return A;
	}
	update(i, [s]) {
		const e = s !== this.G;
		return e && void 0 !== this.G && this.rt(void 0), (e || this.lt !== this.ct) && (this.G = s, this.ht = i.options?.host, this.rt(this.ct = i.element)), A;
	}
	rt(t) {
		if (this.isConnected || (t = void 0), "function" == typeof this.G) {
			const i = this.ht ?? globalThis;
			let s = o.get(i);
			void 0 === s && (s = /* @__PURE__ */ new WeakMap(), o.set(i, s)), void 0 !== s.get(this.G) && this.G.call(this.ht, void 0), s.set(this.G, t), void 0 !== t && this.G.call(this.ht, t);
		} else this.G.value = t;
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
//#endregion
//#region lib/index.ts
function escapeXml(unsafe) {
	return unsafe.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");
}
var SVG_ICON_CALENDAR = w`<!-- Adapted from https://iconoir.com/ -->
<svg width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon">
  <path d="M15 4V2M15 4V6M15 4H10.5M3 10V19C3 20.1046 3.89543 21 5 21H19C20.1046 21 21 20.1046 21 19V10H3Z" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M3 10V6C3 4.89543 3.89543 4 5 4H7" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M7 2V6" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M21 10V6C21 4.89543 20.1046 4 19 4H18.5" stroke-linecap="round" stroke-linejoin="round"></path>
</svg>
`;
var SVG_ICON_INTERNET = w`<!-- Adapted from https://iconoir.com/ -->
<svg class="icon" width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M13 2.04932C13 2.04932 16 5.99994 16 11.9999" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M11 21.9506C11 21.9506 8 17.9999 8 11.9999C8 5.99994 11 2.04932 11 2.04932" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 15.5H12" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 8.5H21.3704" stroke-linecap="round" stroke-linejoin="round"></path>
  <path fill-rule="evenodd" clip-rule="evenodd" d="M21.8789 17.9174C22.3727 18.2211 22.3423 18.9604 21.8337 19.0181L19.2671 19.309L18.1159 21.6213C17.8878 22.0795 17.1827 21.8552 17.0661 21.2873L15.8108 15.1713C15.7123 14.6913 16.1437 14.3892 16.561 14.646L21.8789 17.9174Z"></path>
</svg>
`;
//#endregion
//#region \0@oxc-project+runtime@0.115.0/helpers/decorate.js
function __decorate(decorators, target, key, desc) {
	var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
	if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
	else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
	return c > 3 && r && Object.defineProperty(target, key, r), r;
}
//#endregion
//#region lib/create-addressbook-form.ts
var CreateAddressbookForm = class CreateAddressbookForm extends i$2 {
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Create addressbook</button>
      <dialog ${n(this.dialog)}>
        <h3>Create addressbook</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group addressbooks)
            <select .value=${this.user} @change=${(e) => this.principal = e.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => b`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" .value=${this.addr_id} @change=${(e) => this.addr_id = e.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e) => this.description = e.target.value} />
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
	async submit(e) {
		console.log(this.displayname);
		e.preventDefault();
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
			headers: { "Content-Type": "application/xml" },
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
__decorate([n$3()], CreateAddressbookForm.prototype, "user", void 0);
__decorate([n$3()], CreateAddressbookForm.prototype, "principal", void 0);
__decorate([n$3()], CreateAddressbookForm.prototype, "addr_id", void 0);
__decorate([n$3()], CreateAddressbookForm.prototype, "displayname", void 0);
__decorate([n$3()], CreateAddressbookForm.prototype, "description", void 0);
CreateAddressbookForm = __decorate([t$2("create-addressbook-form")], CreateAddressbookForm);
//#endregion
//#region lib/create-birthday-calendar-form.ts
var CreateBirthdayCalendarForm = class CreateBirthdayCalendarForm extends i$2 {
	constructor(..._args) {
		super(..._args);
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Create birthday calendar</button>
      <dialog ${n(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} required @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e) => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e) => this.color = e.target.value} />
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
	async submit(e) {
		e.preventDefault();
		if (!this.addr_id) {
			alert("Empty id");
			return;
		}
		if (!this.displayname) {
			alert("Empty displayname");
			return;
		}
		const order = Math.floor(Math.random() * 5e3);
		let response = await fetch(`/caldav/principal/${this.principal}/_birthdays_${this.addr_id}`, {
			method: "MKCOL",
			headers: { "Content-Type": "application/xml" },
			body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
            <ICAL:calendar-order>${order}</ICAL:calendar-order>
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
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "principal", void 0);
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "addr_id", void 0);
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "displayname", void 0);
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "description", void 0);
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "color", void 0);
__decorate([n$3()], CreateBirthdayCalendarForm.prototype, "timezones", void 0);
CreateBirthdayCalendarForm = __decorate([t$2("create-birthday-calendar-form")], CreateBirthdayCalendarForm);
//#endregion
//#region lib/timezones.ts
var timezonesPromise = null;
async function getTimezones() {
	timezonesPromise ||= new Promise(async (resolve, reject) => {
		try {
			resolve(await (await fetch("/frontend/_timezones.json")).json());
		} catch (e) {
			reject(e);
		}
	});
	return await timezonesPromise;
}
//#endregion
//#region lib/create-calendar-form.ts
var CreateCalendarForm = class CreateCalendarForm extends i$2 {
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
		this.components = new Set(["VEVENT", "VTODO"]);
	}
	async fetchTimezones() {
		this.timezones = await getTimezones();
	}
	createRenderRoot() {
		return this;
	}
	render() {
		return b`
      <button @click=${(e) => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${n(this.dialog)} @close=${(e) => this.resetForm()}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group calendars)
            <select required value=${this.user} @change=${(e) => this.principal = e.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => b`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${(e) => this.cal_id = e.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${(e) => this.timezone_id = e.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map((timezone) => b`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e) => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e) => this.color = e.target.value} />
          </label>
          <br>
          <br>
          <label>Type</label>
          <div class="tab-radio">
            <label>
              <input type="radio" name="type" .checked=${!this.isSubscription} @change=${(e) => this.isSubscription = false}></input>
              ${SVG_ICON_CALENDAR}
              Calendar
            </label>
            <label>
              <input type="radio" name="type" .checked=${this.isSubscription} @change=${(e) => this.isSubscription = true}></input>
              ${SVG_ICON_INTERNET}
              webCal Subscription
            </label>
          </div>
          <br>
          ${this.isSubscription ? b`
            <label>
              Subscription URL
              <input type="text" pattern="https://.*" .required=${this.isSubscription} .value=${this.subscriptionUrl} @change=${(e) => this.subscriptionUrl = e.target.value}  />
            </label>
            <br>
            <br>
          ` : b``}

          <label>Components</label>
          <div>
            ${[
			"VEVENT",
			"VTODO",
			"VJOURNAL"
		].map((comp) => b`
              <label>
                Support ${comp}
                <input type="checkbox" .value=${comp} @change=${(e) => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} .checked=${this.components.has(comp)} />
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
	async submit(e) {
		e.preventDefault();
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
		const order = Math.floor(Math.random() * 5e3);
		let response = await fetch(`/caldav/principal/${this.principal || this.user}/${this.cal_id}`, {
			method: "MKCOL",
			headers: { "Content-Type": "application/xml" },
			body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ""}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
            <ICAL:calendar-order>${order}</ICAL:calendar-order>
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
__decorate([n$3()], CreateCalendarForm.prototype, "user", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "principal", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "cal_id", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "displayname", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "description", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "timezone_id", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "color", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "isSubscription", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "subscriptionUrl", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "components", void 0);
__decorate([n$3()], CreateCalendarForm.prototype, "timezones", void 0);
CreateCalendarForm = __decorate([t$2("create-calendar-form")], CreateCalendarForm);
//#endregion
//#region lib/delete-button.ts
var DeleteButton = class DeleteButton extends i$2 {
	constructor() {
		super();
		this.trash = false;
	}
	createRenderRoot() {
		return this;
	}
	render() {
		return b`<button class="delete" @click=${(e) => this._onClick(e)}>${this.trash ? "Trash" : "Delete"}</button>`;
	}
	async _onClick(event) {
		event.preventDefault();
		if (!this.trash && !confirm("Do you want to delete this collection permanently?")) return;
		let response = await fetch(this.href, {
			method: "DELETE",
			headers: { "X-No-Trashbin": this.trash ? "0" : "1" }
		});
		if (response.status < 200 || response.status >= 300) {
			alert("An error occured, look into the console");
			console.error(response);
			return;
		}
		window.location.reload();
	}
};
__decorate([n$3({ type: Boolean })], DeleteButton.prototype, "trash", void 0);
__decorate([n$3()], DeleteButton.prototype, "href", void 0);
DeleteButton = __decorate([t$2("delete-button")], DeleteButton);
//#endregion
//#region lib/edit-addressbook-form.ts
var EditAddressbookForm = class EditAddressbookForm extends i$2 {
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${n(this.dialog)}>
        <h3>Edit addressbook</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" .value=${this.displayname} @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e) => this.description = e.target.value} />
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
	async submit(e) {
		e.preventDefault();
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
			headers: { "Content-Type": "application/xml" },
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
__decorate([n$3()], EditAddressbookForm.prototype, "principal", void 0);
__decorate([n$3()], EditAddressbookForm.prototype, "addr_id", void 0);
__decorate([n$3()], EditAddressbookForm.prototype, "displayname", void 0);
__decorate([n$3()], EditAddressbookForm.prototype, "description", void 0);
EditAddressbookForm = __decorate([t$2("edit-addressbook-form")], EditAddressbookForm);
//#endregion
//#region lib/edit-calendar-form.ts
var EditCalendarForm = class EditCalendarForm extends i$2 {
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${n(this.dialog)}>
        <h3>Edit calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${(e) => this.timezone_id = e.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map((timezone) => b`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e) => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e) => this.color = e.target.value} />
          </label>
          <br>
          ${[
			"VEVENT",
			"VTODO",
			"VJOURNAL"
		].map((comp) => b`
            <label>
              Support ${comp}
              <input type="checkbox" .value=${comp} ?checked=${this.components.has(comp)} @change=${(e) => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} />
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
	async submit(e) {
		e.preventDefault();
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
			headers: { "Content-Type": "application/xml" },
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
__decorate([n$3()], EditCalendarForm.prototype, "principal", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "cal_id", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "displayname", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "description", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "timezone_id", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "color", void 0);
__decorate([n$3({ converter: {
	fromAttribute: (value, _type) => new Set(value ? JSON.parse(value) : []),
	toAttribute: (value, _type) => JSON.stringify(value)
} })], EditCalendarForm.prototype, "components", void 0);
__decorate([n$3()], EditCalendarForm.prototype, "timezones", void 0);
EditCalendarForm = __decorate([t$2("edit-calendar-form")], EditCalendarForm);
//#endregion
//#region lib/import-addressbook-form.ts
var ImportAddressbookForm = class ImportAddressbookForm extends i$2 {
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Import addressbook</button>
      <dialog ${n(this.dialog)}>
        <h3>Import addressbook</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group addressbook)
            <select name="principal" required .value=${this.user} @change=${(e) => this.principal = e.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => b`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.addressbook_id} @change=${(e) => this.addressbook_id = e.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" accept="text/vcard" required @change=${(e) => this.file = e.target.files[0]} />
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
	async submit(e) {
		e.preventDefault();
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
			headers: { "Content-Type": "text/vcard" },
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
__decorate([n$3()], ImportAddressbookForm.prototype, "user", void 0);
__decorate([n$3()], ImportAddressbookForm.prototype, "principal", void 0);
__decorate([n$3()], ImportAddressbookForm.prototype, "addressbook_id", void 0);
ImportAddressbookForm = __decorate([t$2("import-addressbook-form")], ImportAddressbookForm);
//#endregion
//#region lib/import-calendar-form.ts
var ImportCalendarForm = class ImportCalendarForm extends i$2 {
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
		return b`
      <button @click=${() => this.dialog.value.showModal()}>Import calendar</button>
      <dialog ${n(this.dialog)}>
        <h3>Import calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group calendars)
            <select name="principal" required .value=${this.user} @change=${(e) => this.principal = e.target.value}>
              <option .value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => b`
                <option .value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${(e) => this.cal_id = e.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" required accept="text/calendar" @change=${(e) => this.file = e.target.files[0]} />
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
	async submit(e) {
		e.preventDefault();
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
			headers: { "Content-Type": "text/calendar" },
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
__decorate([n$3()], ImportCalendarForm.prototype, "user", void 0);
__decorate([n$3()], ImportCalendarForm.prototype, "principal", void 0);
__decorate([n$3()], ImportCalendarForm.prototype, "cal_id", void 0);
ImportCalendarForm = __decorate([t$2("import-calendar-form")], ImportCalendarForm);
//#endregion
export { CreateAddressbookForm, CreateBirthdayCalendarForm, CreateCalendarForm, DeleteButton, EditAddressbookForm, EditCalendarForm, ImportAddressbookForm, ImportCalendarForm };
