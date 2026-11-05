import { html, LitElement } from "lit";
import { customElement } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js'
import { ADMIN_CREATED_EVENT, formParams, submitForm } from './admin-shared.ts'

@customElement("create-group-form")
export class CreateGroupForm extends LitElement {
  dialog: Ref<HTMLDialogElement> = createRef()
  form: Ref<HTMLFormElement> = createRef()

  protected createRenderRoot() {
    return this
  }

  override render() {
    return html`
      <button class="button" @click=${() => this.dialog.value?.showModal()}>New group</button>
      <dialog ${ref(this.dialog)}>
        <h3>Create group</h3>
        <form class="admin-create-form" ${ref(this.form)} @submit=${this.submit}>
          <label>
            ID
            <input type="text" name="id" required />
          </label>
          <label>
            Display name
            <input type="text" name="displayname" />
          </label>
          <div class="margin-top-m">
            <button type="submit" class="primary">Create group</button>
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
      await submitForm('/frontend/settings/groups/create', data)
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
    'create-group-form': CreateGroupForm
  }
}
