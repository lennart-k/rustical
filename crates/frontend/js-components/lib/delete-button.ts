import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("delete-button")
export class DeleteButton extends LitElement {
  constructor() {
    super()
  }

  @property({ type: Boolean })
  trash: boolean = false
  @property()
  href: string

  protected createRenderRoot() {
    return this
  }

  protected render() {
    let text = this.trash ? 'Move to trash' : 'Delete'
    return html`<button class="delete" @click=${e => this._onClick(e)}>${text}</button>`
  }

  async _onClick(event: Event) {
    event.preventDefault()
    if (!this.trash && !confirm('Do you want to delete this collection permanently?')) {
      return
    }

    let response = await fetch(this.href, {
      method: 'DELETE',
      headers: {
        'X-No-Trashbin': this.trash ? '0' : '1'
      }
    })
    if (response.status < 200 || response.status >= 300) {
      alert('An error occured, look into the console')
      console.error(response)
      return
    }
    window.location.reload()
  }
}
