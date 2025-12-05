import { i, x } from "./lit-DkXrt_Iv.mjs";
import { n as n$1, t } from "./property-B8WoKf1Y.mjs";
import { e, n } from "./ref-BwbQvJBB.mjs";
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
let CreateCalendarForm = class extends i {
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
      <dialog ${n(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" name="color"  @change=${(e2) => this.color = e2.target.value} />
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
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "principal", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "addr_id", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "displayname", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "description", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "color", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "timezones", 2);
CreateCalendarForm = __decorateClass([
  t("create-birthday-calendar-form")
], CreateCalendarForm);
export {
  CreateCalendarForm
};
