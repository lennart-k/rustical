import { i, x } from "./lit-DkXrt_Iv.mjs";
import { n, t } from "./property-B8WoKf1Y.mjs";
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
let PreviewButton = class extends i {
  constructor() {
    super();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    let text = "Preview";
    return x`<form action="${this.href}" method="GET"><button class="open" type="submit">${text}</button></form>`;
  }
};
__decorateClass([
  n()
], PreviewButton.prototype, "href", 2);
PreviewButton = __decorateClass([
  t("preview-button")
], PreviewButton);
export {
  PreviewButton
};
