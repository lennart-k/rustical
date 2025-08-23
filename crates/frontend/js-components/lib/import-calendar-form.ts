import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { escapeXml } from ".";

@customElement("import-calendar-form")
export class ImportCalendarForm extends LitElement {
  constructor() {
    super()
  }

  protected override createRenderRoot() {
    return this
  }

  @property()
  user: string = ''
  @property()
  principal: string
  @property()
  cal_id: string = self.crypto.randomUUID()
  @property()
  timezone_id: string = ''

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()
  file: File;


  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Import calendar</button>
      <dialog ${ref(this.dialog)}>
        <h3>Import calendar</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            principal (for group calendars)
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
            <input type="text" name="id" value=${this.cal_id} @change=${e => this.cal_id = e.target.value} />
          </label>
          <br>
          <label>
            file
            <input type="file" accept="text/calendar" name="file" @change=${e => this.file = e.target.files[0]} />
          </label>
          <button type="submit">Import</button>
          <button type="submit" @click=${event => { event.preventDefault(); this.dialog.value.close(); this.form.value.reset() }} class="cancel">Cancel</button>
      </form>
      </dialog>
        `
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
    this.principal ||= this.user
    if (!this.principal) {
      alert("Empty principal")
      return
    }
    if (!this.cal_id) {
      alert("Empty id")
      return
    }
    let response = await fetch(`/caldav/principal/${this.principal}/${this.cal_id}`, {
      method: 'IMPORT',
      headers: {
        'Content-Type': 'text/calendar'
      },
      body: this.file,
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
    'import-calendar-form': ImportCalendarForm
  }
}
