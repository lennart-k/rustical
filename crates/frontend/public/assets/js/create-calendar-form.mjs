import { i as m, x as c } from "./lit-Dq9MfRDi.mjs";
import { n as s, t as d } from "./property-DwhV4xIV.mjs";
import { a as u } from "./webdav-Bz4I5vNH.mjs";
var h = Object.defineProperty, b = Object.getOwnPropertyDescriptor, a = (e, t, o, n) => {
  for (var i = n > 1 ? void 0 : n ? b(t, o) : t, l = e.length - 1, p; l >= 0; l--)
    (p = e[l]) && (i = (n ? p(t, o, i) : p(i)) || i);
  return n && i && h(t, o, i), i;
};
let r = class extends m {
  constructor() {
    super(), this.client = u("/caldav"), this.user = "", this.id = "", this.displayname = "", this.description = "", this.color = "", this.subscriptionUrl = "", this.components = /* @__PURE__ */ new Set();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return c`
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
          <label>
            Color
            <input type="color" name="color"  @change=${(e) => this.color = e.target.value} />
          </label>
          <br>
          <label>
            Subscription URL
            <input type="text" name="subscription_url" @change=${(e) => this.subscriptionUrl = e.target.value}  />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map((e) => c`
            <label>
              Support ${e}
              <input type="checkbox" value=${e} @change=${(t) => t.target.checked ? this.components.add(t.target.value) : this.components.delete(t.target.value)} />
            </label>
          `)}
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
            ${this.subscriptionUrl ? `<CS:source>${this.subscriptionUrl}</CS:source>` : ""}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map((t) => `<CAL:comp name="${t}" />`).join(`
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
  s()
], r.prototype, "user", 2);
a([
  s()
], r.prototype, "id", 2);
a([
  s()
], r.prototype, "displayname", 2);
a([
  s()
], r.prototype, "description", 2);
a([
  s()
], r.prototype, "color", 2);
a([
  s()
], r.prototype, "subscriptionUrl", 2);
a([
  s()
], r.prototype, "components", 2);
r = a([
  d("create-calendar-form")
], r);
export {
  r as CreateCalendarForm
};
