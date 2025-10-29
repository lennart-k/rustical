import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { escapeXml } from ".";
import { allowed_timezones } from "./allowed-timezones";

@customElement("edit-calendar-form")
export class EditCalendarForm extends LitElement {
  constructor() {
    super()
  }

  protected override createRenderRoot() {
    return this
  }

  @property()
  principal: string
  @property()
  cal_id: string

  @property()
  displayname: string = ''
  @property()
  description: string = ''
  @property()
  timezone_id: string = ''
  @property()
  color: string = ''
  @property({
    converter: {
      fromAttribute: (value, _type) => new Set(value ? JSON.parse(value) : []),
      toAttribute: (value, _type) => JSON.stringify(value)
    }
  })
  components: Set<"VEVENT" | "VTODO" | "VJOURNAL"> = new Set()

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()


  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Edit</button>
      <dialog ${ref(this.dialog)}>
        <h3>Edit calendar</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            Displayname
            <input type="text" name="displayname" .value=${this.displayname} @change=${e => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <input type="text" list="timezone-list" name="timezone" .value=${this.timezone_id} @change=${e => this.timezone_id = e.target.value} />
            <datalist id="timezone-list">
            ${allowed_timezones.map(timezone => {
              html`
              <option>${timezone}</option>
              `
            })}
            </datalist>
          </label>
          <br>
          <label>
            Description
            <input type="text" name="description" .value=${this.description} @change=${e => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" name="color" .value=${this.color} @change=${e => this.color = e.target.value} />
          </label>
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map(comp => html`
            <label>
              Support ${comp}
              <input type="checkbox" value=${comp} ?checked=${this.components.has(comp)} @change=${e => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} />
            </label>
            <br>
          `)}
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
    if (!this.cal_id) {
      alert("Empty id")
      return
    }
    if (!this.displayname) {
      alert("Empty displayname")
      return
    }
    if (!this.components.size) {
      alert("No calendar components selected")
      return
    }
    let response = await fetch(`/caldav/principal/${this.principal}/${this.cal_id}`, {
      method: 'PROPPATCH',
      headers: {
        'Content-Type': 'application/xml'
      },
      body: `
      <propertyupdate xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ''}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ''}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ''}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map(comp => `<CAL:comp name="${escapeXml(comp)}" />`).join('\n')}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
        <remove>
          <prop>
            ${!this.timezone_id ? `<CAL:calendar-timezone-id />` : ''}
            ${!this.description ? '<CAL:calendar-description />' : ''}
            ${!this.color ? '<ICAL:calendar-color />' : ''}
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
    'edit-calendar-form': EditCalendarForm
  }
}
