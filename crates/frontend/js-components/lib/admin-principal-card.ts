import { html, LitElement, nothing } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import {
  ADMIN_CHANGED_EVENT,
  ADMIN_DELETED_EVENT,
  PRINCIPALS_URL,
  PrincipalCardData,
  PrincipalView,
  adminFetch,
  formParams,
  groupPath,
  principalPath,
  submitForm,
} from './admin-shared.ts'
import './admin-principal-suggest.ts'

@customElement("admin-principal-card")
export class AdminPrincipalCard extends LitElement {
  @property({ attribute: 'principal-id' })
  principalId: string = ''

  @property({ type: Boolean })
  embedded: boolean = false

  @state()
  private data: PrincipalCardData | null = null

  @state()
  private error: string = ''

  @state()
  private showPasswordForm: boolean = false

  protected createRenderRoot() {
    return this
  }

  protected firstUpdated() {
    this.load()
  }

  async load() {
    if (!this.principalId) {
      this.error = 'No principal id given.'
      return
    }

    try {
      const response = await adminFetch(`${principalPath(this.principalId)}/data`, {
        headers: { Accept: 'application/json' }
      })
      if (!response.ok) {
        this.data = null
        this.error = (await response.text()) || 'Could not load this principal.'
        return
      }
      this.data = await response.json() as PrincipalCardData
      this.error = ''
      this.showPasswordForm = false
    } catch (error) {
      this.data = null
      this.error = String((error as Error).message ?? error)
    }
  }

  private notifyChanged() {
    this.dispatchEvent(new CustomEvent(ADMIN_CHANGED_EVENT, { bubbles: true, composed: true }))
  }

  private async run(action: string, data: URLSearchParams) {
    try {
      await submitForm(action, data)
    } catch (error) {
      alert(String((error as Error).message ?? error))
      return
    }

    await this.load()
    this.notifyChanged()
  }

  private saveProfile(event: SubmitEvent) {
    event.preventDefault()
    const form = event.target as HTMLFormElement
    this.run(`${principalPath(this.principalId)}/update`, formParams(form))
  }

  private changePassword(event: SubmitEvent) {
    event.preventDefault()
    const form = event.target as HTMLFormElement
    this.run(`${principalPath(this.principalId)}/update`, formParams(form))
  }

  private toggleOAuthOnly(event: Event) {
    const checked = (event.target as HTMLInputElement).checked
    if (checked) {
      this.run(`${principalPath(this.principalId)}/update`, new URLSearchParams({ remove_password: 'on' }))
    } else {
      this.showPasswordForm = true
    }
  }

  private addMembership(event: SubmitEvent) {
    event.preventDefault()
    const form = event.target as HTMLFormElement
    const data = formParams(form)
    form.reset()
    this.run(`${principalPath(this.principalId)}/memberships/add`, data)
  }

  private removeMembership(group: string) {
    if (this.data?.admin_group === group && this.data.is_current_user
      && !confirm('Remove yourself from the admin group? You will lose admin access until the next server start, when initial admins are added back.')) {
      return
    }

    this.run(`${principalPath(this.principalId)}/memberships/remove`, new URLSearchParams({ group }))
  }

  private addGroupMember(event: SubmitEvent) {
    event.preventDefault()
    const form = event.target as HTMLFormElement
    const data = formParams(form)
    form.reset()
    this.run(`${groupPath(this.principalId)}/members/add`, data)
  }

  private removeGroupMember(memberId: string) {
    this.run(`${groupPath(this.principalId)}/members/remove`, new URLSearchParams({ principal: memberId }))
  }

  private async deletePrincipal() {
    const isAdminGroup = this.data !== null && this.data.admin_group === this.principalId
    const message = isAdminGroup
      ? `Delete the admin group "${this.principalId}"? All admins will lose admin access until the next server start, when initial admins are added back.`
      : `Delete principal "${this.principalId}"?`
    if (!confirm(message)) {
      return
    }

    try {
      await submitForm(`${principalPath(this.principalId)}/delete`, new URLSearchParams())
    } catch (error) {
      alert(String((error as Error).message ?? error))
      return
    }

    if (this.embedded) {
      this.dispatchEvent(new CustomEvent(ADMIN_DELETED_EVENT, { bubbles: true, composed: true }))
    } else {
      window.location.assign(PRINCIPALS_URL)
    }
  }

  private submitClosestForm(event: Event) {
    const form = (event.target as HTMLElement).closest('form')
    if (form && typeof form.requestSubmit === 'function') {
      form.requestSubmit()
    }
  }

