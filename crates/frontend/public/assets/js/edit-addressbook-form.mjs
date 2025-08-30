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
let EditAddressbookForm = class extends i {
  constructor() {
    super();
    this.principal = "";
    this.addr_id = "";
    this.displayname = "";
    this.description = "";
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Edit addressbook</button>
      <dialog ${n(this.dialog)}>
        <h3>Edit addressbook</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" .value=${this.displayname} @change=${(e2) => this.displayname = e2.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" .value=${this.description} @change=${(e2) => this.description = e2.target.value} />
          </label>
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
    if (!this.addr_id) {
      alert("Empty id");
      return;
    }
    if (!this.displayname) {
      alert("Empty displayname");
      return;
    }
    let response = await fetch(`/carddav/principal/${this.principal}/${this.addr_id}`, {
      method: "PROPPATCH",
      headers: {
        "Content-Type": "application/xml"
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CARD:addressbook-description>${escapeXml(this.description)}</CARD:addressbook-description>` : ""}
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.description ? "<CARD:calendar-description />" : ""}
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
], EditAddressbookForm.prototype, "principal", 2);
__decorateClass([
  n$1()
], EditAddressbookForm.prototype, "addr_id", 2);
__decorateClass([
  n$1()
], EditAddressbookForm.prototype, "displayname", 2);
__decorateClass([
  n$1()
], EditAddressbookForm.prototype, "description", 2);
EditAddressbookForm = __decorateClass([
  t("edit-addressbook-form")
], EditAddressbookForm);
export {
  EditAddressbookForm
};
