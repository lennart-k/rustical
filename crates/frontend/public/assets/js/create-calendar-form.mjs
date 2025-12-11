import { i, x } from "./lit-DKg0et_P.mjs";
import { n as n$1, t } from "./property-C8WJQOrH.mjs";
import { e, n } from "./ref-BivNNNRN.mjs";
import { S as SVG_ICON_CALENDAR, a as SVG_ICON_INTERNET, e as escapeXml } from "./index-fgowJCc1.mjs";
import { g as getTimezones } from "./timezones-B0vBBzCP.mjs";
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
    this.components = /* @__PURE__ */ new Set(["VEVENT", "VTODO"]);
  }
  async fetchTimezones() {
    this.timezones = await getTimezones();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${(e2) => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${n(this.dialog)} @close=${(e2) => this.resetForm()}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group calendars)
            <select required value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${(e2) => this.cal_id = e2.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${(e2) => this.timezone_id = e2.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map((timezone) => x`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${(e2) => this.color = e2.target.value} />
          </label>
          <br>
          <br>
          <label>Type</label>
          <div class="tab-radio">
            <label>
              <input type="radio" name="type" .checked=${!this.isSubscription} @change=${(e2) => this.isSubscription = false}></input>
              ${SVG_ICON_CALENDAR}
              Calendar
            </label>
            <label>
              <input type="radio" name="type" .checked=${this.isSubscription} @change=${(e2) => this.isSubscription = true}></input>
              ${SVG_ICON_INTERNET}
              webCal Subscription
            </label>
          </div>
          <br>
          ${this.isSubscription ? x`
            <label>
              Subscription URL
              <input type="text" pattern="https://.*" .required=${this.isSubscription} .value=${this.subscriptionUrl} @change=${(e2) => this.subscriptionUrl = e2.target.value}  />
            </label>
            <br>
            <br>
          ` : x``}

          <label>Components</label>
          <div>
            ${["VEVENT", "VTODO", "VJOURNAL"].map((comp) => x`
              <label>
                Support ${comp}
                <input type="checkbox" .value=${comp} @change=${(e2) => e2.target.checked ? this.components.add(e2.target.value) : this.components.delete(e2.target.value)} .checked=${this.components.has(comp)} />
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
  async submit(e2) {
    e2.preventDefault();
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
    let response = await fetch(`/caldav/principal/${this.principal || this.user}/${this.cal_id}`, {
      method: "MKCOL",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ""}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ""}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ""}
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
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "user", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "principal", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "cal_id", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "displayname", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "description", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "timezone_id", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "color", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "isSubscription", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "subscriptionUrl", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "components", 2);
__decorateClass([
  n$1()
], CreateCalendarForm.prototype, "timezones", 2);
CreateCalendarForm = __decorateClass([
  t("create-calendar-form")
], CreateCalendarForm);
export {
  CreateCalendarForm
};
