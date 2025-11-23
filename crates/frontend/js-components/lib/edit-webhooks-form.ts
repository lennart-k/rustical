import { html, LitElement } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';

interface WebhookSubscription {
  id: string;
  target_url: string;
  resource_type: string;
  resource_id: string;
  secret_key?: string | null;
}

@customElement("edit-webhooks-form")
export class EditWebhooksForm extends LitElement {
  protected override createRenderRoot() { return this }

  @property() resource_type: string = ''
  @property() resource_id: string = ''

  @state() subscriptions: WebhookSubscription[] = []
  @state() editingId: string | null = null
  @state() id: string = ''
  @state() target_url: string = ''
  @state() secret_key: string = ''

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()

  override render() {
    return html`
      <button @click=${() => { this.dialog.value.showModal(); this.load(); }}>Webhooks</button>
      <dialog ${ref(this.dialog)}>
        <h3>Manage webhooks</h3>
        <div class="subscriptions">
          ${this.subscriptions.length ? html`
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Target URL</th>
                <th>Secret?</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              ${this.subscriptions.map(sub => html`
                <tr>
                  <td>${sub.id}</td>
                  <td>${sub.target_url}</td>
                  <td>${sub.secret_key ? 'Yes' : 'No'}</td>
                  <td>
                    <button @click=${() => this.startEdit(sub)}>Edit</button>
                    <button @click=${() => this.delete(sub.id)}>Delete</button>
                  </td>
                </tr>`)}
            </tbody>
          </table>` : html`<p>No webhooks yet.</p>`}
        </div>
        <hr>
        <h4>${this.editingId ? 'Edit subscription' : 'Create subscription'}</h4>
        <form @submit=${this.submit} ${ref(this.form)}>
          <label>
            ID
            <input type="text" name="id" .value=${this.id} ?disabled=${this.editingId !== null} @input=${(e: any) => this.id = e.target.value} required />
          </label>
          <br>
          <label>
            Target URL
            <input type="url" name="target_url" .value=${this.target_url} @input=${(e: any) => this.target_url = e.target.value} required />
          </label>
          <br>
          <label>
            Secret (optional)
            <input type="text" name="secret_key" .value=${this.secret_key} @input=${(e: any) => this.secret_key = e.target.value} />
          </label>
          <br>
          <button type="submit">${this.editingId ? 'Update' : 'Create'}</button>
          <button @click=${(e: Event) => { e.preventDefault(); this.clearForm(); }} type="button">New</button>
          <button @click=${(e: Event) => { e.preventDefault(); this.dialog.value.close(); }} type="button" class="cancel">Close</button>
        </form>
      </dialog>
    `
  }

  async load() {
    if (!this.resource_type || !this.resource_id) return;
    const resp = await fetch(`/webhooks/subscriptions/${this.resource_type}/${this.resource_id}`)
    if (resp.ok) {
      const data = await resp.json()
      this.subscriptions = data.subscriptions || []
    } else {
      alert(`Failed loading subscriptions: ${resp.status}`)
    }
  }

  startEdit(sub: WebhookSubscription) {
    this.editingId = sub.id
    this.id = sub.id
    this.target_url = sub.target_url
    this.secret_key = sub.secret_key || ''
  }

  clearForm() {
    this.editingId = null
    this.id = ''
    this.target_url = ''
    this.secret_key = ''
  }

  async delete(id: string) {
    if (!confirm(`Delete webhook ${id}?`)) return;
    const resp = await fetch(`/webhooks/subscriptions/delete/${encodeURIComponent(id)}`, { method: 'DELETE' })
    if (!resp.ok && resp.status !== 204) {
      alert(`Failed deleting: ${resp.status}`)
      return
    }
    await this.load()
    if (this.editingId === id) this.clearForm()
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
    if (!this.id) { alert('Missing id'); return }
    if (!this.target_url) { alert('Missing target url'); return }
    if (!this.resource_type || !this.resource_id) { alert('Missing resource info'); return }
    const payload = {
      id: this.id,
      target_url: this.target_url,
      resource_type: this.resource_type,
      resource_id: this.resource_id,
      secret_key: this.secret_key ? this.secret_key : null
    }
    const resp = await fetch('/webhooks/subscriptions/upsert', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    })
    if (!resp.ok) {
      alert(`Upsert failed: ${resp.status} ${await resp.text()}`)
      return
    }
    await this.load()
    this.editingId = this.id // remain in edit mode
  }
}

declare global {
  interface HTMLElementTagNameMap { 'edit-webhooks-form': EditWebhooksForm }
}