  override render() {
    return html`
      ${this.embedded ? nothing : html`
        <p class="admin-overlay-back"><a href=${PRINCIPALS_URL}>Back to principals and groups</a></p>
      `}
      <h2>${this.principalId}</h2>
      ${this.error ? html`<p class="admin-status">${this.error}</p>` : nothing}
      ${this.data ? this.renderCards(this.data) : nothing}
    `
  }

  private renderCards(data: PrincipalCardData) {
    const principal = data.principal
    const passwordFieldHidden = data.oidc_configured && !principal.has_password && !this.showPasswordForm

    return html`
      <div class="admin-grid">
        <div class="admin-card">
          <p><strong>Admin:</strong> ${principal.is_admin ? 'yes' : 'no'}</p>
          <p><strong>Type:</strong> ${principal.principal_type}</p>
          <p><strong>Local password:</strong> ${principal.has_password ? 'enabled' : 'disabled'}</p>
          ${data.is_current_user ? html`<p>You are signed in as this principal.</p>` : nothing}
        </div>

        <form class="admin-card" @submit=${this.saveProfile}>
          <h3>Principal</h3>
          <label>
            Display name
            <input type="text" name="displayname" .value=${principal.displayname ?? ''} />
          </label>
          <button type="submit">Save changes</button>
        </form>

        ${data.allow_password_login ? html`
          <div class="admin-card">
            <h3>Authentication</h3>
            ${data.oidc_configured ? html`
              <label class="admin-oauth-toggle">
                <input type="checkbox" .checked=${!principal.has_password} @change=${this.toggleOAuthOnly} />
                <span>OAuth only</span>
              </label>
            ` : nothing}
            <form ?hidden=${passwordFieldHidden} @submit=${this.changePassword}>
              <label>
                Set or change password
                <input type="password" name="password" autocomplete="new-password" placeholder="Leave empty to keep the current password" />
              </label>
              <button type="submit">Change password</button>
            </form>
          </div>
        ` : nothing}

        ${this.renderMemberships(principal)}

        ${principal.principal_type === 'GROUP' ? this.renderGroupMembers(data) : this.renderMembers(data)}

        <section class="admin-danger-zone">
          <button
            type="button"
            class="delete"
            ?disabled=${data.is_current_user}
            title=${data.is_current_user ? 'You cannot delete the signed-in principal' : nothing}
            @click=${this.deletePrincipal}
          >Delete principal</button>
        </section>
      </div>
    `
  }

  private renderMemberships(principal: PrincipalView) {
    return html`
      <section class="admin-card">
        <h3>Group memberships</h3>
        ${principal.memberships.length > 0 ? html`
          <table class="admin-inline-table">
            <tbody>
              ${principal.memberships.map(membership => html`
                <tr>
                  <td>${membership}</td>
                  <td><button type="button" @click=${() => this.removeMembership(membership)}>Remove</button></td>
                </tr>
              `)}
            </tbody>
          </table>
        ` : html`<p>This principal has no group memberships.</p>`}

        <form class="admin-suggest" @submit=${this.addMembership}>
          <label>
            Add group
            <admin-principal-suggest kind="group" name="group" placeholder="Search groups by ID or display name" @suggest-pick=${this.submitClosestForm}></admin-principal-suggest>
          </label>
          <button type="submit">Add group</button>
        </form>
      </section>
    `
  }

  private renderGroupMembers(data: PrincipalCardData) {
    return html`
      <section class="admin-card">
        <h3>Group members</h3>
        ${data.members.length > 0 ? html`
          <table class="admin-inline-table">
            <tbody>
              ${data.members.map(member => html`
                <tr>
                  <td>${member.id}</td>
                  <td>${member.principal_type}</td>
                  <td><button type="button" @click=${() => this.removeGroupMember(member.id)}>Remove</button></td>
                </tr>
              `)}
            </tbody>
          </table>
        ` : html`<p>No principals are members of this group.</p>`}

        <form class="admin-suggest" @submit=${this.addGroupMember}>
          <label>
            Add principal to group
            <admin-principal-suggest kind="person" name="principal" placeholder="Search users by ID or display name" @suggest-pick=${this.submitClosestForm}></admin-principal-suggest>
          </label>
          <button type="submit">Add member</button>
        </form>
      </section>
    `
  }

  private renderMembers(data: PrincipalCardData) {
    return html`
      <section class="admin-card">
        <h3>Members</h3>
        ${data.members.length > 0 ? html`
          <ul>
            ${data.members.map(member => html`<li>${member.id} · ${member.principal_type}</li>`)}
          </ul>
        ` : html`<p>No principals directly implement this principal.</p>`}
      </section>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'admin-principal-card': AdminPrincipalCard
  }
}
