import { html, LitElement, nothing } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import {
  ADMIN_CHANGED_EVENT,
  ADMIN_CREATED_EVENT,
  ADMIN_DELETED_EVENT,
  PrincipalItem,
  SEARCH_URL,
  SearchResponse,
  adminFetch,
  closeAdminOverlay,
  openAdminOverlay,
} from './admin-shared.ts'
import './admin-principal-card.ts'
import './create-principal-form.ts'
import './create-group-form.ts'

@customElement("admin-principal-table")
export class AdminPrincipalTable extends LitElement {
  @property({ type: Boolean, attribute: 'oidc-configured' })
  oidcConfigured: boolean = false

  @property({ type: Boolean, attribute: 'allow-password-login' })
  allowPasswordLogin: boolean = false

  @state()
  private q: string = ''

  @state()
  private page: number = 1

  @state()
  private items: PrincipalItem[] = []

  @state()
  private result: SearchResponse | null = null

  @state()
  private error: string = ''

  private timer?: ReturnType<typeof setTimeout>

  protected createRenderRoot() {
    return this
  }

  connectedCallback() {
    super.connectedCallback()

    const params = new URLSearchParams(window.location.search)
    this.q = params.get('q') ?? ''
    const page = Number(params.get('page') ?? '1')
    this.page = Number.isFinite(page) && page >= 1 ? page : 1

    document.addEventListener(ADMIN_CHANGED_EVENT, this.reload)
    document.addEventListener(ADMIN_DELETED_EVENT, this.onDeleted)
    document.addEventListener(ADMIN_CREATED_EVENT, this.onCreated)
    this.load()
  }

  disconnectedCallback() {
    document.removeEventListener(ADMIN_CHANGED_EVENT, this.reload)
    document.removeEventListener(ADMIN_DELETED_EVENT, this.onDeleted)
    document.removeEventListener(ADMIN_CREATED_EVENT, this.onCreated)
    super.disconnectedCallback()
  }

  private reload = () => {
    this.load()
  }

  private onDeleted = () => {
    closeAdminOverlay()
    this.load()
  }

  private onCreated = (event: Event) => {
    this.load()
    const id = (event as CustomEvent<{ id?: string }>).detail?.id
    if (id) this.openDetail(id)
  }

  async load() {
    const params = new URLSearchParams()
    if (this.q) params.set('q', this.q)
    params.set('page', String(this.page))

    try {
      const response = await adminFetch(`${SEARCH_URL}?${params.toString()}`, {
        headers: { Accept: 'application/json' }
      })
      if (!response.ok) {
        this.items = []
        this.result = null
        this.error = await response.text()
        return
      }

      const data = await response.json() as SearchResponse
      this.items = data.items ?? []
      this.result = data
      this.error = ''
      this.updateUrl()
    } catch (error) {
      this.items = []
      this.result = null
      this.error = String((error as Error).message ?? error)
    }
  }

  private onSearch(event: Event) {
    this.q = (event.target as HTMLInputElement).value
    this.page = 1
    if (this.timer) clearTimeout(this.timer)
    this.timer = setTimeout(() => this.load(), 160)
  }

  private goToPage(page: number) {
    this.page = page
    this.load()
  }

  private openDetail(id: string) {
    const card = document.createElement('admin-principal-card')
    card.principalId = id
    card.embedded = true
    openAdminOverlay(card)
  }

  private updateUrl() {
    const url = new URL(window.location.href)

    if (this.q) {
      url.searchParams.set('q', this.q)
    } else {
      url.searchParams.delete('q')
    }

    if (this.page > 1) {
      url.searchParams.set('page', String(this.page))
    } else {
      url.searchParams.delete('page')
    }

    window.history.replaceState({}, '', url)
  }

  override render() {
    return html`
      <section class="admin-list">
        <div class="admin-toolbar">
          <div class="admin-search">
            <label class="admin-search-label" for="admin-search">Search principals and groups</label>
            <input
              id="admin-search"
              type="search"
              .value=${this.q}
              placeholder="Search by ID, display name, type, or group"
              autocomplete="off"
              @input=${this.onSearch}
            />
          </div>
          <create-principal-form ?oidc-configured=${this.oidcConfigured} ?allow-password-login=${this.allowPasswordLogin}></create-principal-form>
          <create-group-form></create-group-form>
        </div>

        <table class="admin-table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Display name</th>
              <th>Type</th>
              <th>Groups</th>
              <th>Admin</th>
              <th>Password</th>
            </tr>
          </thead>
          <tbody>
            ${this.error
        ? html`<tr><td colspan="6">${this.error}</td></tr>`
        : this.items.length === 0
          ? html`<tr><td colspan="6">No principals match this search.</td></tr>`
          : this.items.map(item => html`
            <tr
              class="admin-row"
              tabindex="0"
              role="button"
              @click=${() => this.openDetail(item.id)}
              @keydown=${(event: KeyboardEvent) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault()
              this.openDetail(item.id)
            }
          }}
            >
              <td>${item.id}</td>
              <td>${item.display_name ?? ''}</td>
              <td>${item.principal_type}</td>
              <td>${item.memberships.join(', ')}</td>
              <td>${item.is_admin ? 'yes' : 'no'}</td>
              <td>${item.has_password ? 'yes' : 'no'}</td>
            </tr>
          `)}
          </tbody>
          <tfoot>
            ${this.result ? this.renderPagination(this.result) : nothing}
          </tfoot>
        </table>
      </section>
    `
  }

  private renderPagination(result: SearchResponse) {
    const previousPage = result.previous_page
    const nextPage = result.next_page

    return html`
      <tr>
        <td colspan="6">
          <div class="admin-pagination">
            <span>${result.total} result(s) · page ${result.page} of ${result.total_pages} · max ${result.per_page}</span>
            ${previousPage ? html`<button type="button" @click=${() => this.goToPage(previousPage)}>Previous</button>` : nothing}
            ${nextPage ? html`<button type="button" @click=${() => this.goToPage(nextPage)}>Next</button>` : nothing}
          </div>
        </td>
      </tr>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'admin-principal-table': AdminPrincipalTable
  }
}
