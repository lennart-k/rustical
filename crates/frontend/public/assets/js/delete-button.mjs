import { i as c, x as p } from "./lit-CWlWuEHk.mjs";
import { n as h, t as u } from "./property-DYFkTqgI.mjs";
var f = Object.defineProperty, d = Object.getOwnPropertyDescriptor, i = (r, t, n, o) => {
  for (var e = o > 1 ? void 0 : o ? d(t, n) : t, l = r.length - 1, a; l >= 0; l--)
    (a = r[l]) && (e = (o ? a(t, n, e) : a(e)) || e);
  return o && e && f(t, n, e), e;
};
let s = class extends c {
  constructor() {
    super(), this.trash = !1;
  }
  createRenderRoot() {
    return this;
  }
  render() {
    let r = this.trash ? "Move to trash" : "Delete";
    return p`<button class="delete" @click=${(t) => this._onClick(t)}>${r}</button>`;
  }
  async _onClick(r) {
    if (r.preventDefault(), !this.trash && !confirm("Do you want to delete this collection permanently?"))
      return;
    let t = await fetch(this.href, {
      method: "DELETE",
      headers: {
        "X-No-Trashbin": this.trash ? "0" : "1"
      }
    });
    if (t.status < 200 || t.status >= 300) {
      alert("An error occured, look into the console"), console.error(t);
      return;
    }
    window.location.reload();
  }
};
i([
  h({ type: Boolean })
], s.prototype, "trash", 2);
i([
  h()
], s.prototype, "href", 2);
s = i([
  u("delete-button")
], s);
export {
  s as DeleteButton
};
