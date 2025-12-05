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
let ThemeSelector = class extends i {
  constructor() {
    super();
    this.theme = "modern-light";
    this.themes = [
      { value: "modern-dark", label: "Modern-Dark" },
      { value: "modern-light", label: "Modern-Light" },
      { value: "classic-light", label: "Classic-Light" }
    ];
    this.loadSavedTheme();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <select name="theme" id="theme" @change="${this.handleThemeChange}">
        ${this.themes.map((theme) => x`
          <option value="${theme.value}" ?selected="${this.theme === theme.value}">
            ${theme.label}
          </option>
        `)}
      </select>
    `;
  }
  handleThemeChange(e) {
    const selectedTheme = e.target.value;
    this.theme = selectedTheme;
    localStorage.setItem("theme", selectedTheme);
    document.body.className = selectedTheme;
    document.documentElement.setAttribute("data-theme", selectedTheme);
  }
  loadSavedTheme() {
    const savedTheme = localStorage.getItem("theme");
    if (savedTheme) {
      this.theme = savedTheme;
    } else {
      const isDarkMode = window.matchMedia("(prefers-color-scheme: dark)").matches;
      this.theme = isDarkMode ? "modern-dark" : "modern-light";
    }
    document.body.className = this.theme;
    document.documentElement.setAttribute("data-theme", this.theme);
  }
};
__decorateClass([
  n()
], ThemeSelector.prototype, "theme", 2);
ThemeSelector = __decorateClass([
  t("theme-selector")
], ThemeSelector);
export {
  ThemeSelector
};
