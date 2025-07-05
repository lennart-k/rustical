import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { createClient } from "webdav";

@customElement("create-addressbook-form")
export class CreateAddressbookForm extends LitElement {
  constructor() {
    super()

  }

  protected override createRenderRoot() {
    return this
  }

  client = createClient("/carddav")

  @property()
  user: String = ''
  @property()
  principal: String = ''
  @property()
  addr_id: String = ''
  @property()
  displayname: String = ''
  @property()
  description: String = ''

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()

  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Create addressbook</button>
      <dialog ${ref(this.dialog)}>
        <h3>Create addressbook</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            principal (for group addressbooks)
            <select name="principal" value=${this.user} @change=${e => this.principal = e.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map(membership => html`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" name="id" @change=${e => this.addr_id = e.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" name="displayname" value=${this.displayname} @change=${e => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" @change=${e => this.description = e.target.value} />
          </label>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${event => { event.preventDefault(); this.dialog.value.close(); this.form.value.reset() }} class="cancel">Cancel</button>
        </form>
      </dialog>
    `
  }

  async submit(e: SubmitEvent) {
    console.log(this.displayname)
    e.preventDefault()
    if (!this.addr_id) {
      alert("Empty id")
      return
    }
    if (!this.displayname) {
      alert("Empty displayname")
      return
    }
    // TODO: Escape user input: There's not really a security risk here but would be nicer
    await this.client.createDirectory(`/principal/${this.principal || this.user}/${this.addr_id}`, {
      data: `
      <mkcol xmlns="DAV:" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
        <set>
          <prop>
            <displayname>${this.displayname}</displayname>
            ${this.description ? `<CARD:addressbook-description>${this.description}</CARD:addressbook-description>` : ''}
          </prop>
        </set>
      </mkcol>
      `
    })
    window.location.reload()
    return null
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'create-addressbook-form': CreateAddressbookForm
  }
}
