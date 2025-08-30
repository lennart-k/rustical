import { i, x } from "./lit-z6_uA4GX.mjs";
import { n as n$1, t } from "./property-D0NJdseG.mjs";
import { e, n } from "./ref-CPp9J0V5.mjs";
import { e as escapeXml } from "./index-_IB1wMbZ.mjs";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i2 = decorators.length - 1, decorator; i2 >= 0; i2--)
    if (decorator = decorators[i2])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
let EditCalendarForm = class extends i {
  constructor() {
    super();
    this.displayname = "";
    this.description = "";
    this.timezone_id = "";
    this.color = "";
    this.components = /* @__PURE__ */ new Set();
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Edit calendar</button>
      <dialog ${n(this.dialog)}>
        <h3>Edit calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <input type="text" name="timezone" .value=${this.timezone_id} @change=${(e2) => this.timezone_id = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" name="color" .value=${this.color} @change=${(e2) => this.color = e2.target.value} />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map((comp) => x`
            <label>
              Support ${comp}
              <input type="checkbox" value=${comp} ?checked=${this.components.has(comp)} @change=${(e2) => e2.target.checked ? this.components.add(e2.target.value) : this.components.delete(e2.target.value)} />
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
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "principal", 2);
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "cal_id", 2);
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "displayname", 2);
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "description", 2);
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "timezone_id", 2);
__decorateClass([
  n$1()
], EditCalendarForm.prototype, "color", 2);
__decorateClass([
  n$1({
    converter: {
      fromAttribute: (value, _type) => new Set(value ? JSON.parse(value) : []),
      toAttribute: (value, _type) => JSON.stringify(value)
    }
  })
], EditCalendarForm.prototype, "components", 2);
EditCalendarForm = __decorateClass([
  t("edit-calendar-form")
], EditCalendarForm);
export {
  EditCalendarForm
};
