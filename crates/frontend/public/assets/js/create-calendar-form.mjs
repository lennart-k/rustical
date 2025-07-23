import { i, x } from "./lit-z6_uA4GX.mjs";
import { n as n$1, t } from "./property-D0NJdseG.mjs";
import { e, n, a as escapeXml } from "./index-b86iLJlP.mjs";
import { a as an } from "./webdav-D0R7xCzX.mjs";
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
    this.client = an("/caldav");
    this.user = "";
    this.principal = "";
    this.cal_id = "";
    this.displayname = "";
    this.description = "";
    this.timezone_id = "";
    this.color = "";
    this.isSubscription = false;
    this.subscriptionUrl = "";
    this.components = /* @__PURE__ */ new Set();
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${n(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group calendars)
            <select name="principal" value=${this.user} @change=${(e2) => this.principal = e2.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map((membership) => x`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" name="id" @change=${(e2) => this.cal_id = e2.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" name="displayname" value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <input type="text" name="timezone" .value=${this.timezone_id} @change=${(e2) => this.timezone_id = e2.target.value} />
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
          <br>
          <label>
            Calendar is subscription to external calendar
            <input type="checkbox" name="is_subscription" @change=${(e2) => this.isSubscription = e2.target.checked}  />
          </label>
          <br>
          ${this.isSubscription ? x`
            <label>
              Subscription URL
              <input type="text" name="subscription_url" @change=${(e2) => this.subscriptionUrl = e2.target.value}  />
            </label>
            <br>
          ` : x``}
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map((comp) => x`
            <label>
              Support ${comp}
              <input type="checkbox" value=${comp} @change=${(e2) => e2.target.checked ? this.components.add(e2.target.value) : this.components.delete(e2.target.value)} />
            </label>
            <br>
          `)}
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
    console.log(this.displayname);
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
    await this.client.createDirectory(`/principal/${this.principal || this.user}/${this.cal_id}`, {
      data: `
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
CreateCalendarForm = __decorateClass([
  t("create-calendar-form")
], CreateCalendarForm);
export {
  CreateCalendarForm
};
