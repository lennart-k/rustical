import { i as u, x as c } from "./lit-CWlWuEHk.mjs";
import { n as o, t as h } from "./property-DYFkTqgI.mjs";
import { e as m, n as d } from "./ref-nf9JiOyl.mjs";
import { a as b } from "./webdav-Bz4I5vNH.mjs";
var y = Object.defineProperty, $ = Object.getOwnPropertyDescriptor, a = (t, e, n, s) => {
  for (var i = s > 1 ? void 0 : s ? $(e, n) : e, l = t.length - 1, p; l >= 0; l--)
    (p = t[l]) && (i = (s ? p(e, n, i) : p(i)) || i);
  return s && i && y(e, n, i), i;
};
let r = class extends u {
  constructor() {
    super(), this.client = b("/caldav"), this.user = "", this.id = "", this.displayname = "", this.description = "", this.color = "", this.subscriptionUrl = "", this.components = /* @__PURE__ */ new Set(), this.dialog = m(), this.form = m();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return c`
      <button @click=${() => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${d(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${d(this.form)}>
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
          <label>
            Color
            <input type="color" name="color"  @change=${(t) => this.color = t.target.value} />
          </label>
          <br>
          <label>
            Subscription URL
            <input type="text" name="subscription_url" @change=${(t) => this.subscriptionUrl = t.target.value}  />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map((t) => c`
            <label>
              Support ${t}
              <input type="checkbox" value=${t} @change=${(e) => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} />
            </label>
          `)}
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
    if (!this.components.size) {
      alert("No calendar components selected");
      return;
    }
    return await this.client.createDirectory(`/principal/${this.user}/${this.id}`, {
      data: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${this.displayname}</displayname>
            ${this.description ? `<CAL:calendar-description>${this.description}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${this.color}</ICAL:calendar-color>` : ""}
            ${this.subscriptionUrl ? `<CS:source><href>${this.subscriptionUrl}</href></CS:source>` : ""}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map((e) => `<CAL:comp name="${e}" />`).join(`
`)}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
      </mkcol>
      `
    }), window.location.reload(), null;
  }
};
a([
  o()
], r.prototype, "user", 2);
a([
  o()
], r.prototype, "id", 2);
a([
  o()
], r.prototype, "displayname", 2);
a([
  o()
], r.prototype, "description", 2);
a([
  o()
], r.prototype, "color", 2);
a([
  o()
], r.prototype, "subscriptionUrl", 2);
a([
  o()
], r.prototype, "components", 2);
r = a([
  h("create-calendar-form")
], r);
export {
  r as CreateCalendarForm
};
