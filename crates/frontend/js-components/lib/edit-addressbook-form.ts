import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { escapeXml } from ".";

@customElement("edit-addressbook-form")
export class EditAddressbookForm extends LitElement {
  constructor() {
    super()

  }

  protected override createRenderRoot() {
    return this
  }

  @property()
  principal: string = ''
  @property()
  addr_id: string = ''
  @property()
  displayname: string = ''
  @property()
  description: string = ''

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()

  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${ref(this.dialog)}>
        <h3>Edit addressbook</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" .value=${this.displayname} @change=${e => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" .value=${this.description} @change=${e => this.description = e.target.value} />
          </label>
          <br>
          <button type="submit">Submit</button>
          <button type="submit" @click=${event => { event.preventDefault(); this.dialog.value.close(); this.form.value.reset() }} class="cancel">Cancel</button>
        </form>
      </dialog>
    `
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
    if (!this.principal) {
      alert("Empty principal")
      return
    }
    if (!this.addr_id) {
      alert("Empty id")
      return
    }
    if (!this.displayname) {
      alert("Empty displayname")
      return
    }
    let response = await fetch(`/carddav/principal/${this.principal}/${this.addr_id}`, {
      method: 'PROPPATCH',
      headers: {
        'Content-Type': 'application/xml'
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.description ? `<CARD:addressbook-description>${escapeXml(this.description)}</CARD:addressbook-description>` : ''}
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.description ? '<CARD:calendar-description />' : ''}
          </prop>
        </remove>
      </propertyupdate>
      `

    })

    if (response.status >= 400) {
      alert(`Error ${response.status}: ${await response.text()}`)
      return null
    }

    window.location.reload()
    return null
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'edit-addressbook-form': EditAddressbookForm
  }
}
