import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("preview-button")
export class PreviewButton extends LitElement {
  constructor() {
    super()
  }

  @property()
  href: string

  protected createRenderRoot() {
    return this
  }

  protected render() {
    let text = "Preview";
    return html`<form action="${this.href}" method="GET"><button class="open" type="submit">${text}</button></form>`;
  }
}
