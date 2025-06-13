import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
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
  id: String = ''
  @property()
  displayname: String = ''
  @property()
  description: String = ''


  override render() {
    return html`
      <section>
        <h3>Create calendar</h3>
        <form @submit=${this.submit}>
          <label>
            id
            <input type="text" name="id" @change=${e => this.id = e.target.value} />
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
        </form>
      </section>
    `
  }

  async submit(e: SubmitEvent) {
    console.log(this.displayname)
    e.preventDefault()
    if (!this.id) {
      alert("Empty id")
      return
    }
    if (!this.displayname) {
      alert("Empty displayname")
      return
    }
    // TODO: Escape user input: There's not really a security risk here but would be nicer
    await this.client.createDirectory(`/principal/${this.user}/${this.id}`, {
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
