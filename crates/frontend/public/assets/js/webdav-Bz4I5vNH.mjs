var Qe = Object.defineProperty;
var ce = (r) => {
  throw TypeError(r);
};
var tr = (r, t, e) => t in r ? Qe(r, t, { enumerable: !0, configurable: !0, writable: !0, value: e }) : r[t] = e;
var W = (r, t, e) => tr(r, typeof t != "symbol" ? t + "" : t, e), Dt = (r, t, e) => t.has(r) || ce("Cannot " + e);
var T = (r, t, e) => (Dt(r, t, "read from private field"), e ? e.call(r) : t.get(r)), rt = (r, t, e) => t.has(r) ? ce("Cannot add the same private member more than once") : t instanceof WeakSet ? t.add(r) : t.set(r, e), F = (r, t, e, n) => (Dt(r, t, "write to private field"), n ? n.call(r, e) : t.set(r, e), e), ht = (r, t, e) => (Dt(r, t, "access private method"), e);
/*! For license information please see index.js.LICENSE.txt */
var er = { 2: (r) => {
  function t(o, i, l) {
    o instanceof RegExp && (o = e(o, l)), i instanceof RegExp && (i = e(i, l));
    var u = n(o, i, l);
    return u && { start: u[0], end: u[1], pre: l.slice(0, u[0]), body: l.slice(u[0] + o.length, u[1]), post: l.slice(u[1] + i.length) };
  }
  function e(o, i) {
    var l = i.match(o);
    return l ? l[0] : null;
  }
  function n(o, i, l) {
    var u, y, c, s, h, a = l.indexOf(o), d = l.indexOf(i, a + 1), p = a;
    if (a >= 0 && d > 0) {
      for (u = [], c = l.length; p >= 0 && !h; ) p == a ? (u.push(p), a = l.indexOf(o, p + 1)) : u.length == 1 ? h = [u.pop(), d] : ((y = u.pop()) < c && (c = y, s = d), d = l.indexOf(i, p + 1)), p = a < d && a >= 0 ? a : d;
      u.length && (h = [c, s]);
    }
    return h;
  }
  r.exports = t, t.range = n;
}, 101: function(r, t, e) {
  var n;
  r = e.nmd(r), function(o) {
    r && r.exports, typeof global == "object" && global;
    var i = function(s) {
      this.message = s;
    };
    (i.prototype = new Error()).name = "InvalidCharacterError";
    var l = function(s) {
      throw new i(s);
    }, u = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", y = /[\t\n\f\r ]/g, c = { encode: function(s) {
      s = String(s), /[^\0-\xFF]/.test(s) && l("The string to be encoded contains characters outside of the Latin1 range.");
      for (var h, a, d, p, g = s.length % 3, m = "", N = -1, f = s.length - g; ++N < f; ) h = s.charCodeAt(N) << 16, a = s.charCodeAt(++N) << 8, d = s.charCodeAt(++N), m += u.charAt((p = h + a + d) >> 18 & 63) + u.charAt(p >> 12 & 63) + u.charAt(p >> 6 & 63) + u.charAt(63 & p);
      return g == 2 ? (h = s.charCodeAt(N) << 8, a = s.charCodeAt(++N), m += u.charAt((p = h + a) >> 10) + u.charAt(p >> 4 & 63) + u.charAt(p << 2 & 63) + "=") : g == 1 && (p = s.charCodeAt(N), m += u.charAt(p >> 2) + u.charAt(p << 4 & 63) + "=="), m;
    }, decode: function(s) {
      var h = (s = String(s).replace(y, "")).length;
      h % 4 == 0 && (h = (s = s.replace(/==?$/, "")).length), (h % 4 == 1 || /[^+a-zA-Z0-9/]/.test(s)) && l("Invalid character: the string to be decoded is not correctly encoded.");
      for (var a, d, p = 0, g = "", m = -1; ++m < h; ) d = u.indexOf(s.charAt(m)), a = p % 4 ? 64 * a + d : d, p++ % 4 && (g += String.fromCharCode(255 & a >> (-2 * p & 6)));
      return g;
    }, version: "1.0.0" };
    (n = (function() {
      return c;
    }).call(t, e, t, r)) === void 0 || (r.exports = n);
  }();
}, 172: (r, t) => {
  t.d = function(e) {
    if (!e) return 0;
    for (var n = (e = e.toString()).length, o = e.length; o--; ) {
      var i = e.charCodeAt(o);
      56320 <= i && i <= 57343 && o--, 127 < i && i <= 2047 ? n++ : 2047 < i && i <= 65535 && (n += 2);
    }
    return n;
  };
}, 526: (r) => {
  var t = { utf8: { stringToBytes: function(e) {
    return t.bin.stringToBytes(unescape(encodeURIComponent(e)));
  }, bytesToString: function(e) {
    return decodeURIComponent(escape(t.bin.bytesToString(e)));
  } }, bin: { stringToBytes: function(e) {
    for (var n = [], o = 0; o < e.length; o++) n.push(255 & e.charCodeAt(o));
    return n;
  }, bytesToString: function(e) {
    for (var n = [], o = 0; o < e.length; o++) n.push(String.fromCharCode(e[o]));
    return n.join("");
  } } };
  r.exports = t;
}, 298: (r) => {
  var t, e;
  t = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/", e = { rotl: function(n, o) {
    return n << o | n >>> 32 - o;
  }, rotr: function(n, o) {
    return n << 32 - o | n >>> o;
  }, endian: function(n) {
    if (n.constructor == Number) return 16711935 & e.rotl(n, 8) | 4278255360 & e.rotl(n, 24);
    for (var o = 0; o < n.length; o++) n[o] = e.endian(n[o]);
    return n;
  }, randomBytes: function(n) {
    for (var o = []; n > 0; n--) o.push(Math.floor(256 * Math.random()));
    return o;
  }, bytesToWords: function(n) {
    for (var o = [], i = 0, l = 0; i < n.length; i++, l += 8) o[l >>> 5] |= n[i] << 24 - l % 32;
    return o;
  }, wordsToBytes: function(n) {
    for (var o = [], i = 0; i < 32 * n.length; i += 8) o.push(n[i >>> 5] >>> 24 - i % 32 & 255);
    return o;
  }, bytesToHex: function(n) {
    for (var o = [], i = 0; i < n.length; i++) o.push((n[i] >>> 4).toString(16)), o.push((15 & n[i]).toString(16));
    return o.join("");
  }, hexToBytes: function(n) {
    for (var o = [], i = 0; i < n.length; i += 2) o.push(parseInt(n.substr(i, 2), 16));
    return o;
  }, bytesToBase64: function(n) {
    for (var o = [], i = 0; i < n.length; i += 3) for (var l = n[i] << 16 | n[i + 1] << 8 | n[i + 2], u = 0; u < 4; u++) 8 * i + 6 * u <= 8 * n.length ? o.push(t.charAt(l >>> 6 * (3 - u) & 63)) : o.push("=");
    return o.join("");
  }, base64ToBytes: function(n) {
    n = n.replace(/[^A-Z0-9+\/]/gi, "");
    for (var o = [], i = 0, l = 0; i < n.length; l = ++i % 4) l != 0 && o.push((t.indexOf(n.charAt(i - 1)) & Math.pow(2, -2 * l + 8) - 1) << 2 * l | t.indexOf(n.charAt(i)) >>> 6 - 2 * l);
    return o;
  } }, r.exports = e;
}, 635: (r, t, e) => {
  const n = e(31), o = e(338), i = e(221);
  r.exports = { XMLParser: o, XMLValidator: n, XMLBuilder: i };
}, 118: (r) => {
  r.exports = function(t) {
    return typeof t == "function" ? t : Array.isArray(t) ? (e) => {
      for (const n of t)
        if (typeof n == "string" && e === n || n instanceof RegExp && n.test(e)) return !0;
    } : () => !1;
  };
}, 705: (r, t) => {
  const e = ":A-Za-z_\\u00C0-\\u00D6\\u00D8-\\u00F6\\u00F8-\\u02FF\\u0370-\\u037D\\u037F-\\u1FFF\\u200C-\\u200D\\u2070-\\u218F\\u2C00-\\u2FEF\\u3001-\\uD7FF\\uF900-\\uFDCF\\uFDF0-\\uFFFD", n = "[" + e + "][" + e + "\\-.\\d\\u00B7\\u0300-\\u036F\\u203F-\\u2040]*", o = new RegExp("^" + n + "$");
  t.isExist = function(i) {
    return i !== void 0;
  }, t.isEmptyObject = function(i) {
    return Object.keys(i).length === 0;
  }, t.merge = function(i, l, u) {
    if (l) {
      const y = Object.keys(l), c = y.length;
      for (let s = 0; s < c; s++) i[y[s]] = u === "strict" ? [l[y[s]]] : l[y[s]];
    }
  }, t.getValue = function(i) {
    return t.isExist(i) ? i : "";
  }, t.isName = function(i) {
    return o.exec(i) != null;
  }, t.getAllMatches = function(i, l) {
    const u = [];
    let y = l.exec(i);
    for (; y; ) {
      const c = [];
      c.startIndex = l.lastIndex - y[0].length;
      const s = y.length;
      for (let h = 0; h < s; h++) c.push(y[h]);
      u.push(c), y = l.exec(i);
    }
    return u;
  }, t.nameRegexp = n;
}, 31: (r, t, e) => {
  const n = e(705), o = { allowBooleanAttributes: !1, unpairedTags: [] };
  function i(f) {
    return f === " " || f === "	" || f === `
` || f === "\r";
  }
  function l(f, v) {
    const x = v;
    for (; v < f.length; v++) if (!(f[v] != "?" && f[v] != " ")) {
      const A = f.substr(x, v - x);
      if (v > 5 && A === "xml") return p("InvalidXml", "XML declaration allowed only at the start of the document.", m(f, v));
      if (f[v] == "?" && f[v + 1] == ">") {
        v++;
        break;
      }
    }
    return v;
  }
  function u(f, v) {
    if (f.length > v + 5 && f[v + 1] === "-" && f[v + 2] === "-") {
      for (v += 3; v < f.length; v++) if (f[v] === "-" && f[v + 1] === "-" && f[v + 2] === ">") {
        v += 2;
        break;
      }
    } else if (f.length > v + 8 && f[v + 1] === "D" && f[v + 2] === "O" && f[v + 3] === "C" && f[v + 4] === "T" && f[v + 5] === "Y" && f[v + 6] === "P" && f[v + 7] === "E") {
      let x = 1;
      for (v += 8; v < f.length; v++) if (f[v] === "<") x++;
      else if (f[v] === ">" && (x--, x === 0)) break;
    } else if (f.length > v + 9 && f[v + 1] === "[" && f[v + 2] === "C" && f[v + 3] === "D" && f[v + 4] === "A" && f[v + 5] === "T" && f[v + 6] === "A" && f[v + 7] === "[") {
      for (v += 8; v < f.length; v++) if (f[v] === "]" && f[v + 1] === "]" && f[v + 2] === ">") {
        v += 2;
        break;
      }
    }
    return v;
  }
  t.validate = function(f, v) {
    v = Object.assign({}, o, v);
    const x = [];
    let A = !1, b = !1;
    f[0] === "\uFEFF" && (f = f.substr(1));
    for (let w = 0; w < f.length; w++) if (f[w] === "<" && f[w + 1] === "?") {
      if (w += 2, w = l(f, w), w.err) return w;
    } else {
      if (f[w] !== "<") {
        if (i(f[w])) continue;
        return p("InvalidChar", "char '" + f[w] + "' is not expected.", m(f, w));
      }
      {
        let E = w;
        if (w++, f[w] === "!") {
          w = u(f, w);
          continue;
        }
        {
          let j = !1;
          f[w] === "/" && (j = !0, w++);
          let S = "";
          for (; w < f.length && f[w] !== ">" && f[w] !== " " && f[w] !== "	" && f[w] !== `
` && f[w] !== "\r"; w++) S += f[w];
          if (S = S.trim(), S[S.length - 1] === "/" && (S = S.substring(0, S.length - 1), w--), O = S, !n.isName(O)) {
            let I;
            return I = S.trim().length === 0 ? "Invalid space after '<'." : "Tag '" + S + "' is an invalid name.", p("InvalidTag", I, m(f, w));
          }
          const $ = s(f, w);
          if ($ === !1) return p("InvalidAttr", "Attributes for '" + S + "' have open quote.", m(f, w));
          let P = $.value;
          if (w = $.index, P[P.length - 1] === "/") {
            const I = w - P.length;
            P = P.substring(0, P.length - 1);
            const C = a(P, v);
            if (C !== !0) return p(C.err.code, C.err.msg, m(f, I + C.err.line));
            A = !0;
          } else if (j) {
            if (!$.tagClosed) return p("InvalidTag", "Closing tag '" + S + "' doesn't have proper closing.", m(f, w));
            if (P.trim().length > 0) return p("InvalidTag", "Closing tag '" + S + "' can't have attributes or invalid starting.", m(f, E));
            if (x.length === 0) return p("InvalidTag", "Closing tag '" + S + "' has not been opened.", m(f, E));
            {
              const I = x.pop();
              if (S !== I.tagName) {
                let C = m(f, I.tagStartPos);
                return p("InvalidTag", "Expected closing tag '" + I.tagName + "' (opened in line " + C.line + ", col " + C.col + ") instead of closing tag '" + S + "'.", m(f, E));
              }
              x.length == 0 && (b = !0);
            }
          } else {
            const I = a(P, v);
            if (I !== !0) return p(I.err.code, I.err.msg, m(f, w - P.length + I.err.line));
            if (b === !0) return p("InvalidXml", "Multiple possible root nodes found.", m(f, w));
            v.unpairedTags.indexOf(S) !== -1 || x.push({ tagName: S, tagStartPos: E }), A = !0;
          }
          for (w++; w < f.length; w++) if (f[w] === "<") {
            if (f[w + 1] === "!") {
              w++, w = u(f, w);
              continue;
            }
            if (f[w + 1] !== "?") break;
            if (w = l(f, ++w), w.err) return w;
          } else if (f[w] === "&") {
            const I = d(f, w);
            if (I == -1) return p("InvalidChar", "char '&' is not expected.", m(f, w));
            w = I;
          } else if (b === !0 && !i(f[w])) return p("InvalidXml", "Extra text at the end", m(f, w));
          f[w] === "<" && w--;
        }
      }
    }
    var O;
    return A ? x.length == 1 ? p("InvalidTag", "Unclosed tag '" + x[0].tagName + "'.", m(f, x[0].tagStartPos)) : !(x.length > 0) || p("InvalidXml", "Invalid '" + JSON.stringify(x.map((w) => w.tagName), null, 4).replace(/\r?\n/g, "") + "' found.", { line: 1, col: 1 }) : p("InvalidXml", "Start tag expected.", 1);
  };
  const y = '"', c = "'";
  function s(f, v) {
    let x = "", A = "", b = !1;
    for (; v < f.length; v++) {
      if (f[v] === y || f[v] === c) A === "" ? A = f[v] : A !== f[v] || (A = "");
      else if (f[v] === ">" && A === "") {
        b = !0;
        break;
      }
      x += f[v];
    }
    return A === "" && { value: x, index: v, tagClosed: b };
  }
  const h = new RegExp(`(\\s*)([^\\s=]+)(\\s*=)?(\\s*(['"])(([\\s\\S])*?)\\5)?`, "g");
  function a(f, v) {
    const x = n.getAllMatches(f, h), A = {};
    for (let b = 0; b < x.length; b++) {
      if (x[b][1].length === 0) return p("InvalidAttr", "Attribute '" + x[b][2] + "' has no space in starting.", N(x[b]));
      if (x[b][3] !== void 0 && x[b][4] === void 0) return p("InvalidAttr", "Attribute '" + x[b][2] + "' is without value.", N(x[b]));
      if (x[b][3] === void 0 && !v.allowBooleanAttributes) return p("InvalidAttr", "boolean attribute '" + x[b][2] + "' is not allowed.", N(x[b]));
      const O = x[b][2];
      if (!g(O)) return p("InvalidAttr", "Attribute '" + O + "' is an invalid name.", N(x[b]));
      if (A.hasOwnProperty(O)) return p("InvalidAttr", "Attribute '" + O + "' is repeated.", N(x[b]));
      A[O] = 1;
    }
    return !0;
  }
  function d(f, v) {
    if (f[++v] === ";") return -1;
    if (f[v] === "#") return function(A, b) {
      let O = /\d/;
      for (A[b] === "x" && (b++, O = /[\da-fA-F]/); b < A.length; b++) {
        if (A[b] === ";") return b;
        if (!A[b].match(O)) break;
      }
      return -1;
    }(f, ++v);
    let x = 0;
    for (; v < f.length; v++, x++) if (!(f[v].match(/\w/) && x < 20)) {
      if (f[v] === ";") break;
      return -1;
    }
    return v;
  }
  function p(f, v, x) {
    return { err: { code: f, msg: v, line: x.line || x, col: x.col } };
  }
  function g(f) {
    return n.isName(f);
  }
  function m(f, v) {
    const x = f.substring(0, v).split(/\r?\n/);
    return { line: x.length, col: x[x.length - 1].length + 1 };
  }
  function N(f) {
    return f.startIndex + f[1].length;
  }
}, 221: (r, t, e) => {
  const n = e(87), o = e(118), i = { attributeNamePrefix: "@_", attributesGroupName: !1, textNodeName: "#text", ignoreAttributes: !0, cdataPropName: !1, format: !1, indentBy: "  ", suppressEmptyNode: !1, suppressUnpairedNode: !0, suppressBooleanAttributes: !0, tagValueProcessor: function(s, h) {
    return h;
  }, attributeValueProcessor: function(s, h) {
    return h;
  }, preserveOrder: !1, commentPropName: !1, unpairedTags: [], entities: [{ regex: new RegExp("&", "g"), val: "&amp;" }, { regex: new RegExp(">", "g"), val: "&gt;" }, { regex: new RegExp("<", "g"), val: "&lt;" }, { regex: new RegExp("'", "g"), val: "&apos;" }, { regex: new RegExp('"', "g"), val: "&quot;" }], processEntities: !0, stopNodes: [], oneListGroup: !1 };
  function l(s) {
    this.options = Object.assign({}, i, s), this.options.ignoreAttributes === !0 || this.options.attributesGroupName ? this.isAttribute = function() {
      return !1;
    } : (this.ignoreAttributesFn = o(this.options.ignoreAttributes), this.attrPrefixLen = this.options.attributeNamePrefix.length, this.isAttribute = c), this.processTextOrObjNode = u, this.options.format ? (this.indentate = y, this.tagEndChar = `>
`, this.newLine = `
`) : (this.indentate = function() {
      return "";
    }, this.tagEndChar = ">", this.newLine = "");
  }
  function u(s, h, a, d) {
    const p = this.j2x(s, a + 1, d.concat(h));
    return s[this.options.textNodeName] !== void 0 && Object.keys(s).length === 1 ? this.buildTextValNode(s[this.options.textNodeName], h, p.attrStr, a) : this.buildObjectNode(p.val, h, p.attrStr, a);
  }
  function y(s) {
    return this.options.indentBy.repeat(s);
  }
  function c(s) {
    return !(!s.startsWith(this.options.attributeNamePrefix) || s === this.options.textNodeName) && s.substr(this.attrPrefixLen);
  }
  l.prototype.build = function(s) {
    return this.options.preserveOrder ? n(s, this.options) : (Array.isArray(s) && this.options.arrayNodeName && this.options.arrayNodeName.length > 1 && (s = { [this.options.arrayNodeName]: s }), this.j2x(s, 0, []).val);
  }, l.prototype.j2x = function(s, h, a) {
    let d = "", p = "";
    const g = a.join(".");
    for (let m in s) if (Object.prototype.hasOwnProperty.call(s, m)) if (s[m] === void 0) this.isAttribute(m) && (p += "");
    else if (s[m] === null) this.isAttribute(m) ? p += "" : m[0] === "?" ? p += this.indentate(h) + "<" + m + "?" + this.tagEndChar : p += this.indentate(h) + "<" + m + "/" + this.tagEndChar;
    else if (s[m] instanceof Date) p += this.buildTextValNode(s[m], m, "", h);
    else if (typeof s[m] != "object") {
      const N = this.isAttribute(m);
      if (N && !this.ignoreAttributesFn(N, g)) d += this.buildAttrPairStr(N, "" + s[m]);
      else if (!N) if (m === this.options.textNodeName) {
        let f = this.options.tagValueProcessor(m, "" + s[m]);
        p += this.replaceEntitiesValue(f);
      } else p += this.buildTextValNode(s[m], m, "", h);
    } else if (Array.isArray(s[m])) {
      const N = s[m].length;
      let f = "", v = "";
      for (let x = 0; x < N; x++) {
        const A = s[m][x];
        if (A !== void 0) if (A === null) m[0] === "?" ? p += this.indentate(h) + "<" + m + "?" + this.tagEndChar : p += this.indentate(h) + "<" + m + "/" + this.tagEndChar;
        else if (typeof A == "object") if (this.options.oneListGroup) {
          const b = this.j2x(A, h + 1, a.concat(m));
          f += b.val, this.options.attributesGroupName && A.hasOwnProperty(this.options.attributesGroupName) && (v += b.attrStr);
        } else f += this.processTextOrObjNode(A, m, h, a);
        else if (this.options.oneListGroup) {
          let b = this.options.tagValueProcessor(m, A);
          b = this.replaceEntitiesValue(b), f += b;
        } else f += this.buildTextValNode(A, m, "", h);
      }
      this.options.oneListGroup && (f = this.buildObjectNode(f, m, v, h)), p += f;
    } else if (this.options.attributesGroupName && m === this.options.attributesGroupName) {
      const N = Object.keys(s[m]), f = N.length;
      for (let v = 0; v < f; v++) d += this.buildAttrPairStr(N[v], "" + s[m][N[v]]);
    } else p += this.processTextOrObjNode(s[m], m, h, a);
    return { attrStr: d, val: p };
  }, l.prototype.buildAttrPairStr = function(s, h) {
    return h = this.options.attributeValueProcessor(s, "" + h), h = this.replaceEntitiesValue(h), this.options.suppressBooleanAttributes && h === "true" ? " " + s : " " + s + '="' + h + '"';
  }, l.prototype.buildObjectNode = function(s, h, a, d) {
    if (s === "") return h[0] === "?" ? this.indentate(d) + "<" + h + a + "?" + this.tagEndChar : this.indentate(d) + "<" + h + a + this.closeTag(h) + this.tagEndChar;
    {
      let p = "</" + h + this.tagEndChar, g = "";
      return h[0] === "?" && (g = "?", p = ""), !a && a !== "" || s.indexOf("<") !== -1 ? this.options.commentPropName !== !1 && h === this.options.commentPropName && g.length === 0 ? this.indentate(d) + `<!--${s}-->` + this.newLine : this.indentate(d) + "<" + h + a + g + this.tagEndChar + s + this.indentate(d) + p : this.indentate(d) + "<" + h + a + g + ">" + s + p;
    }
  }, l.prototype.closeTag = function(s) {
    let h = "";
    return this.options.unpairedTags.indexOf(s) !== -1 ? this.options.suppressUnpairedNode || (h = "/") : h = this.options.suppressEmptyNode ? "/" : `></${s}`, h;
  }, l.prototype.buildTextValNode = function(s, h, a, d) {
    if (this.options.cdataPropName !== !1 && h === this.options.cdataPropName) return this.indentate(d) + `<![CDATA[${s}]]>` + this.newLine;
    if (this.options.commentPropName !== !1 && h === this.options.commentPropName) return this.indentate(d) + `<!--${s}-->` + this.newLine;
    if (h[0] === "?") return this.indentate(d) + "<" + h + a + "?" + this.tagEndChar;
    {
      let p = this.options.tagValueProcessor(h, s);
      return p = this.replaceEntitiesValue(p), p === "" ? this.indentate(d) + "<" + h + a + this.closeTag(h) + this.tagEndChar : this.indentate(d) + "<" + h + a + ">" + p + "</" + h + this.tagEndChar;
    }
  }, l.prototype.replaceEntitiesValue = function(s) {
    if (s && s.length > 0 && this.options.processEntities) for (let h = 0; h < this.options.entities.length; h++) {
      const a = this.options.entities[h];
      s = s.replace(a.regex, a.val);
    }
    return s;
  }, r.exports = l;
}, 87: (r) => {
  function t(l, u, y, c) {
    let s = "", h = !1;
    for (let a = 0; a < l.length; a++) {
      const d = l[a], p = e(d);
      if (p === void 0) continue;
      let g = "";
      if (g = y.length === 0 ? p : `${y}.${p}`, p === u.textNodeName) {
        let v = d[p];
        o(g, u) || (v = u.tagValueProcessor(p, v), v = i(v, u)), h && (s += c), s += v, h = !1;
        continue;
      }
      if (p === u.cdataPropName) {
        h && (s += c), s += `<![CDATA[${d[p][0][u.textNodeName]}]]>`, h = !1;
        continue;
      }
      if (p === u.commentPropName) {
        s += c + `<!--${d[p][0][u.textNodeName]}-->`, h = !0;
        continue;
      }
      if (p[0] === "?") {
        const v = n(d[":@"], u), x = p === "?xml" ? "" : c;
        let A = d[p][0][u.textNodeName];
        A = A.length !== 0 ? " " + A : "", s += x + `<${p}${A}${v}?>`, h = !0;
        continue;
      }
      let m = c;
      m !== "" && (m += u.indentBy);
      const N = c + `<${p}${n(d[":@"], u)}`, f = t(d[p], u, g, m);
      u.unpairedTags.indexOf(p) !== -1 ? u.suppressUnpairedNode ? s += N + ">" : s += N + "/>" : f && f.length !== 0 || !u.suppressEmptyNode ? f && f.endsWith(">") ? s += N + `>${f}${c}</${p}>` : (s += N + ">", f && c !== "" && (f.includes("/>") || f.includes("</")) ? s += c + u.indentBy + f + c : s += f, s += `</${p}>`) : s += N + "/>", h = !0;
    }
    return s;
  }
  function e(l) {
    const u = Object.keys(l);
    for (let y = 0; y < u.length; y++) {
      const c = u[y];
      if (l.hasOwnProperty(c) && c !== ":@") return c;
    }
  }
  function n(l, u) {
    let y = "";
    if (l && !u.ignoreAttributes) for (let c in l) {
      if (!l.hasOwnProperty(c)) continue;
      let s = u.attributeValueProcessor(c, l[c]);
      s = i(s, u), s === !0 && u.suppressBooleanAttributes ? y += ` ${c.substr(u.attributeNamePrefix.length)}` : y += ` ${c.substr(u.attributeNamePrefix.length)}="${s}"`;
    }
    return y;
  }
  function o(l, u) {
    let y = (l = l.substr(0, l.length - u.textNodeName.length - 1)).substr(l.lastIndexOf(".") + 1);
    for (let c in u.stopNodes) if (u.stopNodes[c] === l || u.stopNodes[c] === "*." + y) return !0;
    return !1;
  }
  function i(l, u) {
    if (l && l.length > 0 && u.processEntities) for (let y = 0; y < u.entities.length; y++) {
      const c = u.entities[y];
      l = l.replace(c.regex, c.val);
    }
    return l;
  }
  r.exports = function(l, u) {
    let y = "";
    return u.format && u.indentBy.length > 0 && (y = `
`), t(l, u, "", y);
  };
}, 193: (r, t, e) => {
  const n = e(705);
  function o(h, a) {
    let d = "";
    for (; a < h.length && h[a] !== "'" && h[a] !== '"'; a++) d += h[a];
    if (d = d.trim(), d.indexOf(" ") !== -1) throw new Error("External entites are not supported");
    const p = h[a++];
    let g = "";
    for (; a < h.length && h[a] !== p; a++) g += h[a];
    return [d, g, a];
  }
  function i(h, a) {
    return h[a + 1] === "!" && h[a + 2] === "-" && h[a + 3] === "-";
  }
  function l(h, a) {
    return h[a + 1] === "!" && h[a + 2] === "E" && h[a + 3] === "N" && h[a + 4] === "T" && h[a + 5] === "I" && h[a + 6] === "T" && h[a + 7] === "Y";
  }
  function u(h, a) {
    return h[a + 1] === "!" && h[a + 2] === "E" && h[a + 3] === "L" && h[a + 4] === "E" && h[a + 5] === "M" && h[a + 6] === "E" && h[a + 7] === "N" && h[a + 8] === "T";
  }
  function y(h, a) {
    return h[a + 1] === "!" && h[a + 2] === "A" && h[a + 3] === "T" && h[a + 4] === "T" && h[a + 5] === "L" && h[a + 6] === "I" && h[a + 7] === "S" && h[a + 8] === "T";
  }
  function c(h, a) {
    return h[a + 1] === "!" && h[a + 2] === "N" && h[a + 3] === "O" && h[a + 4] === "T" && h[a + 5] === "A" && h[a + 6] === "T" && h[a + 7] === "I" && h[a + 8] === "O" && h[a + 9] === "N";
  }
  function s(h) {
    if (n.isName(h)) return h;
    throw new Error(`Invalid entity name ${h}`);
  }
  r.exports = function(h, a) {
    const d = {};
    if (h[a + 3] !== "O" || h[a + 4] !== "C" || h[a + 5] !== "T" || h[a + 6] !== "Y" || h[a + 7] !== "P" || h[a + 8] !== "E") throw new Error("Invalid Tag instead of DOCTYPE");
    {
      a += 9;
      let p = 1, g = !1, m = !1, N = "";
      for (; a < h.length; a++) if (h[a] !== "<" || m) if (h[a] === ">") {
        if (m ? h[a - 1] === "-" && h[a - 2] === "-" && (m = !1, p--) : p--, p === 0) break;
      } else h[a] === "[" ? g = !0 : N += h[a];
      else {
        if (g && l(h, a)) {
          let f, v;
          a += 7, [f, v, a] = o(h, a + 1), v.indexOf("&") === -1 && (d[s(f)] = { regx: RegExp(`&${f};`, "g"), val: v });
        } else if (g && u(h, a)) a += 8;
        else if (g && y(h, a)) a += 8;
        else if (g && c(h, a)) a += 9;
        else {
          if (!i) throw new Error("Invalid DOCTYPE");
          m = !0;
        }
        p++, N = "";
      }
      if (p !== 0) throw new Error("Unclosed DOCTYPE");
    }
    return { entities: d, i: a };
  };
}, 63: (r, t) => {
  const e = { preserveOrder: !1, attributeNamePrefix: "@_", attributesGroupName: !1, textNodeName: "#text", ignoreAttributes: !0, removeNSPrefix: !1, allowBooleanAttributes: !1, parseTagValue: !0, parseAttributeValue: !1, trimValues: !0, cdataPropName: !1, numberParseOptions: { hex: !0, leadingZeros: !0, eNotation: !0 }, tagValueProcessor: function(n, o) {
    return o;
  }, attributeValueProcessor: function(n, o) {
    return o;
  }, stopNodes: [], alwaysCreateTextNode: !1, isArray: () => !1, commentPropName: !1, unpairedTags: [], processEntities: !0, htmlEntities: !1, ignoreDeclaration: !1, ignorePiTags: !1, transformTagName: !1, transformAttributeName: !1, updateTag: function(n, o, i) {
    return n;
  } };
  t.buildOptions = function(n) {
    return Object.assign({}, e, n);
  }, t.defaultOptions = e;
}, 299: (r, t, e) => {
  const n = e(705), o = e(365), i = e(193), l = e(494), u = e(118);
  function y(b) {
    const O = Object.keys(b);
    for (let w = 0; w < O.length; w++) {
      const E = O[w];
      this.lastEntities[E] = { regex: new RegExp("&" + E + ";", "g"), val: b[E] };
    }
  }
  function c(b, O, w, E, j, S, $) {
    if (b !== void 0 && (this.options.trimValues && !E && (b = b.trim()), b.length > 0)) {
      $ || (b = this.replaceEntitiesValue(b));
      const P = this.options.tagValueProcessor(O, b, w, j, S);
      return P == null ? b : typeof P != typeof b || P !== b ? P : this.options.trimValues || b.trim() === b ? A(b, this.options.parseTagValue, this.options.numberParseOptions) : b;
    }
  }
  function s(b) {
    if (this.options.removeNSPrefix) {
      const O = b.split(":"), w = b.charAt(0) === "/" ? "/" : "";
      if (O[0] === "xmlns") return "";
      O.length === 2 && (b = w + O[1]);
    }
    return b;
  }
  const h = new RegExp(`([^\\s=]+)\\s*(=\\s*(['"])([\\s\\S]*?)\\3)?`, "gm");
  function a(b, O, w) {
    if (this.options.ignoreAttributes !== !0 && typeof b == "string") {
      const E = n.getAllMatches(b, h), j = E.length, S = {};
      for (let $ = 0; $ < j; $++) {
        const P = this.resolveNameSpace(E[$][1]);
        if (this.ignoreAttributesFn(P, O)) continue;
        let I = E[$][4], C = this.options.attributeNamePrefix + P;
        if (P.length) if (this.options.transformAttributeName && (C = this.options.transformAttributeName(C)), C === "__proto__" && (C = "#__proto__"), I !== void 0) {
          this.options.trimValues && (I = I.trim()), I = this.replaceEntitiesValue(I);
          const R = this.options.attributeValueProcessor(P, I, O);
          S[C] = R == null ? I : typeof R != typeof I || R !== I ? R : A(I, this.options.parseAttributeValue, this.options.numberParseOptions);
        } else this.options.allowBooleanAttributes && (S[C] = !0);
      }
      if (!Object.keys(S).length) return;
      if (this.options.attributesGroupName) {
        const $ = {};
        return $[this.options.attributesGroupName] = S, $;
      }
      return S;
    }
  }
  const d = function(b) {
    b = b.replace(/\r\n?/g, `
`);
    const O = new o("!xml");
    let w = O, E = "", j = "";
    for (let S = 0; S < b.length; S++) if (b[S] === "<") if (b[S + 1] === "/") {
      const $ = f(b, ">", S, "Closing Tag is not closed.");
      let P = b.substring(S + 2, $).trim();
      if (this.options.removeNSPrefix) {
        const R = P.indexOf(":");
        R !== -1 && (P = P.substr(R + 1));
      }
      this.options.transformTagName && (P = this.options.transformTagName(P)), w && (E = this.saveTextToParentTag(E, w, j));
      const I = j.substring(j.lastIndexOf(".") + 1);
      if (P && this.options.unpairedTags.indexOf(P) !== -1) throw new Error(`Unpaired tag can not be used as closing tag: </${P}>`);
      let C = 0;
      I && this.options.unpairedTags.indexOf(I) !== -1 ? (C = j.lastIndexOf(".", j.lastIndexOf(".") - 1), this.tagsNodeStack.pop()) : C = j.lastIndexOf("."), j = j.substring(0, C), w = this.tagsNodeStack.pop(), E = "", S = $;
    } else if (b[S + 1] === "?") {
      let $ = v(b, S, !1, "?>");
      if (!$) throw new Error("Pi Tag is not closed.");
      if (E = this.saveTextToParentTag(E, w, j), !(this.options.ignoreDeclaration && $.tagName === "?xml" || this.options.ignorePiTags)) {
        const P = new o($.tagName);
        P.add(this.options.textNodeName, ""), $.tagName !== $.tagExp && $.attrExpPresent && (P[":@"] = this.buildAttributesMap($.tagExp, j, $.tagName)), this.addChild(w, P, j);
      }
      S = $.closeIndex + 1;
    } else if (b.substr(S + 1, 3) === "!--") {
      const $ = f(b, "-->", S + 4, "Comment is not closed.");
      if (this.options.commentPropName) {
        const P = b.substring(S + 4, $ - 2);
        E = this.saveTextToParentTag(E, w, j), w.add(this.options.commentPropName, [{ [this.options.textNodeName]: P }]);
      }
      S = $;
    } else if (b.substr(S + 1, 2) === "!D") {
      const $ = i(b, S);
      this.docTypeEntities = $.entities, S = $.i;
    } else if (b.substr(S + 1, 2) === "![") {
      const $ = f(b, "]]>", S, "CDATA is not closed.") - 2, P = b.substring(S + 9, $);
      E = this.saveTextToParentTag(E, w, j);
      let I = this.parseTextData(P, w.tagname, j, !0, !1, !0, !0);
      I == null && (I = ""), this.options.cdataPropName ? w.add(this.options.cdataPropName, [{ [this.options.textNodeName]: P }]) : w.add(this.options.textNodeName, I), S = $ + 2;
    } else {
      let $ = v(b, S, this.options.removeNSPrefix), P = $.tagName;
      const I = $.rawTagName;
      let C = $.tagExp, R = $.attrExpPresent, Z = $.closeIndex;
      this.options.transformTagName && (P = this.options.transformTagName(P)), w && E && w.tagname !== "!xml" && (E = this.saveTextToParentTag(E, w, j, !1));
      const V = w;
      if (V && this.options.unpairedTags.indexOf(V.tagname) !== -1 && (w = this.tagsNodeStack.pop(), j = j.substring(0, j.lastIndexOf("."))), P !== O.tagname && (j += j ? "." + P : P), this.isItStopNode(this.options.stopNodes, j, P)) {
        let M = "";
        if (C.length > 0 && C.lastIndexOf("/") === C.length - 1) P[P.length - 1] === "/" ? (P = P.substr(0, P.length - 1), j = j.substr(0, j.length - 1), C = P) : C = C.substr(0, C.length - 1), S = $.closeIndex;
        else if (this.options.unpairedTags.indexOf(P) !== -1) S = $.closeIndex;
        else {
          const D = this.readStopNodeData(b, I, Z + 1);
          if (!D) throw new Error(`Unexpected end of ${I}`);
          S = D.i, M = D.tagContent;
        }
        const nt = new o(P);
        P !== C && R && (nt[":@"] = this.buildAttributesMap(C, j, P)), M && (M = this.parseTextData(M, P, j, !0, R, !0, !0)), j = j.substr(0, j.lastIndexOf(".")), nt.add(this.options.textNodeName, M), this.addChild(w, nt, j);
      } else {
        if (C.length > 0 && C.lastIndexOf("/") === C.length - 1) {
          P[P.length - 1] === "/" ? (P = P.substr(0, P.length - 1), j = j.substr(0, j.length - 1), C = P) : C = C.substr(0, C.length - 1), this.options.transformTagName && (P = this.options.transformTagName(P));
          const M = new o(P);
          P !== C && R && (M[":@"] = this.buildAttributesMap(C, j, P)), this.addChild(w, M, j), j = j.substr(0, j.lastIndexOf("."));
        } else {
          const M = new o(P);
          this.tagsNodeStack.push(w), P !== C && R && (M[":@"] = this.buildAttributesMap(C, j, P)), this.addChild(w, M, j), w = M;
        }
        E = "", S = Z;
      }
    }
    else E += b[S];
    return O.child;
  };
  function p(b, O, w) {
    const E = this.options.updateTag(O.tagname, w, O[":@"]);
    E === !1 || (typeof E == "string" && (O.tagname = E), b.addChild(O));
  }
  const g = function(b) {
    if (this.options.processEntities) {
      for (let O in this.docTypeEntities) {
        const w = this.docTypeEntities[O];
        b = b.replace(w.regx, w.val);
      }
      for (let O in this.lastEntities) {
        const w = this.lastEntities[O];
        b = b.replace(w.regex, w.val);
      }
      if (this.options.htmlEntities) for (let O in this.htmlEntities) {
        const w = this.htmlEntities[O];
        b = b.replace(w.regex, w.val);
      }
      b = b.replace(this.ampEntity.regex, this.ampEntity.val);
    }
    return b;
  };
  function m(b, O, w, E) {
    return b && (E === void 0 && (E = Object.keys(O.child).length === 0), (b = this.parseTextData(b, O.tagname, w, !1, !!O[":@"] && Object.keys(O[":@"]).length !== 0, E)) !== void 0 && b !== "" && O.add(this.options.textNodeName, b), b = ""), b;
  }
  function N(b, O, w) {
    const E = "*." + w;
    for (const j in b) {
      const S = b[j];
      if (E === S || O === S) return !0;
    }
    return !1;
  }
  function f(b, O, w, E) {
    const j = b.indexOf(O, w);
    if (j === -1) throw new Error(E);
    return j + O.length - 1;
  }
  function v(b, O, w) {
    const E = function(R, Z) {
      let V, M = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : ">", nt = "";
      for (let D = Z; D < R.length; D++) {
        let Y = R[D];
        if (V) Y === V && (V = "");
        else if (Y === '"' || Y === "'") V = Y;
        else if (Y === M[0]) {
          if (!M[1]) return { data: nt, index: D };
          if (R[D + 1] === M[1]) return { data: nt, index: D };
        } else Y === "	" && (Y = " ");
        nt += Y;
      }
    }(b, O + 1, arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : ">");
    if (!E) return;
    let j = E.data;
    const S = E.index, $ = j.search(/\s/);
    let P = j, I = !0;
    $ !== -1 && (P = j.substring(0, $), j = j.substring($ + 1).trimStart());
    const C = P;
    if (w) {
      const R = P.indexOf(":");
      R !== -1 && (P = P.substr(R + 1), I = P !== E.data.substr(R + 1));
    }
    return { tagName: P, tagExp: j, closeIndex: S, attrExpPresent: I, rawTagName: C };
  }
  function x(b, O, w) {
    const E = w;
    let j = 1;
    for (; w < b.length; w++) if (b[w] === "<") if (b[w + 1] === "/") {
      const S = f(b, ">", w, `${O} is not closed`);
      if (b.substring(w + 2, S).trim() === O && (j--, j === 0)) return { tagContent: b.substring(E, w), i: S };
      w = S;
    } else if (b[w + 1] === "?") w = f(b, "?>", w + 1, "StopNode is not closed.");
    else if (b.substr(w + 1, 3) === "!--") w = f(b, "-->", w + 3, "StopNode is not closed.");
    else if (b.substr(w + 1, 2) === "![") w = f(b, "]]>", w, "StopNode is not closed.") - 2;
    else {
      const S = v(b, w, ">");
      S && ((S && S.tagName) === O && S.tagExp[S.tagExp.length - 1] !== "/" && j++, w = S.closeIndex);
    }
  }
  function A(b, O, w) {
    if (O && typeof b == "string") {
      const E = b.trim();
      return E === "true" || E !== "false" && l(b, w);
    }
    return n.isExist(b) ? b : "";
  }
  r.exports = class {
    constructor(b) {
      this.options = b, this.currentNode = null, this.tagsNodeStack = [], this.docTypeEntities = {}, this.lastEntities = { apos: { regex: /&(apos|#39|#x27);/g, val: "'" }, gt: { regex: /&(gt|#62|#x3E);/g, val: ">" }, lt: { regex: /&(lt|#60|#x3C);/g, val: "<" }, quot: { regex: /&(quot|#34|#x22);/g, val: '"' } }, this.ampEntity = { regex: /&(amp|#38|#x26);/g, val: "&" }, this.htmlEntities = { space: { regex: /&(nbsp|#160);/g, val: " " }, cent: { regex: /&(cent|#162);/g, val: "¢" }, pound: { regex: /&(pound|#163);/g, val: "£" }, yen: { regex: /&(yen|#165);/g, val: "¥" }, euro: { regex: /&(euro|#8364);/g, val: "€" }, copyright: { regex: /&(copy|#169);/g, val: "©" }, reg: { regex: /&(reg|#174);/g, val: "®" }, inr: { regex: /&(inr|#8377);/g, val: "₹" }, num_dec: { regex: /&#([0-9]{1,7});/g, val: (O, w) => String.fromCharCode(Number.parseInt(w, 10)) }, num_hex: { regex: /&#x([0-9a-fA-F]{1,6});/g, val: (O, w) => String.fromCharCode(Number.parseInt(w, 16)) } }, this.addExternalEntities = y, this.parseXml = d, this.parseTextData = c, this.resolveNameSpace = s, this.buildAttributesMap = a, this.isItStopNode = N, this.replaceEntitiesValue = g, this.readStopNodeData = x, this.saveTextToParentTag = m, this.addChild = p, this.ignoreAttributesFn = u(this.options.ignoreAttributes);
    }
  };
}, 338: (r, t, e) => {
  const { buildOptions: n } = e(63), o = e(299), { prettify: i } = e(728), l = e(31);
  r.exports = class {
    constructor(u) {
      this.externalEntities = {}, this.options = n(u);
    }
    parse(u, y) {
      if (typeof u != "string") {
        if (!u.toString) throw new Error("XML data is accepted in String or Bytes[] form.");
        u = u.toString();
      }
      if (y) {
        y === !0 && (y = {});
        const h = l.validate(u, y);
        if (h !== !0) throw Error(`${h.err.msg}:${h.err.line}:${h.err.col}`);
      }
      const c = new o(this.options);
      c.addExternalEntities(this.externalEntities);
      const s = c.parseXml(u);
      return this.options.preserveOrder || s === void 0 ? s : i(s, this.options);
    }
    addEntity(u, y) {
      if (y.indexOf("&") !== -1) throw new Error("Entity value can't have '&'");
      if (u.indexOf("&") !== -1 || u.indexOf(";") !== -1) throw new Error("An entity must be set without '&' and ';'. Eg. use '#xD' for '&#xD;'");
      if (y === "&") throw new Error("An entity with value '&' is not permitted");
      this.externalEntities[u] = y;
    }
  };
}, 728: (r, t) => {
  function e(l, u, y) {
    let c;
    const s = {};
    for (let h = 0; h < l.length; h++) {
      const a = l[h], d = n(a);
      let p = "";
      if (p = y === void 0 ? d : y + "." + d, d === u.textNodeName) c === void 0 ? c = a[d] : c += "" + a[d];
      else {
        if (d === void 0) continue;
        if (a[d]) {
          let g = e(a[d], u, p);
          const m = i(g, u);
          a[":@"] ? o(g, a[":@"], p, u) : Object.keys(g).length !== 1 || g[u.textNodeName] === void 0 || u.alwaysCreateTextNode ? Object.keys(g).length === 0 && (u.alwaysCreateTextNode ? g[u.textNodeName] = "" : g = "") : g = g[u.textNodeName], s[d] !== void 0 && s.hasOwnProperty(d) ? (Array.isArray(s[d]) || (s[d] = [s[d]]), s[d].push(g)) : u.isArray(d, p, m) ? s[d] = [g] : s[d] = g;
        }
      }
    }
    return typeof c == "string" ? c.length > 0 && (s[u.textNodeName] = c) : c !== void 0 && (s[u.textNodeName] = c), s;
  }
  function n(l) {
    const u = Object.keys(l);
    for (let y = 0; y < u.length; y++) {
      const c = u[y];
      if (c !== ":@") return c;
    }
  }
  function o(l, u, y, c) {
    if (u) {
      const s = Object.keys(u), h = s.length;
      for (let a = 0; a < h; a++) {
        const d = s[a];
        c.isArray(d, y + "." + d, !0, !0) ? l[d] = [u[d]] : l[d] = u[d];
      }
    }
  }
  function i(l, u) {
    const { textNodeName: y } = u, c = Object.keys(l).length;
    return c === 0 || !(c !== 1 || !l[y] && typeof l[y] != "boolean" && l[y] !== 0);
  }
  t.prettify = function(l, u) {
    return e(l, u);
  };
}, 365: (r) => {
  r.exports = class {
    constructor(t) {
      this.tagname = t, this.child = [], this[":@"] = {};
    }
    add(t, e) {
      t === "__proto__" && (t = "#__proto__"), this.child.push({ [t]: e });
    }
    addChild(t) {
      t.tagname === "__proto__" && (t.tagname = "#__proto__"), t[":@"] && Object.keys(t[":@"]).length > 0 ? this.child.push({ [t.tagname]: t.child, ":@": t[":@"] }) : this.child.push({ [t.tagname]: t.child });
    }
  };
}, 135: (r) => {
  function t(e) {
    return !!e.constructor && typeof e.constructor.isBuffer == "function" && e.constructor.isBuffer(e);
  }
  r.exports = function(e) {
    return e != null && (t(e) || function(n) {
      return typeof n.readFloatLE == "function" && typeof n.slice == "function" && t(n.slice(0, 0));
    }(e) || !!e._isBuffer);
  };
}, 542: (r, t, e) => {
  (function() {
    var n = e(298), o = e(526).utf8, i = e(135), l = e(526).bin, u = function(y, c) {
      y.constructor == String ? y = c && c.encoding === "binary" ? l.stringToBytes(y) : o.stringToBytes(y) : i(y) ? y = Array.prototype.slice.call(y, 0) : Array.isArray(y) || y.constructor === Uint8Array || (y = y.toString());
      for (var s = n.bytesToWords(y), h = 8 * y.length, a = 1732584193, d = -271733879, p = -1732584194, g = 271733878, m = 0; m < s.length; m++) s[m] = 16711935 & (s[m] << 8 | s[m] >>> 24) | 4278255360 & (s[m] << 24 | s[m] >>> 8);
      s[h >>> 5] |= 128 << h % 32, s[14 + (h + 64 >>> 9 << 4)] = h;
      var N = u._ff, f = u._gg, v = u._hh, x = u._ii;
      for (m = 0; m < s.length; m += 16) {
        var A = a, b = d, O = p, w = g;
        a = N(a, d, p, g, s[m + 0], 7, -680876936), g = N(g, a, d, p, s[m + 1], 12, -389564586), p = N(p, g, a, d, s[m + 2], 17, 606105819), d = N(d, p, g, a, s[m + 3], 22, -1044525330), a = N(a, d, p, g, s[m + 4], 7, -176418897), g = N(g, a, d, p, s[m + 5], 12, 1200080426), p = N(p, g, a, d, s[m + 6], 17, -1473231341), d = N(d, p, g, a, s[m + 7], 22, -45705983), a = N(a, d, p, g, s[m + 8], 7, 1770035416), g = N(g, a, d, p, s[m + 9], 12, -1958414417), p = N(p, g, a, d, s[m + 10], 17, -42063), d = N(d, p, g, a, s[m + 11], 22, -1990404162), a = N(a, d, p, g, s[m + 12], 7, 1804603682), g = N(g, a, d, p, s[m + 13], 12, -40341101), p = N(p, g, a, d, s[m + 14], 17, -1502002290), a = f(a, d = N(d, p, g, a, s[m + 15], 22, 1236535329), p, g, s[m + 1], 5, -165796510), g = f(g, a, d, p, s[m + 6], 9, -1069501632), p = f(p, g, a, d, s[m + 11], 14, 643717713), d = f(d, p, g, a, s[m + 0], 20, -373897302), a = f(a, d, p, g, s[m + 5], 5, -701558691), g = f(g, a, d, p, s[m + 10], 9, 38016083), p = f(p, g, a, d, s[m + 15], 14, -660478335), d = f(d, p, g, a, s[m + 4], 20, -405537848), a = f(a, d, p, g, s[m + 9], 5, 568446438), g = f(g, a, d, p, s[m + 14], 9, -1019803690), p = f(p, g, a, d, s[m + 3], 14, -187363961), d = f(d, p, g, a, s[m + 8], 20, 1163531501), a = f(a, d, p, g, s[m + 13], 5, -1444681467), g = f(g, a, d, p, s[m + 2], 9, -51403784), p = f(p, g, a, d, s[m + 7], 14, 1735328473), a = v(a, d = f(d, p, g, a, s[m + 12], 20, -1926607734), p, g, s[m + 5], 4, -378558), g = v(g, a, d, p, s[m + 8], 11, -2022574463), p = v(p, g, a, d, s[m + 11], 16, 1839030562), d = v(d, p, g, a, s[m + 14], 23, -35309556), a = v(a, d, p, g, s[m + 1], 4, -1530992060), g = v(g, a, d, p, s[m + 4], 11, 1272893353), p = v(p, g, a, d, s[m + 7], 16, -155497632), d = v(d, p, g, a, s[m + 10], 23, -1094730640), a = v(a, d, p, g, s[m + 13], 4, 681279174), g = v(g, a, d, p, s[m + 0], 11, -358537222), p = v(p, g, a, d, s[m + 3], 16, -722521979), d = v(d, p, g, a, s[m + 6], 23, 76029189), a = v(a, d, p, g, s[m + 9], 4, -640364487), g = v(g, a, d, p, s[m + 12], 11, -421815835), p = v(p, g, a, d, s[m + 15], 16, 530742520), a = x(a, d = v(d, p, g, a, s[m + 2], 23, -995338651), p, g, s[m + 0], 6, -198630844), g = x(g, a, d, p, s[m + 7], 10, 1126891415), p = x(p, g, a, d, s[m + 14], 15, -1416354905), d = x(d, p, g, a, s[m + 5], 21, -57434055), a = x(a, d, p, g, s[m + 12], 6, 1700485571), g = x(g, a, d, p, s[m + 3], 10, -1894986606), p = x(p, g, a, d, s[m + 10], 15, -1051523), d = x(d, p, g, a, s[m + 1], 21, -2054922799), a = x(a, d, p, g, s[m + 8], 6, 1873313359), g = x(g, a, d, p, s[m + 15], 10, -30611744), p = x(p, g, a, d, s[m + 6], 15, -1560198380), d = x(d, p, g, a, s[m + 13], 21, 1309151649), a = x(a, d, p, g, s[m + 4], 6, -145523070), g = x(g, a, d, p, s[m + 11], 10, -1120210379), p = x(p, g, a, d, s[m + 2], 15, 718787259), d = x(d, p, g, a, s[m + 9], 21, -343485551), a = a + A >>> 0, d = d + b >>> 0, p = p + O >>> 0, g = g + w >>> 0;
      }
      return n.endian([a, d, p, g]);
    };
    u._ff = function(y, c, s, h, a, d, p) {
      var g = y + (c & s | ~c & h) + (a >>> 0) + p;
      return (g << d | g >>> 32 - d) + c;
    }, u._gg = function(y, c, s, h, a, d, p) {
      var g = y + (c & h | s & ~h) + (a >>> 0) + p;
      return (g << d | g >>> 32 - d) + c;
    }, u._hh = function(y, c, s, h, a, d, p) {
      var g = y + (c ^ s ^ h) + (a >>> 0) + p;
      return (g << d | g >>> 32 - d) + c;
    }, u._ii = function(y, c, s, h, a, d, p) {
      var g = y + (s ^ (c | ~h)) + (a >>> 0) + p;
      return (g << d | g >>> 32 - d) + c;
    }, u._blocksize = 16, u._digestsize = 16, r.exports = function(y, c) {
      if (y == null) throw new Error("Illegal argument " + y);
      var s = n.wordsToBytes(u(y, c));
      return c && c.asBytes ? s : c && c.asString ? l.bytesToString(s) : n.bytesToHex(s);
    };
  })();
}, 285: (r, t, e) => {
  var n = e(2);
  r.exports = function(N) {
    return N ? (N.substr(0, 2) === "{}" && (N = "\\{\\}" + N.substr(2)), m(function(f) {
      return f.split("\\\\").join(o).split("\\{").join(i).split("\\}").join(l).split("\\,").join(u).split("\\.").join(y);
    }(N), !0).map(s)) : [];
  };
  var o = "\0SLASH" + Math.random() + "\0", i = "\0OPEN" + Math.random() + "\0", l = "\0CLOSE" + Math.random() + "\0", u = "\0COMMA" + Math.random() + "\0", y = "\0PERIOD" + Math.random() + "\0";
  function c(N) {
    return parseInt(N, 10) == N ? parseInt(N, 10) : N.charCodeAt(0);
  }
  function s(N) {
    return N.split(o).join("\\").split(i).join("{").split(l).join("}").split(u).join(",").split(y).join(".");
  }
  function h(N) {
    if (!N) return [""];
    var f = [], v = n("{", "}", N);
    if (!v) return N.split(",");
    var x = v.pre, A = v.body, b = v.post, O = x.split(",");
    O[O.length - 1] += "{" + A + "}";
    var w = h(b);
    return b.length && (O[O.length - 1] += w.shift(), O.push.apply(O, w)), f.push.apply(f, O), f;
  }
  function a(N) {
    return "{" + N + "}";
  }
  function d(N) {
    return /^-?0\d/.test(N);
  }
  function p(N, f) {
    return N <= f;
  }
  function g(N, f) {
    return N >= f;
  }
  function m(N, f) {
    var v = [], x = n("{", "}", N);
    if (!x) return [N];
    var A = x.pre, b = x.post.length ? m(x.post, !1) : [""];
    if (/\$$/.test(x.pre)) for (var O = 0; O < b.length; O++) {
      var w = A + "{" + x.body + "}" + b[O];
      v.push(w);
    }
    else {
      var E, j, S = /^-?\d+\.\.-?\d+(?:\.\.-?\d+)?$/.test(x.body), $ = /^[a-zA-Z]\.\.[a-zA-Z](?:\.\.-?\d+)?$/.test(x.body), P = S || $, I = x.body.indexOf(",") >= 0;
      if (!P && !I) return x.post.match(/,.*\}/) ? m(N = x.pre + "{" + x.body + l + x.post) : [N];
      if (P) E = x.body.split(/\.\./);
      else if ((E = h(x.body)).length === 1 && (E = m(E[0], !1).map(a)).length === 1) return b.map(function(Je) {
        return x.pre + E[0] + Je;
      });
      if (P) {
        var C = c(E[0]), R = c(E[1]), Z = Math.max(E[0].length, E[1].length), V = E.length == 3 ? Math.abs(c(E[2])) : 1, M = p;
        R < C && (V *= -1, M = g);
        var nt = E.some(d);
        j = [];
        for (var D = C; M(D, R); D += V) {
          var Y;
          if ($) (Y = String.fromCharCode(D)) === "\\" && (Y = "");
          else if (Y = String(D), nt) {
            var ue = Z - Y.length;
            if (ue > 0) {
              var le = new Array(ue + 1).join("0");
              Y = D < 0 ? "-" + le + Y.slice(1) : le + Y;
            }
          }
          j.push(Y);
        }
      } else {
        j = [];
        for (var dt = 0; dt < E.length; dt++) j.push.apply(j, m(E[dt], !1));
      }
      for (dt = 0; dt < j.length; dt++) for (O = 0; O < b.length; O++) w = A + j[dt] + b[O], (!f || P || w) && v.push(w);
    }
    return v;
  }
}, 829: (r) => {
  function t(c) {
    return t = typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? function(s) {
      return typeof s;
    } : function(s) {
      return s && typeof Symbol == "function" && s.constructor === Symbol && s !== Symbol.prototype ? "symbol" : typeof s;
    }, t(c);
  }
  function e(c) {
    var s = typeof Map == "function" ? /* @__PURE__ */ new Map() : void 0;
    return e = function(h) {
      if (h === null || (a = h, Function.toString.call(a).indexOf("[native code]") === -1)) return h;
      var a;
      if (typeof h != "function") throw new TypeError("Super expression must either be null or a function");
      if (s !== void 0) {
        if (s.has(h)) return s.get(h);
        s.set(h, d);
      }
      function d() {
        return n(h, arguments, i(this).constructor);
      }
      return d.prototype = Object.create(h.prototype, { constructor: { value: d, enumerable: !1, writable: !0, configurable: !0 } }), o(d, h);
    }, e(c);
  }
  function n(c, s, h) {
    return n = function() {
      if (typeof Reflect > "u" || !Reflect.construct || Reflect.construct.sham) return !1;
      if (typeof Proxy == "function") return !0;
      try {
        return Date.prototype.toString.call(Reflect.construct(Date, [], function() {
        })), !0;
      } catch {
        return !1;
      }
    }() ? Reflect.construct : function(a, d, p) {
      var g = [null];
      g.push.apply(g, d);
      var m = new (Function.bind.apply(a, g))();
      return p && o(m, p.prototype), m;
    }, n.apply(null, arguments);
  }
  function o(c, s) {
    return o = Object.setPrototypeOf || function(h, a) {
      return h.__proto__ = a, h;
    }, o(c, s);
  }
  function i(c) {
    return i = Object.setPrototypeOf ? Object.getPrototypeOf : function(s) {
      return s.__proto__ || Object.getPrototypeOf(s);
    }, i(c);
  }
  var l = function(c) {
    function s(h) {
      var a;
      return function(d, p) {
        if (!(d instanceof p)) throw new TypeError("Cannot call a class as a function");
      }(this, s), (a = function(d, p) {
        return !p || t(p) !== "object" && typeof p != "function" ? function(g) {
          if (g === void 0) throw new ReferenceError("this hasn't been initialised - super() hasn't been called");
          return g;
        }(d) : p;
      }(this, i(s).call(this, h))).name = "ObjectPrototypeMutationError", a;
    }
    return function(h, a) {
      if (typeof a != "function" && a !== null) throw new TypeError("Super expression must either be null or a function");
      h.prototype = Object.create(a && a.prototype, { constructor: { value: h, writable: !0, configurable: !0 } }), a && o(h, a);
    }(s, c), s;
  }(e(Error));
  function u(c, s) {
    for (var h = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : function() {
    }, a = s.split("."), d = a.length, p = function(N) {
      var f = a[N];
      if (!c) return { v: void 0 };
      if (f === "+") {
        if (Array.isArray(c)) return { v: c.map(function(x, A) {
          var b = a.slice(N + 1);
          return b.length > 0 ? u(x, b.join("."), h) : h(c, A, a, N);
        }) };
        var v = a.slice(0, N).join(".");
        throw new Error("Object at wildcard (".concat(v, ") is not an array"));
      }
      c = h(c, f, a, N);
    }, g = 0; g < d; g++) {
      var m = p(g);
      if (t(m) === "object") return m.v;
    }
    return c;
  }
  function y(c, s) {
    return c.length === s + 1;
  }
  r.exports = { set: function(c, s, h) {
    if (t(c) != "object" || c === null || s === void 0) return c;
    if (typeof s == "number") return c[s] = h, c[s];
    try {
      return u(c, s, function(a, d, p, g) {
        if (a === Reflect.getPrototypeOf({})) throw new l("Attempting to mutate Object.prototype");
        if (!a[d]) {
          var m = Number.isInteger(Number(p[g + 1])), N = p[g + 1] === "+";
          a[d] = m || N ? [] : {};
        }
        return y(p, g) && (a[d] = h), a[d];
      });
    } catch (a) {
      if (a instanceof l) throw a;
      return c;
    }
  }, get: function(c, s) {
    if (t(c) != "object" || c === null || s === void 0) return c;
    if (typeof s == "number") return c[s];
    try {
      return u(c, s, function(h, a) {
        return h[a];
      });
    } catch {
      return c;
    }
  }, has: function(c, s) {
    var h = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
    if (t(c) != "object" || c === null || s === void 0) return !1;
    if (typeof s == "number") return s in c;
    try {
      var a = !1;
      return u(c, s, function(d, p, g, m) {
        if (!y(g, m)) return d && d[p];
        a = h.own ? d.hasOwnProperty(p) : p in d;
      }), a;
    } catch {
      return !1;
    }
  }, hasOwn: function(c, s, h) {
    return this.has(c, s, h || { own: !0 });
  }, isIn: function(c, s, h) {
    var a = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : {};
    if (t(c) != "object" || c === null || s === void 0) return !1;
    try {
      var d = !1, p = !1;
      return u(c, s, function(g, m, N, f) {
        return d = d || g === h || !!g && g[m] === h, p = y(N, f) && t(g) === "object" && m in g, g && g[m];
      }), a.validPath ? d && p : d;
    } catch {
      return !1;
    }
  }, ObjectPrototypeMutationError: l };
}, 47: (r, t, e) => {
  var n = e(410), o = function(c) {
    return typeof c == "string";
  };
  function i(c, s) {
    for (var h = [], a = 0; a < c.length; a++) {
      var d = c[a];
      d && d !== "." && (d === ".." ? h.length && h[h.length - 1] !== ".." ? h.pop() : s && h.push("..") : h.push(d));
    }
    return h;
  }
  var l = /^(\/?|)([\s\S]*?)((?:\.{1,2}|[^\/]+?|)(\.[^.\/]*|))(?:[\/]*)$/, u = {};
  function y(c) {
    return l.exec(c).slice(1);
  }
  u.resolve = function() {
    for (var c = "", s = !1, h = arguments.length - 1; h >= -1 && !s; h--) {
      var a = h >= 0 ? arguments[h] : process.cwd();
      if (!o(a)) throw new TypeError("Arguments to path.resolve must be strings");
      a && (c = a + "/" + c, s = a.charAt(0) === "/");
    }
    return (s ? "/" : "") + (c = i(c.split("/"), !s).join("/")) || ".";
  }, u.normalize = function(c) {
    var s = u.isAbsolute(c), h = c.substr(-1) === "/";
    return (c = i(c.split("/"), !s).join("/")) || s || (c = "."), c && h && (c += "/"), (s ? "/" : "") + c;
  }, u.isAbsolute = function(c) {
    return c.charAt(0) === "/";
  }, u.join = function() {
    for (var c = "", s = 0; s < arguments.length; s++) {
      var h = arguments[s];
      if (!o(h)) throw new TypeError("Arguments to path.join must be strings");
      h && (c += c ? "/" + h : h);
    }
    return u.normalize(c);
  }, u.relative = function(c, s) {
    function h(f) {
      for (var v = 0; v < f.length && f[v] === ""; v++) ;
      for (var x = f.length - 1; x >= 0 && f[x] === ""; x--) ;
      return v > x ? [] : f.slice(v, x + 1);
    }
    c = u.resolve(c).substr(1), s = u.resolve(s).substr(1);
    for (var a = h(c.split("/")), d = h(s.split("/")), p = Math.min(a.length, d.length), g = p, m = 0; m < p; m++) if (a[m] !== d[m]) {
      g = m;
      break;
    }
    var N = [];
    for (m = g; m < a.length; m++) N.push("..");
    return (N = N.concat(d.slice(g))).join("/");
  }, u._makeLong = function(c) {
    return c;
  }, u.dirname = function(c) {
    var s = y(c), h = s[0], a = s[1];
    return h || a ? (a && (a = a.substr(0, a.length - 1)), h + a) : ".";
  }, u.basename = function(c, s) {
    var h = y(c)[2];
    return s && h.substr(-1 * s.length) === s && (h = h.substr(0, h.length - s.length)), h;
  }, u.extname = function(c) {
    return y(c)[3];
  }, u.format = function(c) {
    if (!n.isObject(c)) throw new TypeError("Parameter 'pathObject' must be an object, not " + typeof c);
    var s = c.root || "";
    if (!o(s)) throw new TypeError("'pathObject.root' must be a string or undefined, not " + typeof c.root);
    return (c.dir ? c.dir + u.sep : "") + (c.base || "");
  }, u.parse = function(c) {
    if (!o(c)) throw new TypeError("Parameter 'pathString' must be a string, not " + typeof c);
    var s = y(c);
    if (!s || s.length !== 4) throw new TypeError("Invalid path '" + c + "'");
    return s[1] = s[1] || "", s[2] = s[2] || "", s[3] = s[3] || "", { root: s[0], dir: s[0] + s[1].slice(0, s[1].length - 1), base: s[2], ext: s[3], name: s[2].slice(0, s[2].length - s[3].length) };
  }, u.sep = "/", u.delimiter = ":", r.exports = u;
}, 647: (r, t) => {
  var e = Object.prototype.hasOwnProperty;
  function n(i) {
    try {
      return decodeURIComponent(i.replace(/\+/g, " "));
    } catch {
      return null;
    }
  }
  function o(i) {
    try {
      return encodeURIComponent(i);
    } catch {
      return null;
    }
  }
  t.stringify = function(i, l) {
    l = l || "";
    var u, y, c = [];
    for (y in typeof l != "string" && (l = "?"), i) if (e.call(i, y)) {
      if ((u = i[y]) || u != null && !isNaN(u) || (u = ""), y = o(y), u = o(u), y === null || u === null) continue;
      c.push(y + "=" + u);
    }
    return c.length ? l + c.join("&") : "";
  }, t.parse = function(i) {
    for (var l, u = /([^=?#&]+)=?([^&]*)/g, y = {}; l = u.exec(i); ) {
      var c = n(l[1]), s = n(l[2]);
      c === null || s === null || c in y || (y[c] = s);
    }
    return y;
  };
}, 670: (r) => {
  r.exports = function(t, e) {
    if (e = e.split(":")[0], !(t = +t)) return !1;
    switch (e) {
      case "http":
      case "ws":
        return t !== 80;
      case "https":
      case "wss":
        return t !== 443;
      case "ftp":
        return t !== 21;
      case "gopher":
        return t !== 70;
      case "file":
        return !1;
    }
    return t !== 0;
  };
}, 494: (r) => {
  const t = /^[-+]?0x[a-fA-F0-9]+$/, e = /^([\-\+])?(0*)(\.[0-9]+([eE]\-?[0-9]+)?|[0-9]+(\.[0-9]+([eE]\-?[0-9]+)?)?)$/;
  !Number.parseInt && window.parseInt && (Number.parseInt = window.parseInt), !Number.parseFloat && window.parseFloat && (Number.parseFloat = window.parseFloat);
  const n = { hex: !0, leadingZeros: !0, decimalPoint: ".", eNotation: !0 };
  r.exports = function(o) {
    let i = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    if (i = Object.assign({}, n, i), !o || typeof o != "string") return o;
    let l = o.trim();
    if (i.skipLike !== void 0 && i.skipLike.test(l)) return o;
    if (i.hex && t.test(l)) return Number.parseInt(l, 16);
    {
      const y = e.exec(l);
      if (y) {
        const c = y[1], s = y[2];
        let h = ((u = y[3]) && u.indexOf(".") !== -1 && ((u = u.replace(/0+$/, "")) === "." ? u = "0" : u[0] === "." ? u = "0" + u : u[u.length - 1] === "." && (u = u.substr(0, u.length - 1))), u);
        const a = y[4] || y[6];
        if (!i.leadingZeros && s.length > 0 && c && l[2] !== "." || !i.leadingZeros && s.length > 0 && !c && l[1] !== ".") return o;
        {
          const d = Number(l), p = "" + d;
          return p.search(/[eE]/) !== -1 || a ? i.eNotation ? d : o : l.indexOf(".") !== -1 ? p === "0" && h === "" || p === h || c && p === "-" + h ? d : o : s ? h === p || c + h === p ? d : o : l === p || l === c + p ? d : o;
        }
      }
      return o;
    }
    var u;
  };
}, 737: (r, t, e) => {
  var n = e(670), o = e(647), i = /^[\x00-\x20\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000\ufeff]+/, l = /[\n\r\t]/g, u = /^[A-Za-z][A-Za-z0-9+-.]*:\/\//, y = /:\d+$/, c = /^([a-z][a-z0-9.+-]*:)?(\/\/)?([\\/]+)?([\S\s]*)/i, s = /^[a-zA-Z]:/;
  function h(f) {
    return (f || "").toString().replace(i, "");
  }
  var a = [["#", "hash"], ["?", "query"], function(f, v) {
    return g(v.protocol) ? f.replace(/\\/g, "/") : f;
  }, ["/", "pathname"], ["@", "auth", 1], [NaN, "host", void 0, 1, 1], [/:(\d*)$/, "port", void 0, 1], [NaN, "hostname", void 0, 1, 1]], d = { hash: 1, query: 1 };
  function p(f) {
    var v, x = (typeof window < "u" ? window : typeof global < "u" ? global : typeof self < "u" ? self : {}).location || {}, A = {}, b = typeof (f = f || x);
    if (f.protocol === "blob:") A = new N(unescape(f.pathname), {});
    else if (b === "string") for (v in A = new N(f, {}), d) delete A[v];
    else if (b === "object") {
      for (v in f) v in d || (A[v] = f[v]);
      A.slashes === void 0 && (A.slashes = u.test(f.href));
    }
    return A;
  }
  function g(f) {
    return f === "file:" || f === "ftp:" || f === "http:" || f === "https:" || f === "ws:" || f === "wss:";
  }
  function m(f, v) {
    f = (f = h(f)).replace(l, ""), v = v || {};
    var x, A = c.exec(f), b = A[1] ? A[1].toLowerCase() : "", O = !!A[2], w = !!A[3], E = 0;
    return O ? w ? (x = A[2] + A[3] + A[4], E = A[2].length + A[3].length) : (x = A[2] + A[4], E = A[2].length) : w ? (x = A[3] + A[4], E = A[3].length) : x = A[4], b === "file:" ? E >= 2 && (x = x.slice(2)) : g(b) ? x = A[4] : b ? O && (x = x.slice(2)) : E >= 2 && g(v.protocol) && (x = A[4]), { protocol: b, slashes: O || g(b), slashesCount: E, rest: x };
  }
  function N(f, v, x) {
    if (f = (f = h(f)).replace(l, ""), !(this instanceof N)) return new N(f, v, x);
    var A, b, O, w, E, j, S = a.slice(), $ = typeof v, P = this, I = 0;
    for ($ !== "object" && $ !== "string" && (x = v, v = null), x && typeof x != "function" && (x = o.parse), A = !(b = m(f || "", v = p(v))).protocol && !b.slashes, P.slashes = b.slashes || A && v.slashes, P.protocol = b.protocol || v.protocol || "", f = b.rest, (b.protocol === "file:" && (b.slashesCount !== 2 || s.test(f)) || !b.slashes && (b.protocol || b.slashesCount < 2 || !g(P.protocol))) && (S[3] = [/(.*)/, "pathname"]); I < S.length; I++) typeof (w = S[I]) != "function" ? (O = w[0], j = w[1], O != O ? P[j] = f : typeof O == "string" ? ~(E = O === "@" ? f.lastIndexOf(O) : f.indexOf(O)) && (typeof w[2] == "number" ? (P[j] = f.slice(0, E), f = f.slice(E + w[2])) : (P[j] = f.slice(E), f = f.slice(0, E))) : (E = O.exec(f)) && (P[j] = E[1], f = f.slice(0, E.index)), P[j] = P[j] || A && w[3] && v[j] || "", w[4] && (P[j] = P[j].toLowerCase())) : f = w(f, P);
    x && (P.query = x(P.query)), A && v.slashes && P.pathname.charAt(0) !== "/" && (P.pathname !== "" || v.pathname !== "") && (P.pathname = function(C, R) {
      if (C === "") return R;
      for (var Z = (R || "/").split("/").slice(0, -1).concat(C.split("/")), V = Z.length, M = Z[V - 1], nt = !1, D = 0; V--; ) Z[V] === "." ? Z.splice(V, 1) : Z[V] === ".." ? (Z.splice(V, 1), D++) : D && (V === 0 && (nt = !0), Z.splice(V, 1), D--);
      return nt && Z.unshift(""), M !== "." && M !== ".." || Z.push(""), Z.join("/");
    }(P.pathname, v.pathname)), P.pathname.charAt(0) !== "/" && g(P.protocol) && (P.pathname = "/" + P.pathname), n(P.port, P.protocol) || (P.host = P.hostname, P.port = ""), P.username = P.password = "", P.auth && (~(E = P.auth.indexOf(":")) ? (P.username = P.auth.slice(0, E), P.username = encodeURIComponent(decodeURIComponent(P.username)), P.password = P.auth.slice(E + 1), P.password = encodeURIComponent(decodeURIComponent(P.password))) : P.username = encodeURIComponent(decodeURIComponent(P.auth)), P.auth = P.password ? P.username + ":" + P.password : P.username), P.origin = P.protocol !== "file:" && g(P.protocol) && P.host ? P.protocol + "//" + P.host : "null", P.href = P.toString();
  }
  N.prototype = { set: function(f, v, x) {
    var A = this;
    switch (f) {
      case "query":
        typeof v == "string" && v.length && (v = (x || o.parse)(v)), A[f] = v;
        break;
      case "port":
        A[f] = v, n(v, A.protocol) ? v && (A.host = A.hostname + ":" + v) : (A.host = A.hostname, A[f] = "");
        break;
      case "hostname":
        A[f] = v, A.port && (v += ":" + A.port), A.host = v;
        break;
      case "host":
        A[f] = v, y.test(v) ? (v = v.split(":"), A.port = v.pop(), A.hostname = v.join(":")) : (A.hostname = v, A.port = "");
        break;
      case "protocol":
        A.protocol = v.toLowerCase(), A.slashes = !x;
        break;
      case "pathname":
      case "hash":
        if (v) {
          var b = f === "pathname" ? "/" : "#";
          A[f] = v.charAt(0) !== b ? b + v : v;
        } else A[f] = v;
        break;
      case "username":
      case "password":
        A[f] = encodeURIComponent(v);
        break;
      case "auth":
        var O = v.indexOf(":");
        ~O ? (A.username = v.slice(0, O), A.username = encodeURIComponent(decodeURIComponent(A.username)), A.password = v.slice(O + 1), A.password = encodeURIComponent(decodeURIComponent(A.password))) : A.username = encodeURIComponent(decodeURIComponent(v));
    }
    for (var w = 0; w < a.length; w++) {
      var E = a[w];
      E[4] && (A[E[1]] = A[E[1]].toLowerCase());
    }
    return A.auth = A.password ? A.username + ":" + A.password : A.username, A.origin = A.protocol !== "file:" && g(A.protocol) && A.host ? A.protocol + "//" + A.host : "null", A.href = A.toString(), A;
  }, toString: function(f) {
    f && typeof f == "function" || (f = o.stringify);
    var v, x = this, A = x.host, b = x.protocol;
    b && b.charAt(b.length - 1) !== ":" && (b += ":");
    var O = b + (x.protocol && x.slashes || g(x.protocol) ? "//" : "");
    return x.username ? (O += x.username, x.password && (O += ":" + x.password), O += "@") : x.password ? (O += ":" + x.password, O += "@") : x.protocol !== "file:" && g(x.protocol) && !A && x.pathname !== "/" && (O += "@"), (A[A.length - 1] === ":" || y.test(x.hostname) && !x.port) && (A += ":"), O += A + x.pathname, (v = typeof x.query == "object" ? f(x.query) : x.query) && (O += v.charAt(0) !== "?" ? "?" + v : v), x.hash && (O += x.hash), O;
  } }, N.extractProtocol = m, N.location = p, N.trimLeft = h, N.qs = o, r.exports = N;
}, 410: () => {
}, 388: () => {
}, 805: () => {
}, 345: () => {
}, 800: () => {
} }, he = {};
function k(r) {
  var t = he[r];
  if (t !== void 0) return t.exports;
  var e = he[r] = { id: r, loaded: !1, exports: {} };
  return er[r].call(e.exports, e, e.exports, k), e.loaded = !0, e.exports;
}
k.n = (r) => {
  var t = r && r.__esModule ? () => r.default : () => r;
  return k.d(t, { a: t }), t;
}, k.d = (r, t) => {
  for (var e in t) k.o(t, e) && !k.o(r, e) && Object.defineProperty(r, e, { enumerable: !0, get: t[e] });
}, k.o = (r, t) => Object.prototype.hasOwnProperty.call(r, t), k.nmd = (r) => (r.paths = [], r.children || (r.children = []), r);
var it = {};
k.d(it, { hT: () => Q, O4: () => ct, Kd: () => cr, YK: () => hr, UU: () => vn, Gu: () => Re, ky: () => Ge, h4: () => jt, ch: () => Ft, hq: () => bt, i5: () => qe });
var rr = k(737), nr = k.n(rr);
function Bt(r) {
  if (!Zt(r)) throw new Error("Parameter was not an error");
}
function Zt(r) {
  return !!r && typeof r == "object" && (t = r, Object.prototype.toString.call(t) === "[object Error]") || r instanceof Error;
  var t;
}
class et extends Error {
  constructor(t, e) {
    const n = [...arguments], { options: o, shortMessage: i } = function(u) {
      let y, c = "";
      if (u.length === 0) y = {};
      else if (Zt(u[0])) y = { cause: u[0] }, c = u.slice(1).join(" ") || "";
      else if (u[0] && typeof u[0] == "object") y = Object.assign({}, u[0]), c = u.slice(1).join(" ") || "";
      else {
        if (typeof u[0] != "string") throw new Error("Invalid arguments passed to Layerr");
        y = {}, c = c = u.join(" ") || "";
      }
      return { options: y, shortMessage: c };
    }(n);
    let l = i;
    if (o.cause && (l = `${l}: ${o.cause.message}`), super(l), this.message = l, o.name && typeof o.name == "string" ? this.name = o.name : this.name = "Layerr", o.cause && Object.defineProperty(this, "_cause", { value: o.cause }), Object.defineProperty(this, "_info", { value: {} }), o.info && typeof o.info == "object" && Object.assign(this._info, o.info), Error.captureStackTrace) {
      const u = o.constructorOpt || this.constructor;
      Error.captureStackTrace(this, u);
    }
  }
  static cause(t) {
    return Bt(t), t._cause && Zt(t._cause) ? t._cause : null;
  }
  static fullStack(t) {
    Bt(t);
    const e = et.cause(t);
    return e ? `${t.stack}
caused by: ${et.fullStack(e)}` : t.stack ?? "";
  }
  static info(t) {
    Bt(t);
    const e = {}, n = et.cause(t);
    return n && Object.assign(e, et.info(n)), t._info && Object.assign(e, t._info), e;
  }
  toString() {
    let t = this.name || this.constructor.name || this.constructor.prototype.name;
    return this.message && (t = `${t}: ${this.message}`), t;
  }
}
var sr = k(47), Ct = k.n(sr);
const fe = "__PATH_SEPARATOR_POSIX__", pe = "__PATH_SEPARATOR_WINDOWS__";
function _(r) {
  try {
    const t = r.replace(/\//g, fe).replace(/\\\\/g, pe);
    return encodeURIComponent(t).split(pe).join("\\\\").split(fe).join("/");
  } catch (t) {
    throw new et(t, "Failed encoding path");
  }
}
function ge(r) {
  return r.startsWith("/") ? r : "/" + r;
}
function Et(r) {
  let t = r;
  return t[0] !== "/" && (t = "/" + t), /^.+\/$/.test(t) && (t = t.substr(0, t.length - 1)), t;
}
function or(r) {
  let t = new (nr())(r).pathname;
  return t.length <= 0 && (t = "/"), Et(t);
}
function U() {
  for (var r = arguments.length, t = new Array(r), e = 0; e < r; e++) t[e] = arguments[e];
  return function() {
    return function(n) {
      var o = [];
      if (n.length === 0) return "";
      if (typeof n[0] != "string") throw new TypeError("Url must be a string. Received " + n[0]);
      if (n[0].match(/^[^/:]+:\/*$/) && n.length > 1) {
        var i = n.shift();
        n[0] = i + n[0];
      }
      n[0].match(/^file:\/\/\//) ? n[0] = n[0].replace(/^([^/:]+):\/*/, "$1:///") : n[0] = n[0].replace(/^([^/:]+):\/*/, "$1://");
      for (var l = 0; l < n.length; l++) {
        var u = n[l];
        if (typeof u != "string") throw new TypeError("Url must be a string. Received " + u);
        u !== "" && (l > 0 && (u = u.replace(/^[\/]+/, "")), u = l < n.length - 1 ? u.replace(/[\/]+$/, "") : u.replace(/[\/]+$/, "/"), o.push(u));
      }
      var y = o.join("/"), c = (y = y.replace(/\/(\?|&|#[^!])/g, "$1")).split("?");
      return c.shift() + (c.length > 0 ? "?" : "") + c.join("&");
    }(typeof arguments[0] == "object" ? arguments[0] : [].slice.call(arguments));
  }(t.reduce((n, o, i) => ((i === 0 || o !== "/" || o === "/" && n[n.length - 1] !== "/") && n.push(o), n), []));
}
var ir = k(542), Nt = k.n(ir);
const ar = "abcdef0123456789";
function de(r, t) {
  const e = r.url.replace("//", ""), n = e.indexOf("/") == -1 ? "/" : e.slice(e.indexOf("/")), o = r.method ? r.method.toUpperCase() : "GET", i = !!/(^|,)\s*auth\s*($|,)/.test(t.qop) && "auth", l = `00000000${t.nc}`.slice(-8), u = function(a, d, p, g, m, N, f) {
    const v = f || Nt()(`${d}:${p}:${g}`);
    return a && a.toLowerCase() === "md5-sess" ? Nt()(`${v}:${m}:${N}`) : v;
  }(t.algorithm, t.username, t.realm, t.password, t.nonce, t.cnonce, t.ha1), y = Nt()(`${o}:${n}`), c = i ? Nt()(`${u}:${t.nonce}:${l}:${t.cnonce}:${i}:${y}`) : Nt()(`${u}:${t.nonce}:${y}`), s = { username: t.username, realm: t.realm, nonce: t.nonce, uri: n, qop: i, response: c, nc: l, cnonce: t.cnonce, algorithm: t.algorithm, opaque: t.opaque }, h = [];
  for (const a in s) s[a] && (a === "qop" || a === "nc" || a === "algorithm" ? h.push(`${a}=${s[a]}`) : h.push(`${a}="${s[a]}"`));
  return `Digest ${h.join(", ")}`;
}
function Ce(r) {
  return (r.headers && r.headers.get("www-authenticate") || "").split(/\s/)[0].toLowerCase() === "digest";
}
var ur = k(101), Ie = k.n(ur);
function me(r) {
  return Ie().decode(r);
}
function ye(r, t) {
  var e;
  return `Basic ${e = `${r}:${t}`, Ie().encode(e)}`;
}
const It = typeof WorkerGlobalScope < "u" && self instanceof WorkerGlobalScope ? self : typeof window < "u" ? window : globalThis, lr = It.fetch.bind(It), cr = It.Request, hr = It.Response;
let Q = function(r) {
  return r.Auto = "auto", r.Digest = "digest", r.None = "none", r.Password = "password", r.Token = "token", r;
}({}), ct = function(r) {
  return r.DataTypeNoLength = "data-type-no-length", r.InvalidAuthType = "invalid-auth-type", r.InvalidOutputFormat = "invalid-output-format", r.LinkUnsupportedAuthType = "link-unsupported-auth", r.InvalidUpdateRange = "invalid-update-range", r.NotSupported = "not-supported", r;
}({});
function ke(r, t, e, n, o) {
  switch (r.authType) {
    case Q.Auto:
      t && e && (r.headers.Authorization = ye(t, e));
      break;
    case Q.Digest:
      r.digest = /* @__PURE__ */ function(l, u, y) {
        return { username: l, password: u, ha1: y, nc: 0, algorithm: "md5", hasDigestAuth: !1 };
      }(t, e, o);
      break;
    case Q.None:
      break;
    case Q.Password:
      r.headers.Authorization = ye(t, e);
      break;
    case Q.Token:
      r.headers.Authorization = `${(i = n).token_type} ${i.access_token}`;
      break;
    default:
      throw new et({ info: { code: ct.InvalidAuthType } }, `Invalid auth type: ${r.authType}`);
  }
  var i;
}
k(345), k(800);
const ve = "@@HOTPATCHER", fr = () => {
};
function Vt(r) {
  return { original: r, methods: [r], final: !1 };
}
class pr {
  constructor() {
    this._configuration = { registry: {}, getEmptyAction: "null" }, this.__type__ = ve;
  }
  get configuration() {
    return this._configuration;
  }
  get getEmptyAction() {
    return this.configuration.getEmptyAction;
  }
  set getEmptyAction(t) {
    this.configuration.getEmptyAction = t;
  }
  control(t) {
    let e = arguments.length > 1 && arguments[1] !== void 0 && arguments[1];
    if (!t || t.__type__ !== ve) throw new Error("Failed taking control of target HotPatcher instance: Invalid type or object");
    return Object.keys(t.configuration.registry).forEach((n) => {
      this.configuration.registry.hasOwnProperty(n) ? e && (this.configuration.registry[n] = Object.assign({}, t.configuration.registry[n])) : this.configuration.registry[n] = Object.assign({}, t.configuration.registry[n]);
    }), t._configuration = this.configuration, this;
  }
  execute(t) {
    const e = this.get(t) || fr;
    for (var n = arguments.length, o = new Array(n > 1 ? n - 1 : 0), i = 1; i < n; i++) o[i - 1] = arguments[i];
    return e(...o);
  }
  get(t) {
    const e = this.configuration.registry[t];
    if (!e) switch (this.getEmptyAction) {
      case "null":
        return null;
      case "throw":
        throw new Error(`Failed handling method request: No method provided for override: ${t}`);
      default:
        throw new Error(`Failed handling request which resulted in an empty method: Invalid empty-action specified: ${this.getEmptyAction}`);
    }
    return function() {
      for (var n = arguments.length, o = new Array(n), i = 0; i < n; i++) o[i] = arguments[i];
      if (o.length === 0) throw new Error("Failed creating sequence: No functions provided");
      return function() {
        for (var l = arguments.length, u = new Array(l), y = 0; y < l; y++) u[y] = arguments[y];
        let c = u;
        const s = this;
        for (; o.length > 0; ) c = [o.shift().apply(s, c)];
        return c[0];
      };
    }(...e.methods);
  }
  isPatched(t) {
    return !!this.configuration.registry[t];
  }
  patch(t, e) {
    let n = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
    const { chain: o = !1 } = n;
    if (this.configuration.registry[t] && this.configuration.registry[t].final) throw new Error(`Failed patching '${t}': Method marked as being final`);
    if (typeof e != "function") throw new Error(`Failed patching '${t}': Provided method is not a function`);
    if (o) this.configuration.registry[t] ? this.configuration.registry[t].methods.push(e) : this.configuration.registry[t] = Vt(e);
    else if (this.isPatched(t)) {
      const { original: i } = this.configuration.registry[t];
      this.configuration.registry[t] = Object.assign(Vt(e), { original: i });
    } else this.configuration.registry[t] = Vt(e);
    return this;
  }
  patchInline(t, e) {
    this.isPatched(t) || this.patch(t, e);
    for (var n = arguments.length, o = new Array(n > 2 ? n - 2 : 0), i = 2; i < n; i++) o[i - 2] = arguments[i];
    return this.execute(t, ...o);
  }
  plugin(t) {
    for (var e = arguments.length, n = new Array(e > 1 ? e - 1 : 0), o = 1; o < e; o++) n[o - 1] = arguments[o];
    return n.forEach((i) => {
      this.patch(t, i, { chain: !0 });
    }), this;
  }
  restore(t) {
    if (!this.isPatched(t)) throw new Error(`Failed restoring method: No method present for key: ${t}`);
    if (typeof this.configuration.registry[t].original != "function") throw new Error(`Failed restoring method: Original method not found or of invalid type for key: ${t}`);
    return this.configuration.registry[t].methods = [this.configuration.registry[t].original], this;
  }
  setFinal(t) {
    if (!this.configuration.registry.hasOwnProperty(t)) throw new Error(`Failed marking '${t}' as final: No method found for key`);
    return this.configuration.registry[t].final = !0, this;
  }
}
let Wt = null;
function Re() {
  return Wt || (Wt = new pr()), Wt;
}
function kt(r) {
  return function(t) {
    if (typeof t != "object" || t === null || Object.prototype.toString.call(t) != "[object Object]") return !1;
    if (Object.getPrototypeOf(t) === null) return !0;
    let e = t;
    for (; Object.getPrototypeOf(e) !== null; ) e = Object.getPrototypeOf(e);
    return Object.getPrototypeOf(t) === e;
  }(r) ? Object.assign({}, r) : Object.setPrototypeOf(Object.assign({}, r), Object.getPrototypeOf(r));
}
function be() {
  for (var r = arguments.length, t = new Array(r), e = 0; e < r; e++) t[e] = arguments[e];
  let n = null, o = [...t];
  for (; o.length > 0; ) {
    const i = o.shift();
    n = n ? Le(n, i) : kt(i);
  }
  return n;
}
function Le(r, t) {
  const e = kt(r);
  return Object.keys(t).forEach((n) => {
    e.hasOwnProperty(n) ? Array.isArray(t[n]) ? e[n] = Array.isArray(e[n]) ? [...e[n], ...t[n]] : [...t[n]] : typeof t[n] == "object" && t[n] ? e[n] = typeof e[n] == "object" && e[n] ? Le(e[n], t[n]) : kt(t[n]) : e[n] = t[n] : e[n] = t[n];
  }), e;
}
function gr(r) {
  const t = {};
  for (const e of r.keys()) t[e] = r.get(e);
  return t;
}
function Yt() {
  for (var r = arguments.length, t = new Array(r), e = 0; e < r; e++) t[e] = arguments[e];
  if (t.length === 0) return {};
  const n = {};
  return t.reduce((o, i) => (Object.keys(i).forEach((l) => {
    const u = l.toLowerCase();
    n.hasOwnProperty(u) ? o[n[u]] = i[l] : (n[u] = l, o[l] = i[l]);
  }), o), {});
}
k(805);
const dr = typeof ArrayBuffer == "function", { toString: mr } = Object.prototype;
function _e(r) {
  return dr && (r instanceof ArrayBuffer || mr.call(r) === "[object ArrayBuffer]");
}
function Ue(r) {
  return r != null && r.constructor != null && typeof r.constructor.isBuffer == "function" && r.constructor.isBuffer(r);
}
function ee(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}
function Kt(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const Me = ee(function(r) {
  const t = r._digest;
  return delete r._digest, t.hasDigestAuth && (r = be(r, { headers: { Authorization: de(r, t) } })), Kt(Rt(r), function(e) {
    let n = !1;
    return o = function(l) {
      return n ? l : e;
    }, (i = function() {
      if (e.status == 401) return t.hasDigestAuth = function(l, u) {
        if (!Ce(l)) return !1;
        const y = /([a-z0-9_-]+)=(?:"([^"]+)"|([a-z0-9_-]+))/gi;
        for (; ; ) {
          const c = l.headers && l.headers.get("www-authenticate") || "", s = y.exec(c);
          if (!s) break;
          u[s[1]] = s[2] || s[3];
        }
        return u.nc += 1, u.cnonce = function() {
          let c = "";
          for (let s = 0; s < 32; ++s) c = `${c}${ar[Math.floor(16 * Math.random())]}`;
          return c;
        }(), !0;
      }(e, t), function() {
        if (t.hasDigestAuth) return Kt(Rt(r = be(r, { headers: { Authorization: de(r, t) } })), function(l) {
          return l.status == 401 ? t.hasDigestAuth = !1 : t.nc++, n = !0, l;
        });
      }();
      t.nc++;
    }()) && i.then ? i.then(o) : o(i);
    var o, i;
  });
}), yr = ee(function(r, t) {
  return Kt(Rt(r), function(e) {
    return e.ok ? (t.authType = Q.Password, e) : e.status == 401 && Ce(e) ? (t.authType = Q.Digest, ke(t, t.username, t.password, void 0, void 0), r._digest = t.digest, Me(r)) : e;
  });
}), G = ee(function(r, t) {
  return t.authType === Q.Auto ? yr(r, t) : r._digest ? Me(r) : Rt(r);
});
function q(r, t, e) {
  const n = kt(r);
  return n.headers = Yt(t.headers, n.headers || {}, e.headers || {}), e.data !== void 0 && (n.data = e.data), e.signal && (n.signal = e.signal), t.httpAgent && (n.httpAgent = t.httpAgent), t.httpsAgent && (n.httpsAgent = t.httpsAgent), t.digest && (n._digest = t.digest), typeof t.withCredentials == "boolean" && (n.withCredentials = t.withCredentials), n;
}
function Rt(r) {
  const t = Re();
  return t.patchInline("request", (e) => t.patchInline("fetch", lr, e.url, function(n) {
    let o = {};
    const i = { method: n.method };
    if (n.headers && (o = Yt(o, n.headers)), n.data !== void 0) {
      const [l, u] = function(y) {
        if (typeof y == "string") return [y, {}];
        if (Ue(y)) return [y, {}];
        if (_e(y)) return [y, {}];
        if (y && typeof y == "object") return [JSON.stringify(y), { "content-type": "application/json" }];
        throw new Error("Unable to convert request body: Unexpected body type: " + typeof y);
      }(n.data);
      i.body = l, o = Yt(o, u);
    }
    return n.signal && (i.signal = n.signal), n.withCredentials && (i.credentials = "include"), i.headers = o, i;
  }(e)), r);
}
var vr = k(285);
const Lt = (r) => {
  if (typeof r != "string") throw new TypeError("invalid pattern");
  if (r.length > 65536) throw new TypeError("pattern is too long");
}, br = { "[:alnum:]": ["\\p{L}\\p{Nl}\\p{Nd}", !0], "[:alpha:]": ["\\p{L}\\p{Nl}", !0], "[:ascii:]": ["\\x00-\\x7f", !1], "[:blank:]": ["\\p{Zs}\\t", !0], "[:cntrl:]": ["\\p{Cc}", !0], "[:digit:]": ["\\p{Nd}", !0], "[:graph:]": ["\\p{Z}\\p{C}", !0, !0], "[:lower:]": ["\\p{Ll}", !0], "[:print:]": ["\\p{C}", !0], "[:punct:]": ["\\p{P}", !0], "[:space:]": ["\\p{Z}\\t\\r\\n\\v\\f", !0], "[:upper:]": ["\\p{Lu}", !0], "[:word:]": ["\\p{L}\\p{Nl}\\p{Nd}\\p{Pc}", !0], "[:xdigit:]": ["A-Fa-f0-9", !1] }, At = (r) => r.replace(/[[\]\\-]/g, "\\$&"), we = (r) => r.join(""), wr = (r, t) => {
  const e = t;
  if (r.charAt(e) !== "[") throw new Error("not in a brace expression");
  const n = [], o = [];
  let i = e + 1, l = !1, u = !1, y = !1, c = !1, s = e, h = "";
  t: for (; i < r.length; ) {
    const g = r.charAt(i);
    if (g !== "!" && g !== "^" || i !== e + 1) {
      if (g === "]" && l && !y) {
        s = i + 1;
        break;
      }
      if (l = !0, g !== "\\" || y) {
        if (g === "[" && !y) {
          for (const [m, [N, f, v]] of Object.entries(br)) if (r.startsWith(m, i)) {
            if (h) return ["$.", !1, r.length - e, !0];
            i += m.length, v ? o.push(N) : n.push(N), u = u || f;
            continue t;
          }
        }
        y = !1, h ? (g > h ? n.push(At(h) + "-" + At(g)) : g === h && n.push(At(g)), h = "", i++) : r.startsWith("-]", i + 1) ? (n.push(At(g + "-")), i += 2) : r.startsWith("-", i + 1) ? (h = g, i += 2) : (n.push(At(g)), i++);
      } else y = !0, i++;
    } else c = !0, i++;
  }
  if (s < i) return ["", !1, 0, !1];
  if (!n.length && !o.length) return ["$.", !1, r.length - e, !0];
  if (o.length === 0 && n.length === 1 && /^\\?.$/.test(n[0]) && !c)
    return [(a = n[0].length === 2 ? n[0].slice(-1) : n[0], a.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&")), !1, s - e, !1];
  var a;
  const d = "[" + (c ? "^" : "") + we(n) + "]", p = "[" + (c ? "" : "^") + we(o) + "]";
  return [n.length && o.length ? "(" + d + "|" + p + ")" : n.length ? d : p, u, s - e, !0];
}, Pt = function(r) {
  let { windowsPathsNoEscape: t = !1 } = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  return t ? r.replace(/\[([^\/\\])\]/g, "$1") : r.replace(/((?!\\).|^)\[([^\/\\])\]/g, "$1$2").replace(/\\([^\/])/g, "$1");
}, xr = /* @__PURE__ */ new Set(["!", "?", "+", "*", "@"]), xe = (r) => xr.has(r), zt = "(?!\\.)", Nr = /* @__PURE__ */ new Set(["[", "."]), Ar = /* @__PURE__ */ new Set(["..", "."]), Pr = new Set("().*{}+?[]^$\\!"), re = "[^/]", Ne = re + "*?", Ae = re + "+?";
var z, X, lt, L, B, ft, mt, pt, at, yt, Tt, vt, Fe, gt, $t, Jt, De;
const J = class J {
  constructor(t, e) {
    rt(this, vt);
    W(this, "type");
    rt(this, z);
    rt(this, X);
    rt(this, lt, !1);
    rt(this, L, []);
    rt(this, B);
    rt(this, ft);
    rt(this, mt);
    rt(this, pt, !1);
    rt(this, at);
    rt(this, yt);
    rt(this, Tt, !1);
    let n = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
    this.type = t, t && F(this, X, !0), F(this, B, e), F(this, z, T(this, B) ? T(T(this, B), z) : this), F(this, at, T(this, z) === this ? n : T(T(this, z), at)), F(this, mt, T(this, z) === this ? [] : T(T(this, z), mt)), t !== "!" || T(T(this, z), pt) || T(this, mt).push(this), F(this, ft, T(this, B) ? T(T(this, B), L).length : 0);
  }
  get hasMagic() {
    if (T(this, X) !== void 0) return T(this, X);
    for (const t of T(this, L)) if (typeof t != "string" && (t.type || t.hasMagic)) return F(this, X, !0);
    return T(this, X);
  }
  toString() {
    return T(this, yt) !== void 0 ? T(this, yt) : this.type ? F(this, yt, this.type + "(" + T(this, L).map((t) => String(t)).join("|") + ")") : F(this, yt, T(this, L).map((t) => String(t)).join(""));
  }
  push() {
    for (var t = arguments.length, e = new Array(t), n = 0; n < t; n++) e[n] = arguments[n];
    for (const o of e) if (o !== "") {
      if (typeof o != "string" && !(o instanceof J && T(o, B) === this)) throw new Error("invalid part: " + o);
      T(this, L).push(o);
    }
  }
  toJSON() {
    var e;
    const t = this.type === null ? T(this, L).slice().map((n) => typeof n == "string" ? n : n.toJSON()) : [this.type, ...T(this, L).map((n) => n.toJSON())];
    return this.isStart() && !this.type && t.unshift([]), this.isEnd() && (this === T(this, z) || T(T(this, z), pt) && ((e = T(this, B)) == null ? void 0 : e.type) === "!") && t.push({}), t;
  }
  isStart() {
    var e;
    if (T(this, z) === this) return !0;
    if (!((e = T(this, B)) != null && e.isStart())) return !1;
    if (T(this, ft) === 0) return !0;
    const t = T(this, B);
    for (let n = 0; n < T(this, ft); n++) {
      const o = T(t, L)[n];
      if (!(o instanceof J && o.type === "!")) return !1;
    }
    return !0;
  }
  isEnd() {
    var e, n, o;
    if (T(this, z) === this || ((e = T(this, B)) == null ? void 0 : e.type) === "!") return !0;
    if (!((n = T(this, B)) != null && n.isEnd())) return !1;
    if (!this.type) return (o = T(this, B)) == null ? void 0 : o.isEnd();
    const t = T(this, B) ? T(T(this, B), L).length : 0;
    return T(this, ft) === t - 1;
  }
  copyIn(t) {
    typeof t == "string" ? this.push(t) : this.push(t.clone(this));
  }
  clone(t) {
    const e = new J(this.type, t);
    for (const n of T(this, L)) e.copyIn(n);
    return e;
  }
  static fromGlob(t) {
    var o;
    let e = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    const n = new J(null, void 0, e);
    return ht(o = J, gt, $t).call(o, t, n, 0, e), n;
  }
  toMMPattern() {
    if (this !== T(this, z)) return T(this, z).toMMPattern();
    const t = this.toString(), [e, n, o, i] = this.toRegExpSource();
    if (!(o || T(this, X) || T(this, at).nocase && !T(this, at).nocaseMagicOnly && t.toUpperCase() !== t.toLowerCase())) return n;
    const l = (T(this, at).nocase ? "i" : "") + (i ? "u" : "");
    return Object.assign(new RegExp(`^${e}$`, l), { _src: e, _glob: t });
  }
  get options() {
    return T(this, at);
  }
  toRegExpSource(t) {
    var y;
    const e = t ?? !!T(this, at).dot;
    if (T(this, z) === this && ht(this, vt, Fe).call(this), !this.type) {
      const c = this.isStart() && this.isEnd(), s = T(this, L).map((d) => {
        var f;
        const [p, g, m, N] = typeof d == "string" ? ht(f = J, gt, De).call(f, d, T(this, X), c) : d.toRegExpSource(t);
        return F(this, X, T(this, X) || m), F(this, lt, T(this, lt) || N), p;
      }).join("");
      let h = "";
      if (this.isStart() && typeof T(this, L)[0] == "string" && (T(this, L).length !== 1 || !Ar.has(T(this, L)[0]))) {
        const d = Nr, p = e && d.has(s.charAt(0)) || s.startsWith("\\.") && d.has(s.charAt(2)) || s.startsWith("\\.\\.") && d.has(s.charAt(4)), g = !e && !t && d.has(s.charAt(0));
        h = p ? "(?!(?:^|/)\\.\\.?(?:$|/))" : g ? zt : "";
      }
      let a = "";
      return this.isEnd() && T(T(this, z), pt) && ((y = T(this, B)) == null ? void 0 : y.type) === "!" && (a = "(?:$|\\/)"), [h + s + a, Pt(s), F(this, X, !!T(this, X)), T(this, lt)];
    }
    const n = this.type === "*" || this.type === "+", o = this.type === "!" ? "(?:(?!(?:" : "(?:";
    let i = ht(this, vt, Jt).call(this, e);
    if (this.isStart() && this.isEnd() && !i && this.type !== "!") {
      const c = this.toString();
      return F(this, L, [c]), this.type = null, F(this, X, void 0), [c, Pt(this.toString()), !1, !1];
    }
    let l = !n || t || e ? "" : ht(this, vt, Jt).call(this, !0);
    l === i && (l = ""), l && (i = `(?:${i})(?:${l})*?`);
    let u = "";
    return u = this.type === "!" && T(this, Tt) ? (this.isStart() && !e ? zt : "") + Ae : o + i + (this.type === "!" ? "))" + (!this.isStart() || e || t ? "" : zt) + Ne + ")" : this.type === "@" ? ")" : this.type === "?" ? ")?" : this.type === "+" && l ? ")" : this.type === "*" && l ? ")?" : `)${this.type}`), [u, Pt(i), F(this, X, !!T(this, X)), T(this, lt)];
  }
};
z = new WeakMap(), X = new WeakMap(), lt = new WeakMap(), L = new WeakMap(), B = new WeakMap(), ft = new WeakMap(), mt = new WeakMap(), pt = new WeakMap(), at = new WeakMap(), yt = new WeakMap(), Tt = new WeakMap(), vt = new WeakSet(), Fe = function() {
  if (this !== T(this, z)) throw new Error("should only call on root");
  if (T(this, pt)) return this;
  let t;
  for (this.toString(), F(this, pt, !0); t = T(this, mt).pop(); ) {
    if (t.type !== "!") continue;
    let e = t, n = T(e, B);
    for (; n; ) {
      for (let o = T(e, ft) + 1; !n.type && o < T(n, L).length; o++) for (const i of T(t, L)) {
        if (typeof i == "string") throw new Error("string part in extglob AST??");
        i.copyIn(T(n, L)[o]);
      }
      e = n, n = T(e, B);
    }
  }
  return this;
}, gt = new WeakSet(), $t = function(t, e, n, o) {
  var d, p;
  let i = !1, l = !1, u = -1, y = !1;
  if (e.type === null) {
    let g = n, m = "";
    for (; g < t.length; ) {
      const N = t.charAt(g++);
      if (i || N === "\\") i = !i, m += N;
      else if (l) g === u + 1 ? N !== "^" && N !== "!" || (y = !0) : N !== "]" || g === u + 2 && y || (l = !1), m += N;
      else if (N !== "[") if (o.noext || !xe(N) || t.charAt(g) !== "(") m += N;
      else {
        e.push(m), m = "";
        const f = new J(N, e);
        g = ht(d = J, gt, $t).call(d, t, f, g, o), e.push(f);
      }
      else l = !0, u = g, y = !1, m += N;
    }
    return e.push(m), g;
  }
  let c = n + 1, s = new J(null, e);
  const h = [];
  let a = "";
  for (; c < t.length; ) {
    const g = t.charAt(c++);
    if (i || g === "\\") i = !i, a += g;
    else if (l) c === u + 1 ? g !== "^" && g !== "!" || (y = !0) : g !== "]" || c === u + 2 && y || (l = !1), a += g;
    else if (g !== "[") if (xe(g) && t.charAt(c) === "(") {
      s.push(a), a = "";
      const m = new J(g, s);
      s.push(m), c = ht(p = J, gt, $t).call(p, t, m, c, o);
    } else if (g !== "|") {
      if (g === ")") return a === "" && T(e, L).length === 0 && F(e, Tt, !0), s.push(a), a = "", e.push(...h, s), c;
      a += g;
    } else s.push(a), a = "", h.push(s), s = new J(null, e);
    else l = !0, u = c, y = !1, a += g;
  }
  return e.type = null, F(e, X, void 0), F(e, L, [t.substring(n - 1)]), c;
}, Jt = function(t) {
  return T(this, L).map((e) => {
    if (typeof e == "string") throw new Error("string type in extglob ast??");
    const [n, o, i, l] = e.toRegExpSource(t);
    return F(this, lt, T(this, lt) || l), n;
  }).filter((e) => !(this.isStart() && this.isEnd() && !e)).join("|");
}, De = function(t, e) {
  let n = arguments.length > 2 && arguments[2] !== void 0 && arguments[2], o = !1, i = "", l = !1;
  for (let u = 0; u < t.length; u++) {
    const y = t.charAt(u);
    if (o) o = !1, i += (Pr.has(y) ? "\\" : "") + y;
    else if (y !== "\\") {
      if (y === "[") {
        const [c, s, h, a] = wr(t, u);
        if (h) {
          i += c, l = l || s, u += h - 1, e = e || a;
          continue;
        }
      }
      y !== "*" ? y !== "?" ? i += y.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&") : (i += re, e = !0) : (i += n && t === "*" ? Ae : Ne, e = !0);
    } else u === t.length - 1 ? i += "\\\\" : o = !0;
  }
  return [i, Pt(t), !!e, l];
}, rt(J, gt);
let _t = J;
const K = function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  return Lt(t), !(!e.nocomment && t.charAt(0) === "#") && new Ut(t, e).match(r);
}, Or = /^\*+([^+@!?\*\[\(]*)$/, Er = (r) => (t) => !t.startsWith(".") && t.endsWith(r), Tr = (r) => (t) => t.endsWith(r), jr = (r) => (r = r.toLowerCase(), (t) => !t.startsWith(".") && t.toLowerCase().endsWith(r)), Sr = (r) => (r = r.toLowerCase(), (t) => t.toLowerCase().endsWith(r)), $r = /^\*+\.\*+$/, Cr = (r) => !r.startsWith(".") && r.includes("."), Ir = (r) => r !== "." && r !== ".." && r.includes("."), kr = /^\.\*+$/, Rr = (r) => r !== "." && r !== ".." && r.startsWith("."), Lr = /^\*+$/, _r = (r) => r.length !== 0 && !r.startsWith("."), Ur = (r) => r.length !== 0 && r !== "." && r !== "..", Mr = /^\?+([^+@!?\*\[\(]*)?$/, Fr = (r) => {
  let [t, e = ""] = r;
  const n = Be([t]);
  return e ? (e = e.toLowerCase(), (o) => n(o) && o.toLowerCase().endsWith(e)) : n;
}, Dr = (r) => {
  let [t, e = ""] = r;
  const n = Ve([t]);
  return e ? (e = e.toLowerCase(), (o) => n(o) && o.toLowerCase().endsWith(e)) : n;
}, Br = (r) => {
  let [t, e = ""] = r;
  const n = Ve([t]);
  return e ? (o) => n(o) && o.endsWith(e) : n;
}, Vr = (r) => {
  let [t, e = ""] = r;
  const n = Be([t]);
  return e ? (o) => n(o) && o.endsWith(e) : n;
}, Be = (r) => {
  let [t] = r;
  const e = t.length;
  return (n) => n.length === e && !n.startsWith(".");
}, Ve = (r) => {
  let [t] = r;
  const e = t.length;
  return (n) => n.length === e && n !== "." && n !== "..";
}, We = typeof process == "object" && process ? typeof process.env == "object" && process.env && process.env.__MINIMATCH_TESTING_PLATFORM__ || process.platform : "posix";
K.sep = We === "win32" ? "\\" : "/";
const ot = Symbol("globstar **");
K.GLOBSTAR = ot, K.filter = function(r) {
  let t = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  return (e) => K(e, r, t);
};
const st = function(r) {
  let t = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  return Object.assign({}, r, t);
};
K.defaults = (r) => {
  if (!r || typeof r != "object" || !Object.keys(r).length) return K;
  const t = K;
  return Object.assign(function(e, n) {
    return t(e, n, st(r, arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {}));
  }, { Minimatch: class extends t.Minimatch {
    constructor(e) {
      super(e, st(r, arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {}));
    }
    static defaults(e) {
      return t.defaults(st(r, e)).Minimatch;
    }
  }, AST: class extends t.AST {
    constructor(e, n) {
      super(e, n, st(r, arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {}));
    }
    static fromGlob(e) {
      let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
      return t.AST.fromGlob(e, st(r, n));
    }
  }, unescape: function(e) {
    let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    return t.unescape(e, st(r, n));
  }, escape: function(e) {
    let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    return t.escape(e, st(r, n));
  }, filter: function(e) {
    let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    return t.filter(e, st(r, n));
  }, defaults: (e) => t.defaults(st(r, e)), makeRe: function(e) {
    let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    return t.makeRe(e, st(r, n));
  }, braceExpand: function(e) {
    let n = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    return t.braceExpand(e, st(r, n));
  }, match: function(e, n) {
    let o = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
    return t.match(e, n, st(r, o));
  }, sep: t.sep, GLOBSTAR: ot });
};
const ze = function(r) {
  let t = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  return Lt(r), t.nobrace || !/\{(?:(?!\{).)*\}/.test(r) ? [r] : vr(r);
};
K.braceExpand = ze, K.makeRe = function(r) {
  return new Ut(r, arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {}).makeRe();
}, K.match = function(r, t) {
  const e = new Ut(t, arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {});
  return r = r.filter((n) => e.match(n)), e.options.nonull && !r.length && r.push(t), r;
};
const Pe = /[?*]|[+@!]\(.*?\)|\[|\]/;
class Ut {
  constructor(t) {
    W(this, "options");
    W(this, "set");
    W(this, "pattern");
    W(this, "windowsPathsNoEscape");
    W(this, "nonegate");
    W(this, "negate");
    W(this, "comment");
    W(this, "empty");
    W(this, "preserveMultipleSlashes");
    W(this, "partial");
    W(this, "globSet");
    W(this, "globParts");
    W(this, "nocase");
    W(this, "isWindows");
    W(this, "platform");
    W(this, "windowsNoMagicRoot");
    W(this, "regexp");
    let e = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
    Lt(t), e = e || {}, this.options = e, this.pattern = t, this.platform = e.platform || We, this.isWindows = this.platform === "win32", this.windowsPathsNoEscape = !!e.windowsPathsNoEscape || e.allowWindowsEscape === !1, this.windowsPathsNoEscape && (this.pattern = this.pattern.replace(/\\/g, "/")), this.preserveMultipleSlashes = !!e.preserveMultipleSlashes, this.regexp = null, this.negate = !1, this.nonegate = !!e.nonegate, this.comment = !1, this.empty = !1, this.partial = !!e.partial, this.nocase = !!this.options.nocase, this.windowsNoMagicRoot = e.windowsNoMagicRoot !== void 0 ? e.windowsNoMagicRoot : !(!this.isWindows || !this.nocase), this.globSet = [], this.globParts = [], this.set = [], this.make();
  }
  hasMagic() {
    if (this.options.magicalBraces && this.set.length > 1) return !0;
    for (const t of this.set) for (const e of t) if (typeof e != "string") return !0;
    return !1;
  }
  debug() {
  }
  make() {
    const t = this.pattern, e = this.options;
    if (!e.nocomment && t.charAt(0) === "#") return void (this.comment = !0);
    if (!t) return void (this.empty = !0);
    this.parseNegate(), this.globSet = [...new Set(this.braceExpand())], e.debug && (this.debug = function() {
      return console.error(...arguments);
    }), this.debug(this.pattern, this.globSet);
    const n = this.globSet.map((i) => this.slashSplit(i));
    this.globParts = this.preprocess(n), this.debug(this.pattern, this.globParts);
    let o = this.globParts.map((i, l, u) => {
      if (this.isWindows && this.windowsNoMagicRoot) {
        const y = !(i[0] !== "" || i[1] !== "" || i[2] !== "?" && Pe.test(i[2]) || Pe.test(i[3])), c = /^[a-z]:/i.test(i[0]);
        if (y) return [...i.slice(0, 4), ...i.slice(4).map((s) => this.parse(s))];
        if (c) return [i[0], ...i.slice(1).map((s) => this.parse(s))];
      }
      return i.map((y) => this.parse(y));
    });
    if (this.debug(this.pattern, o), this.set = o.filter((i) => i.indexOf(!1) === -1), this.isWindows) for (let i = 0; i < this.set.length; i++) {
      const l = this.set[i];
      l[0] === "" && l[1] === "" && this.globParts[i][2] === "?" && typeof l[3] == "string" && /^[a-z]:$/i.test(l[3]) && (l[2] = "?");
    }
    this.debug(this.pattern, this.set);
  }
  preprocess(t) {
    if (this.options.noglobstar) for (let n = 0; n < t.length; n++) for (let o = 0; o < t[n].length; o++) t[n][o] === "**" && (t[n][o] = "*");
    const { optimizationLevel: e = 1 } = this.options;
    return e >= 2 ? (t = this.firstPhasePreProcess(t), t = this.secondPhasePreProcess(t)) : t = e >= 1 ? this.levelOneOptimize(t) : this.adjascentGlobstarOptimize(t), t;
  }
  adjascentGlobstarOptimize(t) {
    return t.map((e) => {
      let n = -1;
      for (; (n = e.indexOf("**", n + 1)) !== -1; ) {
        let o = n;
        for (; e[o + 1] === "**"; ) o++;
        o !== n && e.splice(n, o - n);
      }
      return e;
    });
  }
  levelOneOptimize(t) {
    return t.map((e) => (e = e.reduce((n, o) => {
      const i = n[n.length - 1];
      return o === "**" && i === "**" ? n : o === ".." && i && i !== ".." && i !== "." && i !== "**" ? (n.pop(), n) : (n.push(o), n);
    }, [])).length === 0 ? [""] : e);
  }
  levelTwoFileOptimize(t) {
    Array.isArray(t) || (t = this.slashSplit(t));
    let e = !1;
    do {
      if (e = !1, !this.preserveMultipleSlashes) {
        for (let o = 1; o < t.length - 1; o++) {
          const i = t[o];
          o === 1 && i === "" && t[0] === "" || i !== "." && i !== "" || (e = !0, t.splice(o, 1), o--);
        }
        t[0] !== "." || t.length !== 2 || t[1] !== "." && t[1] !== "" || (e = !0, t.pop());
      }
      let n = 0;
      for (; (n = t.indexOf("..", n + 1)) !== -1; ) {
        const o = t[n - 1];
        o && o !== "." && o !== ".." && o !== "**" && (e = !0, t.splice(n - 1, 2), n -= 2);
      }
    } while (e);
    return t.length === 0 ? [""] : t;
  }
  firstPhasePreProcess(t) {
    let e = !1;
    do {
      e = !1;
      for (let n of t) {
        let o = -1;
        for (; (o = n.indexOf("**", o + 1)) !== -1; ) {
          let l = o;
          for (; n[l + 1] === "**"; ) l++;
          l > o && n.splice(o + 1, l - o);
          let u = n[o + 1];
          const y = n[o + 2], c = n[o + 3];
          if (u !== ".." || !y || y === "." || y === ".." || !c || c === "." || c === "..") continue;
          e = !0, n.splice(o, 1);
          const s = n.slice(0);
          s[o] = "**", t.push(s), o--;
        }
        if (!this.preserveMultipleSlashes) {
          for (let l = 1; l < n.length - 1; l++) {
            const u = n[l];
            l === 1 && u === "" && n[0] === "" || u !== "." && u !== "" || (e = !0, n.splice(l, 1), l--);
          }
          n[0] !== "." || n.length !== 2 || n[1] !== "." && n[1] !== "" || (e = !0, n.pop());
        }
        let i = 0;
        for (; (i = n.indexOf("..", i + 1)) !== -1; ) {
          const l = n[i - 1];
          if (l && l !== "." && l !== ".." && l !== "**") {
            e = !0;
            const u = i === 1 && n[i + 1] === "**" ? ["."] : [];
            n.splice(i - 1, 2, ...u), n.length === 0 && n.push(""), i -= 2;
          }
        }
      }
    } while (e);
    return t;
  }
  secondPhasePreProcess(t) {
    for (let e = 0; e < t.length - 1; e++) for (let n = e + 1; n < t.length; n++) {
      const o = this.partsMatch(t[e], t[n], !this.preserveMultipleSlashes);
      if (o) {
        t[e] = [], t[n] = o;
        break;
      }
    }
    return t.filter((e) => e.length);
  }
  partsMatch(t, e) {
    let n = arguments.length > 2 && arguments[2] !== void 0 && arguments[2], o = 0, i = 0, l = [], u = "";
    for (; o < t.length && i < e.length; ) if (t[o] === e[i]) l.push(u === "b" ? e[i] : t[o]), o++, i++;
    else if (n && t[o] === "**" && e[i] === t[o + 1]) l.push(t[o]), o++;
    else if (n && e[i] === "**" && t[o] === e[i + 1]) l.push(e[i]), i++;
    else if (t[o] !== "*" || !e[i] || !this.options.dot && e[i].startsWith(".") || e[i] === "**") {
      if (e[i] !== "*" || !t[o] || !this.options.dot && t[o].startsWith(".") || t[o] === "**" || u === "a") return !1;
      u = "b", l.push(e[i]), o++, i++;
    } else {
      if (u === "b") return !1;
      u = "a", l.push(t[o]), o++, i++;
    }
    return t.length === e.length && l;
  }
  parseNegate() {
    if (this.nonegate) return;
    const t = this.pattern;
    let e = !1, n = 0;
    for (let o = 0; o < t.length && t.charAt(o) === "!"; o++) e = !e, n++;
    n && (this.pattern = t.slice(n)), this.negate = e;
  }
  matchOne(t, e) {
    let n = arguments.length > 2 && arguments[2] !== void 0 && arguments[2];
    const o = this.options;
    if (this.isWindows) {
      const g = typeof t[0] == "string" && /^[a-z]:$/i.test(t[0]), m = !g && t[0] === "" && t[1] === "" && t[2] === "?" && /^[a-z]:$/i.test(t[3]), N = typeof e[0] == "string" && /^[a-z]:$/i.test(e[0]), f = m ? 3 : g ? 0 : void 0, v = !N && e[0] === "" && e[1] === "" && e[2] === "?" && typeof e[3] == "string" && /^[a-z]:$/i.test(e[3]) ? 3 : N ? 0 : void 0;
      if (typeof f == "number" && typeof v == "number") {
        const [x, A] = [t[f], e[v]];
        x.toLowerCase() === A.toLowerCase() && (e[v] = x, v > f ? e = e.slice(v) : f > v && (t = t.slice(f)));
      }
    }
    const { optimizationLevel: i = 1 } = this.options;
    i >= 2 && (t = this.levelTwoFileOptimize(t)), this.debug("matchOne", this, { file: t, pattern: e }), this.debug("matchOne", t.length, e.length);
    for (var l = 0, u = 0, y = t.length, c = e.length; l < y && u < c; l++, u++) {
      this.debug("matchOne loop");
      var s = e[u], h = t[l];
      if (this.debug(e, s, h), s === !1) return !1;
      if (s === ot) {
        this.debug("GLOBSTAR", [e, s, h]);
        var a = l, d = u + 1;
        if (d === c) {
          for (this.debug("** at the end"); l < y; l++) if (t[l] === "." || t[l] === ".." || !o.dot && t[l].charAt(0) === ".") return !1;
          return !0;
        }
        for (; a < y; ) {
          var p = t[a];
          if (this.debug(`
globstar while`, t, a, e, d, p), this.matchOne(t.slice(a), e.slice(d), n)) return this.debug("globstar found match!", a, y, p), !0;
          if (p === "." || p === ".." || !o.dot && p.charAt(0) === ".") {
            this.debug("dot detected!", t, a, e, d);
            break;
          }
          this.debug("globstar swallow a segment, and continue"), a++;
        }
        return !(!n || (this.debug(`
>>> no match, partial?`, t, a, e, d), a !== y));
      }
      let g;
      if (typeof s == "string" ? (g = h === s, this.debug("string match", s, h, g)) : (g = s.test(h), this.debug("pattern match", s, h, g)), !g) return !1;
    }
    if (l === y && u === c) return !0;
    if (l === y) return n;
    if (u === c) return l === y - 1 && t[l] === "";
    throw new Error("wtf?");
  }
  braceExpand() {
    return ze(this.pattern, this.options);
  }
  parse(t) {
    Lt(t);
    const e = this.options;
    if (t === "**") return ot;
    if (t === "") return "";
    let n, o = null;
    (n = t.match(Lr)) ? o = e.dot ? Ur : _r : (n = t.match(Or)) ? o = (e.nocase ? e.dot ? Sr : jr : e.dot ? Tr : Er)(n[1]) : (n = t.match(Mr)) ? o = (e.nocase ? e.dot ? Dr : Fr : e.dot ? Br : Vr)(n) : (n = t.match($r)) ? o = e.dot ? Ir : Cr : (n = t.match(kr)) && (o = Rr);
    const i = _t.fromGlob(t, this.options).toMMPattern();
    return o && typeof i == "object" && Reflect.defineProperty(i, "test", { value: o }), i;
  }
  makeRe() {
    if (this.regexp || this.regexp === !1) return this.regexp;
    const t = this.set;
    if (!t.length) return this.regexp = !1, this.regexp;
    const e = this.options, n = e.noglobstar ? "[^/]*?" : e.dot ? "(?:(?!(?:\\/|^)(?:\\.{1,2})($|\\/)).)*?" : "(?:(?!(?:\\/|^)\\.).)*?", o = new Set(e.nocase ? ["i"] : []);
    let i = t.map((y) => {
      const c = y.map((s) => {
        if (s instanceof RegExp) for (const h of s.flags.split("")) o.add(h);
        return typeof s == "string" ? s.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&") : s === ot ? ot : s._src;
      });
      return c.forEach((s, h) => {
        const a = c[h + 1], d = c[h - 1];
        s === ot && d !== ot && (d === void 0 ? a !== void 0 && a !== ot ? c[h + 1] = "(?:\\/|" + n + "\\/)?" + a : c[h] = n : a === void 0 ? c[h - 1] = d + "(?:\\/|" + n + ")?" : a !== ot && (c[h - 1] = d + "(?:\\/|\\/" + n + "\\/)" + a, c[h + 1] = ot));
      }), c.filter((s) => s !== ot).join("/");
    }).join("|");
    const [l, u] = t.length > 1 ? ["(?:", ")"] : ["", ""];
    i = "^" + l + i + u + "$", this.negate && (i = "^(?!" + i + ").+$");
    try {
      this.regexp = new RegExp(i, [...o].join(""));
    } catch {
      this.regexp = !1;
    }
    return this.regexp;
  }
  slashSplit(t) {
    return this.preserveMultipleSlashes ? t.split("/") : this.isWindows && /^\/\/[^\/]+/.test(t) ? ["", ...t.split(/\/+/)] : t.split(/\/+/);
  }
  match(t) {
    let e = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : this.partial;
    if (this.debug("match", t, this.pattern), this.comment) return !1;
    if (this.empty) return t === "";
    if (t === "/" && e) return !0;
    const n = this.options;
    this.isWindows && (t = t.split("\\").join("/"));
    const o = this.slashSplit(t);
    this.debug(this.pattern, "split", o);
    const i = this.set;
    this.debug(this.pattern, "set", i);
    let l = o[o.length - 1];
    if (!l) for (let u = o.length - 2; !l && u >= 0; u--) l = o[u];
    for (let u = 0; u < i.length; u++) {
      const y = i[u];
      let c = o;
      if (n.matchBase && y.length === 1 && (c = [l]), this.matchOne(c, y, e)) return !!n.flipNegate || !this.negate;
    }
    return !n.flipNegate && this.negate;
  }
  static defaults(t) {
    return K.defaults(t).Minimatch;
  }
}
function ne(r) {
  const t = new Error(`${arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : ""}Invalid response: ${r.status} ${r.statusText}`);
  return t.status = r.status, t.response = r, t;
}
function H(r, t) {
  const { status: e } = t;
  if (e === 401 && r.digest) return t;
  if (e >= 400) throw ne(t);
  return t;
}
function bt(r, t) {
  return arguments.length > 2 && arguments[2] !== void 0 && arguments[2] ? { data: t, headers: r.headers ? gr(r.headers) : {}, status: r.status, statusText: r.statusText } : t;
}
K.AST = _t, K.Minimatch = Ut, K.escape = function(r) {
  let { windowsPathsNoEscape: t = !1 } = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  return t ? r.replace(/[?*()[\]]/g, "[$&]") : r.replace(/[?*()[\]\\]/g, "\\$&");
}, K.unescape = Pt;
const Wr = (Oe = function(r, t, e) {
  let n = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : {};
  const o = q({ url: U(r.remoteURL, _(t)), method: "COPY", headers: { Destination: U(r.remoteURL, _(e)), Overwrite: n.overwrite === !1 ? "F" : "T", Depth: n.shallow ? "0" : "infinity" } }, r, n);
  return l = function(u) {
    H(r, u);
  }, (i = G(o, r)) && i.then || (i = Promise.resolve(i)), l ? i.then(l) : i;
  var i, l;
}, function() {
  for (var r = [], t = 0; t < arguments.length; t++) r[t] = arguments[t];
  try {
    return Promise.resolve(Oe.apply(this, r));
  } catch (e) {
    return Promise.reject(e);
  }
});
var Oe, se = k(635), zr = k(829), ut = k.n(zr), wt = function(r) {
  return r.Array = "array", r.Object = "object", r.Original = "original", r;
}(wt || {});
function St(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : wt.Original;
  const n = ut().get(r, t);
  return e === "array" && Array.isArray(n) === !1 ? [n] : e === "object" && Array.isArray(n) ? n[0] : n;
}
function jt(r) {
  return new Promise((t) => {
    t(function(e) {
      const { multistatus: n } = e;
      if (n === "") return { multistatus: { response: [] } };
      if (!n) throw new Error("Invalid response: No root multistatus found");
      const o = { multistatus: Array.isArray(n) ? n[0] : n };
      return ut().set(o, "multistatus.response", St(o, "multistatus.response", wt.Array)), ut().set(o, "multistatus.response", ut().get(o, "multistatus.response").map((i) => function(l) {
        const u = Object.assign({}, l);
        return u.status ? ut().set(u, "status", St(u, "status", wt.Object)) : (ut().set(u, "propstat", St(u, "propstat", wt.Object)), ut().set(u, "propstat.prop", St(u, "propstat.prop", wt.Object))), u;
      }(i))), o;
    }(new se.XMLParser({ allowBooleanAttributes: !0, attributeNamePrefix: "", textNodeName: "text", ignoreAttributes: !1, removeNSPrefix: !0, numberParseOptions: { hex: !0, leadingZeros: !1 }, attributeValueProcessor: (e, n, o) => n === "true" || n === "false" ? n === "true" : n, tagValueProcessor(e, n, o) {
      if (!o.endsWith("propstat.prop.displayname")) return n;
    } }).parse(r)));
  });
}
function Ft(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 && arguments[2];
  const { getlastmodified: n = null, getcontentlength: o = "0", resourcetype: i = null, getcontenttype: l = null, getetag: u = null } = r, y = i && typeof i == "object" && i.collection !== void 0 ? "directory" : "file", c = { filename: t, basename: Ct().basename(t), lastmod: n, size: parseInt(o, 10), type: y, etag: typeof u == "string" ? u.replace(/"/g, "") : null };
  return y === "file" && (c.mime = l && typeof l == "string" ? l.split(";")[0] : ""), e && (r.displayname !== void 0 && (r.displayname = String(r.displayname)), c.props = r), c;
}
function Ge(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 && arguments[2], n = null;
  try {
    r.multistatus.response[0].propstat && (n = r.multistatus.response[0]);
  } catch {
  }
  if (!n) throw new Error("Failed getting item stat: bad response");
  const { propstat: { prop: o, status: i } } = n, [l, u, y] = i.split(" ", 3), c = parseInt(u, 10);
  if (c >= 400) {
    const s = new Error(`Invalid response: ${c} ${y}`);
    throw s.status = c, s;
  }
  return Ft(o, Et(t), e);
}
function qe(r) {
  switch (String(r)) {
    case "-3":
      return "unlimited";
    case "-2":
    case "-1":
      return "unknown";
    default:
      return parseInt(String(r), 10);
  }
}
function Gt(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const oe = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const { details: n = !1 } = e, o = q({ url: U(r.remoteURL, _(t)), method: "PROPFIND", headers: { Accept: "text/plain,application/xml", Depth: "0" } }, r, e);
  return Gt(G(o, r), function(i) {
    return H(r, i), Gt(i.text(), function(l) {
      return Gt(jt(l), function(u) {
        const y = Ge(u, t, n);
        return bt(i, y, n);
      });
    });
  });
});
function He(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const Gr = Xe(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = function(i) {
    if (!i || i === "/") return [];
    let l = i;
    const u = [];
    do
      u.push(l), l = Ct().dirname(l);
    while (l && l !== "/");
    return u;
  }(Et(t));
  n.sort((i, l) => i.length > l.length ? 1 : l.length > i.length ? -1 : 0);
  let o = !1;
  return function(i, l, u) {
    if (typeof i[Te] == "function") {
      let g = function(m) {
        try {
          for (; !(y = h.next()).done; ) if ((m = l(y.value)) && m.then) {
            if (!je(m)) return void m.then(g, s || (s = tt.bind(null, c = new xt(), 2)));
            m = m.v;
          }
          c ? tt(c, 1, m) : c = m;
        } catch (N) {
          tt(c || (c = new xt()), 2, N);
        }
      };
      var y, c, s, h = i[Te]();
      if (g(), h.return) {
        var a = function(m) {
          try {
            y.done || h.return();
          } catch {
          }
          return m;
        };
        if (c && c.then) return c.then(a, function(m) {
          throw a(m);
        });
        a();
      }
      return c;
    }
    if (!("length" in i)) throw new TypeError("Object is not iterable");
    for (var d = [], p = 0; p < i.length; p++) d.push(i[p]);
    return function(g, m, N) {
      var f, v, x = -1;
      return function A(b) {
        try {
          for (; ++x < g.length && (!N || !N()); ) if ((b = m(x)) && b.then) {
            if (!je(b)) return void b.then(A, v || (v = tt.bind(null, f = new xt(), 2)));
            b = b.v;
          }
          f ? tt(f, 1, b) : f = b;
        } catch (O) {
          tt(f || (f = new xt()), 2, O);
        }
      }(), f;
    }(d, function(g) {
      return l(d[g]);
    }, u);
  }(n, function(i) {
    return l = function() {
      return function(y, c) {
        try {
          var s = He(oe(r, i), function(h) {
            if (h.type !== "directory") throw new Error(`Path includes a file: ${t}`);
          });
        } catch (h) {
          return c(h);
        }
        return s && s.then ? s.then(void 0, c) : s;
      }(0, function(y) {
        const c = y;
        return function() {
          if (c.status === 404) return o = !0, Ee(Qt(r, i, { ...e, recursive: !1 }));
          throw y;
        }();
      });
    }, (u = function() {
      if (o) return Ee(Qt(r, i, { ...e, recursive: !1 }));
    }()) && u.then ? u.then(l) : l();
    var l, u;
  }, function() {
    return !1;
  });
});
function Xe(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}
function qr() {
}
function Ee(r, t) {
  return r && r.then ? r.then(qr) : Promise.resolve();
}
const Te = typeof Symbol < "u" ? Symbol.iterator || (Symbol.iterator = Symbol("Symbol.iterator")) : "@@iterator";
function tt(r, t, e) {
  if (!r.s) {
    if (e instanceof xt) {
      if (!e.s) return void (e.o = tt.bind(null, r, t));
      1 & t && (t = e.s), e = e.v;
    }
    if (e && e.then) return void e.then(tt.bind(null, r, t), tt.bind(null, r, 2));
    r.s = t, r.v = e;
    const n = r.o;
    n && n(r);
  }
}
const xt = function() {
  function r() {
  }
  return r.prototype.then = function(t, e) {
    const n = new r(), o = this.s;
    if (o) {
      const i = 1 & o ? t : e;
      if (i) {
        try {
          tt(n, 1, i(this.v));
        } catch (l) {
          tt(n, 2, l);
        }
        return n;
      }
      return this;
    }
    return this.o = function(i) {
      try {
        const l = i.v;
        1 & i.s ? tt(n, 1, t ? t(l) : l) : e ? tt(n, 1, e(l)) : tt(n, 2, l);
      } catch (l) {
        tt(n, 2, l);
      }
    }, n;
  }, r;
}();
function je(r) {
  return r instanceof xt && 1 & r.s;
}
const Qt = Xe(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  if (e.recursive === !0) return Gr(r, t, e);
  const n = q({ url: U(r.remoteURL, (o = _(t), o.endsWith("/") ? o : o + "/")), method: "MKCOL" }, r, e);
  var o;
  return He(G(n, r), function(i) {
    H(r, i);
  });
});
var Hr = k(388), Se = k.n(Hr);
const Xr = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = {};
  if (typeof e.range == "object" && typeof e.range.start == "number") {
    let u = `bytes=${e.range.start}-`;
    typeof e.range.end == "number" && (u = `${u}${e.range.end}`), n.Range = u;
  }
  const o = q({ url: U(r.remoteURL, _(t)), method: "GET", headers: n }, r, e);
  return l = function(u) {
    if (H(r, u), n.Range && u.status !== 206) {
      const y = new Error(`Invalid response code for partial request: ${u.status}`);
      throw y.status = u.status, y;
    }
    return e.callback && setTimeout(() => {
      e.callback(u);
    }, 0), u.body;
  }, (i = G(o, r)) && i.then || (i = Promise.resolve(i)), l ? i.then(l) : i;
  var i, l;
}), Zr = () => {
}, Yr = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t, e) {
  e.url || (e.url = U(r.remoteURL, _(t)));
  const n = q(e, r, {});
  return i = function(l) {
    return H(r, l), l;
  }, (o = G(n, r)) && o.then || (o = Promise.resolve(o)), i ? o.then(i) : o;
  var o, i;
}), Kr = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = q({ url: U(r.remoteURL, _(t)), method: "DELETE" }, r, e);
  return i = function(l) {
    H(r, l);
  }, (o = G(n, r)) && o.then || (o = Promise.resolve(o)), i ? o.then(i) : o;
  var o, i;
}), Jr = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  return function(n, o) {
    try {
      var i = (l = oe(r, t, e), u = function() {
        return !0;
      }, y ? u ? u(l) : l : (l && l.then || (l = Promise.resolve(l)), u ? l.then(u) : l));
    } catch (c) {
      return o(c);
    }
    var l, u, y;
    return i && i.then ? i.then(void 0, o) : i;
  }(0, function(n) {
    if (n.status === 404) return !1;
    throw n;
  });
});
function qt(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const Qr = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = q({ url: U(r.remoteURL, _(t), "/"), method: "PROPFIND", headers: { Accept: "text/plain,application/xml", Depth: e.deep ? "infinity" : "1" } }, r, e);
  return qt(G(n, r), function(o) {
    return H(r, o), qt(o.text(), function(i) {
      if (!i) throw new Error("Failed parsing directory contents: Empty response");
      return qt(jt(i), function(l) {
        const u = ge(t);
        let y = function(c, s, h) {
          let a = arguments.length > 3 && arguments[3] !== void 0 && arguments[3], d = arguments.length > 4 && arguments[4] !== void 0 && arguments[4];
          const p = Ct().join(s, "/"), { multistatus: { response: g } } = c, m = g.map((N) => {
            const f = function(x) {
              try {
                return x.replace(/^https?:\/\/[^\/]+/, "");
              } catch (A) {
                throw new et(A, "Failed normalising HREF");
              }
            }(N.href), { propstat: { prop: v } } = N;
            return Ft(v, p === "/" ? decodeURIComponent(Et(f)) : Et(Ct().relative(decodeURIComponent(p), decodeURIComponent(f))), a);
          });
          return d ? m : m.filter((N) => N.basename && (N.type === "file" || N.filename !== h.replace(/\/$/, "")));
        }(l, ge(r.remoteBasePath || r.remotePath), u, e.details, e.includeSelf);
        return e.glob && (y = function(c, s) {
          return c.filter((h) => K(h.filename, s, { matchBase: !0 }));
        }(y, e.glob)), bt(o, y, e.details);
      });
    });
  });
});
function ie(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}
const tn = ie(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = q({ url: U(r.remoteURL, _(t)), method: "GET", headers: { Accept: "text/plain" }, transformResponse: [nn] }, r, e);
  return Mt(G(n, r), function(o) {
    return H(r, o), Mt(o.text(), function(i) {
      return bt(o, i, e.details);
    });
  });
});
function Mt(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const en = ie(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = q({ url: U(r.remoteURL, _(t)), method: "GET" }, r, e);
  return Mt(G(n, r), function(o) {
    let i;
    return H(r, o), function(l, u) {
      var y = l();
      return y && y.then ? y.then(u) : u();
    }(function() {
      return Mt(o.arrayBuffer(), function(l) {
        i = l;
      });
    }, function() {
      return bt(o, i, e.details);
    });
  });
}), rn = ie(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const { format: n = "binary" } = e;
  if (n !== "binary" && n !== "text") throw new et({ info: { code: ct.InvalidOutputFormat } }, `Invalid output format: ${n}`);
  return n === "text" ? tn(r, t, e) : en(r, t, e);
}), nn = (r) => r;
function sn(r) {
  return new se.XMLBuilder({ attributeNamePrefix: "@_", format: !0, ignoreAttributes: !1, suppressEmptyNode: !0 }).build(Ze({ lockinfo: { "@_xmlns:d": "DAV:", lockscope: { exclusive: {} }, locktype: { write: {} }, owner: { href: r } } }, "d"));
}
function Ze(r, t) {
  const e = { ...r };
  for (const n in e) e.hasOwnProperty(n) && (e[n] && typeof e[n] == "object" && n.indexOf(":") === -1 ? (e[`${t}:${n}`] = Ze(e[n], t), delete e[n]) : /^@_/.test(n) === !1 && (e[`${t}:${n}`] = e[n], delete e[n]));
  return e;
}
function te(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
function Ye(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}
const on = Ye(function(r, t, e) {
  let n = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : {};
  const o = q({ url: U(r.remoteURL, _(t)), method: "UNLOCK", headers: { "Lock-Token": e } }, r, n);
  return te(G(o, r), function(i) {
    if (H(r, i), i.status !== 204 && i.status !== 200) throw ne(i);
  });
}), an = Ye(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const { refreshToken: n, timeout: o = un } = e, i = { Accept: "text/plain,application/xml", Timeout: o };
  n && (i.If = n);
  const l = q({ url: U(r.remoteURL, _(t)), method: "LOCK", headers: i, data: sn(r.contactHref) }, r, e);
  return te(G(l, r), function(u) {
    return H(r, u), te(u.text(), function(y) {
      const c = (a = y, new se.XMLParser({ removeNSPrefix: !0, parseAttributeValue: !0, parseTagValue: !0 }).parse(a)), s = ut().get(c, "prop.lockdiscovery.activelock.locktoken.href"), h = ut().get(c, "prop.lockdiscovery.activelock.timeout");
      var a;
      if (!s) throw ne(u, "No lock token received: ");
      return { token: s, serverTimeout: h };
    });
  });
}), un = "Infinite, Second-4100000000";
function Ht(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const ln = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r) {
  let t = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  const e = t.path || "/", n = q({ url: U(r.remoteURL, e), method: "PROPFIND", headers: { Accept: "text/plain,application/xml", Depth: "0" } }, r, t);
  return Ht(G(n, r), function(o) {
    return H(r, o), Ht(o.text(), function(i) {
      return Ht(jt(i), function(l) {
        const u = function(y) {
          try {
            const [c] = y.multistatus.response, { propstat: { prop: { "quota-used-bytes": s, "quota-available-bytes": h } } } = c;
            return s !== void 0 && h !== void 0 ? { used: parseInt(String(s), 10), available: qe(h) } : null;
          } catch {
          }
          return null;
        }(l);
        return bt(o, u, t.details);
      });
    });
  });
});
function Xt(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const cn = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const { details: n = !1 } = e, o = q({ url: U(r.remoteURL, _(t)), method: "SEARCH", headers: { Accept: "text/plain,application/xml", "Content-Type": r.headers["Content-Type"] || "application/xml; charset=utf-8" } }, r, e);
  return Xt(G(o, r), function(i) {
    return H(r, i), Xt(i.text(), function(l) {
      return Xt(jt(l), function(u) {
        const y = function(c, s, h) {
          const a = { truncated: !1, results: [] };
          return a.truncated = c.multistatus.response.some((d) => {
            var p, g;
            return ((g = (d.status || ((p = d.propstat) == null ? void 0 : p.status)).split(" ", 3)) == null ? void 0 : g[1]) === "507" && d.href.replace(/\/$/, "").endsWith(_(s).replace(/\/$/, ""));
          }), c.multistatus.response.forEach((d) => {
            if (d.propstat === void 0) return;
            const p = d.href.split("/").map(decodeURIComponent).join("/");
            a.results.push(Ft(d.propstat.prop, p, h));
          }), a;
        }(u, t, n);
        return bt(i, y, n);
      });
    });
  });
}), hn = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t, e) {
  let n = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : {};
  const o = q({ url: U(r.remoteURL, _(t)), method: "MOVE", headers: { Destination: U(r.remoteURL, _(e)), Overwrite: n.overwrite === !1 ? "F" : "T" } }, r, n);
  return l = function(u) {
    H(r, u);
  }, (i = G(o, r)) && i.then || (i = Promise.resolve(i)), l ? i.then(l) : i;
  var i, l;
});
var fn = k(172);
const pn = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t, e) {
  let n = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : {};
  const { contentLength: o = !0, overwrite: i = !0 } = n, l = { "Content-Type": "application/octet-stream" };
  o === !1 || (l["Content-Length"] = typeof o == "number" ? `${o}` : `${function(s) {
    if (_e(s)) return s.byteLength;
    if (Ue(s)) return s.length;
    if (typeof s == "string") return (0, fn.d)(s);
    throw new et({ info: { code: ct.DataTypeNoLength } }, "Cannot calculate data length: Invalid type");
  }(e)}`), i || (l["If-None-Match"] = "*");
  const u = q({ url: U(r.remoteURL, _(t)), method: "PUT", headers: l, data: e }, r, n);
  return c = function(s) {
    try {
      H(r, s);
    } catch (h) {
      const a = h;
      if (a.status !== 412 || i) throw a;
      return !1;
    }
    return !0;
  }, (y = G(u, r)) && y.then || (y = Promise.resolve(y)), c ? y.then(c) : y;
  var y, c;
}), Ke = /* @__PURE__ */ function(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}(function(r, t) {
  let e = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
  const n = q({ url: U(r.remoteURL, _(t)), method: "OPTIONS" }, r, e);
  return i = function(l) {
    try {
      H(r, l);
    } catch (u) {
      throw u;
    }
    return { compliance: (l.headers.get("DAV") ?? "").split(",").map((u) => u.trim()), server: l.headers.get("Server") ?? "" };
  }, (o = G(n, r)) && o.then || (o = Promise.resolve(o)), i ? o.then(i) : o;
  var o, i;
});
function Ot(r, t, e) {
  return e ? t ? t(r) : r : (r && r.then || (r = Promise.resolve(r)), t ? r.then(t) : r);
}
const gn = ae(function(r, t, e, n, o) {
  let i = arguments.length > 5 && arguments[5] !== void 0 ? arguments[5] : {};
  if (e > n || e < 0) throw new et({ info: { code: ct.InvalidUpdateRange } }, `Invalid update range ${e} for partial update`);
  const l = { "Content-Type": "application/octet-stream", "Content-Length": "" + (n - e + 1), "Content-Range": `bytes ${e}-${n}/*` }, u = q({ url: U(r.remoteURL, _(t)), method: "PUT", headers: l, data: o }, r, i);
  return Ot(G(u, r), function(y) {
    H(r, y);
  });
});
function $e(r, t) {
  var e = r();
  return e && e.then ? e.then(t) : t(e);
}
const dn = ae(function(r, t, e, n, o) {
  let i = arguments.length > 5 && arguments[5] !== void 0 ? arguments[5] : {};
  if (e > n || e < 0) throw new et({ info: { code: ct.InvalidUpdateRange } }, `Invalid update range ${e} for partial update`);
  const l = { "Content-Type": "application/x-sabredav-partialupdate", "Content-Length": "" + (n - e + 1), "X-Update-Range": `bytes=${e}-${n}` }, u = q({ url: U(r.remoteURL, _(t)), method: "PATCH", headers: l, data: o }, r, i);
  return Ot(G(u, r), function(y) {
    H(r, y);
  });
});
function ae(r) {
  return function() {
    for (var t = [], e = 0; e < arguments.length; e++) t[e] = arguments[e];
    try {
      return Promise.resolve(r.apply(this, t));
    } catch (n) {
      return Promise.reject(n);
    }
  };
}
const mn = ae(function(r, t, e, n, o) {
  let i = arguments.length > 5 && arguments[5] !== void 0 ? arguments[5] : {};
  return Ot(Ke(r, t, i), function(l) {
    let u = !1;
    return $e(function() {
      if (l.compliance.includes("sabredav-partialupdate")) return Ot(dn(r, t, e, n, o, i), function(y) {
        return u = !0, y;
      });
    }, function(y) {
      let c = !1;
      return u ? y : $e(function() {
        if (l.server.includes("Apache") && l.compliance.includes("<http://apache.org/dav/propset/fs/1>")) return Ot(gn(r, t, e, n, o, i), function(s) {
          return c = !0, s;
        });
      }, function(s) {
        if (c) return s;
        throw new et({ info: { code: ct.NotSupported } }, "Not supported");
      });
    });
  });
}), yn = "https://github.com/perry-mitchell/webdav-client/blob/master/LOCK_CONTACT.md";
function vn(r) {
  let t = arguments.length > 1 && arguments[1] !== void 0 ? arguments[1] : {};
  const { authType: e = null, remoteBasePath: n, contactHref: o = yn, ha1: i, headers: l = {}, httpAgent: u, httpsAgent: y, password: c, token: s, username: h, withCredentials: a } = t;
  let d = e;
  d || (d = h || c ? Q.Password : Q.None);
  const p = { authType: d, remoteBasePath: n, contactHref: o, ha1: i, headers: Object.assign({}, l), httpAgent: u, httpsAgent: y, password: c, remotePath: or(r), remoteURL: r, token: s, username: h, withCredentials: a };
  return ke(p, h, c, s, i), { copyFile: (g, m, N) => Wr(p, g, m, N), createDirectory: (g, m) => Qt(p, g, m), createReadStream: (g, m) => function(N, f) {
    let v = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {};
    const x = new (Se()).PassThrough();
    return Xr(N, f, v).then((A) => {
      A.pipe(x);
    }).catch((A) => {
      x.emit("error", A);
    }), x;
  }(p, g, m), createWriteStream: (g, m, N) => function(f, v) {
    let x = arguments.length > 2 && arguments[2] !== void 0 ? arguments[2] : {}, A = arguments.length > 3 && arguments[3] !== void 0 ? arguments[3] : Zr;
    const b = new (Se()).PassThrough(), O = {};
    x.overwrite === !1 && (O["If-None-Match"] = "*");
    const w = q({ url: U(f.remoteURL, _(v)), method: "PUT", headers: O, data: b, maxRedirects: 0 }, f, x);
    return G(w, f).then((E) => H(f, E)).then((E) => {
      setTimeout(() => {
        A(E);
      }, 0);
    }).catch((E) => {
      b.emit("error", E);
    }), b;
  }(p, g, m, N), customRequest: (g, m) => Yr(p, g, m), deleteFile: (g, m) => Kr(p, g, m), exists: (g, m) => Jr(p, g, m), getDirectoryContents: (g, m) => Qr(p, g, m), getFileContents: (g, m) => rn(p, g, m), getFileDownloadLink: (g) => function(m, N) {
    let f = U(m.remoteURL, _(N));
    const v = /^https:/i.test(f) ? "https" : "http";
    switch (m.authType) {
      case Q.None:
        break;
      case Q.Password: {
        const x = me(m.headers.Authorization.replace(/^Basic /i, "").trim());
        f = f.replace(/^https?:\/\//, `${v}://${x}@`);
        break;
      }
      default:
        throw new et({ info: { code: ct.LinkUnsupportedAuthType } }, `Unsupported auth type for file link: ${m.authType}`);
    }
    return f;
  }(p, g), getFileUploadLink: (g) => function(m, N) {
    let f = `${U(m.remoteURL, _(N))}?Content-Type=application/octet-stream`;
    const v = /^https:/i.test(f) ? "https" : "http";
    switch (m.authType) {
      case Q.None:
        break;
      case Q.Password: {
        const x = me(m.headers.Authorization.replace(/^Basic /i, "").trim());
        f = f.replace(/^https?:\/\//, `${v}://${x}@`);
        break;
      }
      default:
        throw new et({ info: { code: ct.LinkUnsupportedAuthType } }, `Unsupported auth type for file link: ${m.authType}`);
    }
    return f;
  }(p, g), getHeaders: () => Object.assign({}, p.headers), getQuota: (g) => ln(p, g), lock: (g, m) => an(p, g, m), moveFile: (g, m, N) => hn(p, g, m, N), putFileContents: (g, m, N) => pn(p, g, m, N), partialUpdateFileContents: (g, m, N, f, v) => mn(p, g, m, N, f, v), getDAVCompliance: (g) => Ke(p, g), search: (g, m) => cn(p, g, m), setHeaders: (g) => {
    p.headers = Object.assign({}, g);
  }, stat: (g, m) => oe(p, g, m), unlock: (g, m, N) => on(p, g, m, N) };
}
it.hT;
it.O4;
it.Kd;
it.YK;
var wn = it.UU;
it.Gu;
it.ky;
it.h4;
it.ch;
it.hq;
it.i5;
export {
  wn as a
};
