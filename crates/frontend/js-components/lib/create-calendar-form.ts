import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { escapeXml, SVG_ICON_CALENDAR, SVG_ICON_INTERNET } from ".";
import { getTimezones } from "./timezones.ts";

@customElement("create-calendar-form")
export class CreateCalendarForm extends LitElement {
  constructor() {
    super()
    this.resetForm()
    this.fetchTimezones()
  }

  resetForm() {
    this.form.value?.reset()
    this.principal = this.user
    this.cal_id = self.crypto.randomUUID()
    this.displayname = ''
    this.description = ''
    this.timezone_id = ''
    this.color = ''
    this.isSubscription = false
    this.subscriptionUrl = null
    this.components = new Set(["VEVENT", "VTODO"])
  }

  async fetchTimezones() {
    this.timezones = await getTimezones()
  }

  protected override createRenderRoot() {
    return this
  }

  @property()
  user: string = ''
  @property()
  principal: string
  @property()
  cal_id: string
  @property()
  displayname: string
  @property()
  description: string
  @property()
  timezone_id: string
  @property()
  color: string
  @property()
  isSubscription: boolean
  @property()
  subscriptionUrl: string
  @property()
  components: Set<"VEVENT" | "VTODO" | "VJOURNAL">

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()
  @property()
  timezones: Array<String> = []

  override render() {
    return html`
      <button @click=${e => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${ref(this.dialog)} @close=${e => this.resetForm()}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            principal (for group calendars)
            <select required value=${this.user} @change=${e => this.principal = e.target.value}>
              <option value=${this.user}>${this.user}</option>
              ${window.rusticalUser.memberships.map(membership => html`
                <option value=${membership}>${membership}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            id
            <input type="text" required .value=${this.cal_id} @change=${e => this.cal_id = e.target.value} />
          </label>
          <br>
          <label>
            Displayname
            <input type="text" required .value=${this.displayname} @change=${e => this.displayname = e.target.value} />
          </label>
          <br>
          <label>
            Timezone (optional)
            <select .value=${this.timezone_id} @change=${e => this.timezone_id = e.target.value}>
              <option value="">No timezone</option>
              ${this.timezones.map(timezone => html`
                <option value=${timezone} ?selected=${timezone === this.timezone_id}>${timezone}</option>
              `)}
            </select>
          </label>
          <br>
          <label>
            Description
            <input type="text" .value=${this.description} @change=${e => this.description = e.target.value} />
          </label>
          <br>
          <label>
            Color
            <input type="color" .value=${this.color} @change=${e => this.color = e.target.value} />
          </label>
          <br>
          <br>
          <label>Type</label>
          <div class="tab-radio">
            <label>
              <input type="radio" name="type" .checked=${!this.isSubscription} @change=${e => this.isSubscription = false}></input>
              ${SVG_ICON_CALENDAR}
              Calendar
            </label>
            <label>
              <input type="radio" name="type" .checked=${this.isSubscription} @change=${e => this.isSubscription = true}></input>
              ${SVG_ICON_INTERNET}
              webCal Subscription
            </label>
          </div>
          <br>
          ${this.isSubscription ? html`
            <label>
              Subscription URL
              <input type="text" pattern="https://.*" .required=${this.isSubscription} .value=${this.subscriptionUrl} @change=${e => this.subscriptionUrl = e.target.value}  />
            </label>
            <br>
            <br>
          `: html``}

          <label>Components</label>
          <div>
            ${["VEVENT", "VTODO", "VJOURNAL"].map(comp => html`
              <label>
                Support ${comp}
                <input type="checkbox" .value=${comp} @change=${e => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} .checked=${this.components.has(comp)} />
              </label>
              <br>
            `)}
          </div>
          <br>
          <button type="submit">Create</button>
          <button type="submit" @click=${event => { event.preventDefault(); this.dialog.value.close();}} class="cancel">Cancel</button>
      </form>
      </dialog>
        `
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
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
    if (this.isSubscription && !this.subscriptionUrl) {
      alert("Invalid subscription url")
      return
    }

    let response = await fetch(`/caldav/principal/${this.principal || this.user}/${this.cal_id}`, {
      method: 'MKCOL',
      headers: {
        'Content-Type': 'application/xml'
      },
      body: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${escapeXml(this.displayname)}</displayname>
            ${this.timezone_id ? `<CAL:calendar-timezone-id>${escapeXml(this.timezone_id)}</CAL:calendar-timezone-id>` : ''}
            ${this.description ? `<CAL:calendar-description>${escapeXml(this.description)}</CAL:calendar-description>` : ''}
            ${this.color ? `<ICAL:calendar-color>${escapeXml(this.color)}</ICAL:calendar-color>` : ''}
            ${(this.isSubscription && this.subscriptionUrl) ? `<CS:source><href>${escapeXml(this.subscriptionUrl)}</href></CS:source>` : ''}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map(comp => `<CAL:comp name="${escapeXml(comp)}" />`).join('\n')}
            </CAL:supported-calendar-component-set>
          </prop>
        </set>
      </mkcol>
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
    'create-calendar-form': CreateCalendarForm
  }
}
