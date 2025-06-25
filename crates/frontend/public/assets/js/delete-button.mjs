import { i, x } from "./lit-z6_uA4GX.mjs";
import { n, t } from "./property-D0NJdseG.mjs";
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
let DeleteButton = class extends i {
  constructor() {
    super();
    this.trash = false;
  }
  createRenderRoot() {
    return this;
  }
  render() {
    let text = this.trash ? "Move to trash" : "Delete";
    return x`<button class="delete" @click=${(e) => this._onClick(e)}>${text}</button>`;
  }
  async _onClick(event) {
    event.preventDefault();
    if (!this.trash && !confirm("Do you want to delete this collection permanently?")) {
      return;
    }
    let response = await fetch(this.href, {
      method: "DELETE",
      headers: {
        "X-No-Trashbin": this.trash ? "0" : "1"
      }
    });
    if (response.status < 200 || response.status >= 300) {
      alert("An error occured, look into the console");
      console.error(response);
      return;
    }
    window.location.reload();
  }
};
__decorateClass([
  n({ type: Boolean })
], DeleteButton.prototype, "trash", 2);
__decorateClass([
  n()
], DeleteButton.prototype, "href", 2);
DeleteButton = __decorateClass([
  t("delete-button")
], DeleteButton);
export {
  DeleteButton
};
