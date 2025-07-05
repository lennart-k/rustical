import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';
import { createClient } from "webdav";

@customElement("create-calendar-form")
export class CreateCalendarForm extends LitElement {
  constructor() {
    super()
  }

  protected override createRenderRoot() {
    return this
  }

  client = createClient("/caldav")

  @property()
  user: String = ''
  @property()
  principal: String = ''
  @property()
  cal_id: String = ''
  @property()
  displayname: String = ''
  @property()
  description: String = ''
  @property()
  color: String = ''
  @property()
  isSubscription: boolean = false
  @property()
  subscriptionUrl: String = ''
  @property()
  components: Set<"VEVENT" | "VTODO" | "VJOURNAL"> = new Set()

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()


  override render() {
    return html`
      <button @click=${() => this.dialog.value.showModal()}>Create calendar</button>
      <dialog ${ref(this.dialog)}>
        <h3>Create calendar</h3>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            principal (for group calendar)
            <input type="text" name="principal" value=${this.user} @change=${e => this.principal = e.target.value} />
          </label>
          <br>
          <label>
            id
            <input type="text" name="id" @change=${e => this.cal_id = e.target.value} />
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
          <label>
            Color
            <input type="color" name="color"  @change=${e => this.color = e.target.value} />
          </label>
          <br>
          <br>
          <label>
            Calendar is subscription to external calendar
            <input type="checkbox" name="is_subscription" @change=${e => this.isSubscription = e.target.checked}  />
          </label>
          <br>
          ${this.isSubscription ? html`
            <label>
              Subscription URL
              <input type="text" name="subscription_url" @change=${e => this.subscriptionUrl = e.target.value}  />
            </label>
            <br>
          `: html``}
          <br>
          ${["VEVENT", "VTODO", "VJOURNAL"].map(comp => html`
            <label>
              Support ${comp}
              <input type="checkbox" value=${comp} @change=${e => e.target.checked ? this.components.add(e.target.value) : this.components.delete(e.target.value)} />
            </label>
            <br>
          `)}
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
    await this.client.createDirectory(`/principal/${this.principal || this.user}/${this.cal_id}`, {
      data: `
      <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CS="http://calendarserver.org/ns/" xmlns:ICAL="http://apple.com/ns/ical/">
        <set>
          <prop>
            <displayname>${this.displayname}</displayname>
            ${this.description ? `<CAL:calendar-description>${this.description}</CAL:calendar-description>` : ''}
            ${this.color ? `<ICAL:calendar-color>${this.color}</ICAL:calendar-color>` : ''}
            ${(this.isSubscription && this.subscriptionUrl) ? `<CS:source><href>${this.subscriptionUrl}</href></CS:source>` : ''}
            <CAL:supported-calendar-component-set>
              ${Array.from(this.components.keys()).map(comp => `<CAL:comp name="${comp}" />`).join('\n')}
            </CAL:supported-calendar-component-set>
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
    'create-calendar-form': CreateCalendarForm
  }
}
