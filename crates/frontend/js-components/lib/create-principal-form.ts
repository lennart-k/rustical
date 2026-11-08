import { html, LitElement, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js'
import { ADMIN_CREATED_EVENT, formParams, submitForm } from './admin-shared.ts'

@customElement("create-principal-form")
export class CreatePrincipalForm extends LitElement {
  @property({ type: Boolean, attribute: 'oidc-configured' })
  oidcConfigured: boolean = false

  @property({ type: Boolean, attribute: 'allow-password-login' })
  allowPasswordLogin: boolean = false

  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()

  protected createRenderRoot() {
    return this
  }

  override render() {
    return html`
      <button class="button" @click=${() => this.dialog.value?.showModal()}>New user</button>
      <dialog ${ref(this.dialog)}>
        <h3>Create user</h3>
        <form class="admin-create-form" ${ref(this.form)} @submit=${this.submit}>
          <label>
            ID
            <input type="text" name="id" required />
          </label>
          <label>
            Display name
            <input type="text" name="displayname" />
          </label>
          <label>
            Type
            <select name="principal_type">
              <option value="INDIVIDUAL" selected>Individual</option>
              <option value="RESOURCE">Resource</option>
              <option value="ROOM">Room</option>
            </select>
          </label>
          ${this.allowPasswordLogin ? html`
            <label>
              Password
              <input type="password" name="password" autocomplete="new-password" placeholder=${this.oidcConfigured ? 'Leave empty for OAuth only' : 'Optional local password'} />
            </label>
          ` : nothing}
          ${this.oidcConfigured ? html`
            <label>
              <input type="checkbox" name="oauth_only" value="on" />
              OAuth only (create without a local password)
            </label>
          ` : nothing}
          <div class="margin-top-m">
            <button type="submit" class="primary">Create user</button>
            <button type="button" class="cancel" @click=${() => this.dialog.value?.close()}>Cancel</button>
          </div>
        </form>
      </dialog>
    `
  }

  async submit(e: SubmitEvent) {
    e.preventDefault()
    const form = this.form.value
    if (!form) return
    const data = formParams(form)

    try {
      await submitForm('/frontend/settings/users/create', data)
    } catch (error) {
      alert(String((error as Error).message ?? error))
      return
    }

    form.reset()
    this.dialog.value?.close()
    this.dispatchEvent(new CustomEvent(ADMIN_CREATED_EVENT, { bubbles: true, composed: true, detail: { id: data.get('id') } }))
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'create-principal-form': CreatePrincipalForm
  }
}
