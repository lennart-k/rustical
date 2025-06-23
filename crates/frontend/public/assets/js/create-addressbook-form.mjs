import { i as c, x as u } from "./lit-CWlWuEHk.mjs";
import { n as o, t as h } from "./property-DYFkTqgI.mjs";
import { e as d, n as m } from "./ref-nf9JiOyl.mjs";
import { a as b } from "./webdav-Bz4I5vNH.mjs";
var y = Object.defineProperty, f = Object.getOwnPropertyDescriptor, r = (t, a, n, s) => {
  for (var e = s > 1 ? void 0 : s ? f(a, n) : a, l = t.length - 1, p; l >= 0; l--)
    (p = t[l]) && (e = (s ? p(a, n, e) : p(e)) || e);
  return s && e && y(a, n, e), e;
};
let i = class extends c {
  constructor() {
    super(), this.client = b("/carddav"), this.user = "", this.id = "", this.displayname = "", this.description = "", this.dialog = d(), this.form = d();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return u`
      <button @click=${() => this.dialog.value.showModal()}>Create addressbook</button>
      <dialog ${m(this.dialog)}>
        <h3>Create addressbook</h3>
        <form @submit=${this.submit} ${m(this.form)}>
          <label>
            id
            <input type="text" name="id" @change=${(t) => this.id = t.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" name="displayname" value=${this.displayname} @change=${(t) => this.displayname = t.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" @change=${(t) => this.description = t.target.value} />
          </label>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${(t) => {
      t.preventDefault(), this.dialog.value.close(), this.form.value.reset();
    }}> Cancel </button>
        </form>
      </dialog>
    `;
  }
  async submit(t) {
    if (console.log(this.displayname), t.preventDefault(), !this.id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    return await this.client.createDirectory(`/principal/${this.user}/${this.id}`, {
      data: `
      <mkcol xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${this.displayname}</displayname>
            ${this.description ? `<CARD:addressbook-description>${this.description}</CARD:addressbook-description>` : ""}
          </prop>
        </set>
      </mkcol>
      `
    }), window.location.reload(), null;
  }
};
r([
  o()
], i.prototype, "user", 2);
r([
  o()
], i.prototype, "id", 2);
r([
  o()
], i.prototype, "displayname", 2);
r([
  o()
], i.prototype, "description", 2);
i = r([
  h("create-addressbook-form")
], i);
export {
  i as CreateAddressbookForm
};
