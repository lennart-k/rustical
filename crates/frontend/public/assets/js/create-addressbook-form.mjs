import { i as d, x as m } from "./lit-Dq9MfRDi.mjs";
import { n, t as c } from "./property-DwhV4xIV.mjs";
import { a as u } from "./webdav-Bz4I5vNH.mjs";
var h = Object.defineProperty, y = Object.getOwnPropertyDescriptor, r = (e, a, o, s) => {
  for (var t = s > 1 ? void 0 : s ? y(a, o) : a, p = e.length - 1, l; p >= 0; p--)
    (l = e[p]) && (t = (s ? l(a, o, t) : l(t)) || t);
  return s && t && h(a, o, t), t;
};
let i = class extends d {
  constructor() {
    super(), this.client = u("/carddav"), this.user = "", this.id = "", this.displayname = "", this.description = "";
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return m`
      <section>
        <h3>Create calendar</h3>
        <form @submit=${this.submit}>
          <label>
            id
            <input type="text" name="id" @change=${(e) => this.id = e.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" name="displayname" value=${this.displayname} @change=${(e) => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" @change=${(e) => this.description = e.target.value} />
          </label>
          <br>
          <button type="submit">Create</button>
        </form>
      </section>
    `;
  }
  async submit(e) {
    if (console.log(this.displayname), e.preventDefault(), !this.id) {
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
  n()
], i.prototype, "user", 2);
r([
  n()
], i.prototype, "id", 2);
r([
  n()
], i.prototype, "displayname", 2);
r([
  n()
], i.prototype, "description", 2);
i = r([
  c("create-addressbook-form")
], i);
export {
  i as CreateAddressbookForm
};
