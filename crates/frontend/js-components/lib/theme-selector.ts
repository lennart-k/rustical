import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("theme-selector")
export class ThemeSelector extends LitElement {
  @property()
  theme: string = "modern-light";

  private themes = [
    { value: "modern-dark", label: "Modern-Dark" },
    { value: "modern-light", label: "Modern-Light" },
    { value: "classic-light", label: "Classic-Light" }
  ];

  constructor() {
    super();
    this.loadSavedTheme();
  }

  protected createRenderRoot() {
    return this;
  }

  protected render() {
    return html`
      <select name="theme" id="theme" @change="${this.handleThemeChange}">
        ${this.themes.map(theme => html`
          <option value="${theme.value}" ?selected="${this.theme === theme.value}">
            ${theme.label}
          </option>
        `)}
      </select>
    `;
  }

  private handleThemeChange(e: Event) {
    const selectedTheme = (e.target as HTMLSelectElement).value;
    this.theme = selectedTheme;
    
    localStorage.setItem("theme", selectedTheme);
    
    document.body.className = selectedTheme;
    document.documentElement.setAttribute("data-theme", selectedTheme);
  }

  private loadSavedTheme() {
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
}
