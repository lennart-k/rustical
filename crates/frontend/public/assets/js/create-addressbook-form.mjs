import { i as c, x as u } from "./lit-CWlWuEHk.mjs";
import { e as d, n as m, a as o, t as h } from "./ref-DuYNkSJ_.mjs";
import { a as b } from "./webdav-Bz4I5vNH.mjs";
var y = Object.defineProperty, f = Object.getOwnPropertyDescriptor, a = (t, s, l, r) => {
  for (var e = r > 1 ? void 0 : r ? f(s, l) : s, n = t.length - 1, p; n >= 0; n--)
    (p = t[n]) && (e = (r ? p(s, l, e) : p(e)) || e);
  return r && e && y(s, l, e), e;
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
a([
  o()
], i.prototype, "user", 2);
a([
  o()
], i.prototype, "id", 2);
a([
  o()
], i.prototype, "displayname", 2);
a([
  o()
], i.prototype, "description", 2);
i = a([
  h("create-addressbook-form")
], i);
export {
  i as CreateAddressbookForm
};
