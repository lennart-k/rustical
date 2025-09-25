import { i, x } from "./lit-DkXrt_Iv.mjs";
import { n as n$1, t } from "./property-B8WoKf1Y.mjs";
import { e, n } from "./ref-BwbQvJBB.mjs";
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
let ImportAddressbookForm = class extends i {
  constructor() {
    super();
    this.user = "";
    this.addressbook_id = self.crypto.randomUUID();
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => this.dialog.value.showModal()}>Import addressbook</button>
      <dialog ${n(this.dialog)}>
        <h3>Import addressbook</h3>
        <form @submit=${this.submit} ${n(this.form)}>
          <label>
            principal (for group addressbook)
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
            <input type="text" name="id" value=${this.addressbook_id} @change=${(e2) => this.addressbook_id = e2.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" accept="text/vcard" name="file" @change=${(e2) => this.file = e2.target.files[0]} />
          </label>
          <button type="submit">Import</button>
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
    this.principal ||= this.user;
    if (!this.principal) {
      alert("Empty principal");
      return;
    }
    if (!this.addressbook_id) {
      alert("Empty id");
      return;
    }
    let response = await fetch(`/carddav/principal/${this.principal}/${this.addressbook_id}`, {
      method: "IMPORT",
      headers: {
        "Content-Type": "text/vcard"
      },
      body: this.file
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
], ImportAddressbookForm.prototype, "user", 2);
__decorateClass([
  n$1()
], ImportAddressbookForm.prototype, "principal", 2);
__decorateClass([
  n$1()
], ImportAddressbookForm.prototype, "addressbook_id", 2);
ImportAddressbookForm = __decorateClass([
  t("import-addressbook-form")
], ImportAddressbookForm);
export {
  ImportAddressbookForm
};
