import { html, LitElement } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { adminFetch, PrincipalItem, SUGGEST_URL } from './admin-shared.ts'

let nextSuggestId = 0

@customElement("admin-principal-suggest")
export class AdminPrincipalSuggest extends LitElement {
  @property()
  kind: string = 'all'

  @property()
  name: string = ''

  @property()
  placeholder: string = ''

  @query('input')
  input!: HTMLInputElement

  @state()
  private items: PrincipalItem[] = []

  @state()
  private visible: boolean = false

  @state()
  private activeIndex: number = -1

  private inputId: string = `admin-suggest-input-${nextSuggestId}`
  private listId: string = `admin-suggest-list-${nextSuggestId}`
  private timer?: ReturnType<typeof setTimeout>

  constructor() {
    super()
    nextSuggestId += 1
  }

  protected createRenderRoot() {
    return this
  }

  connectedCallback() {
    super.connectedCallback()
    document.addEventListener('mousedown', this.closeOutside)
  }

  disconnectedCallback() {
    document.removeEventListener('mousedown', this.closeOutside)
    super.disconnectedCallback()
  }

  get value(): string {
    return this.input?.value ?? ''
  }

  protected updated() {
    this.querySelector<HTMLElement>('li[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' })
  }

  private closeOutside = (event: MouseEvent) => {
    if (!this.contains(event.target as Node)) this.hide()
  }

  private onInput() {
    if (this.timer) clearTimeout(this.timer)
    this.timer = setTimeout(() => this.load(), 140)
  }

  private async load() {
    const q = this.input.value.trim()
    if (!q) {
      this.hide()
      return
    }

    const params = new URLSearchParams({ kind: this.kind, q, limit: '10' })

    try {
      const response = await adminFetch(`${SUGGEST_URL}?${params.toString()}`, {
        headers: { Accept: 'application/json' }
      })
      if (!response.ok) {
        this.hide()
        return
      }
      const data = await response.json() as { items?: PrincipalItem[] }
      this.items = data.items ?? []
      this.visible = this.items.length > 0
      this.activeIndex = -1
    } catch {
      this.hide()
    }
  }

  private onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      this.hide()
      return
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault()
      this.move(1)
      return
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault()
      this.move(-1)
      return
    }

    if (event.key === 'Enter' && this.activeIndex >= 0) {
      const item = this.items[this.activeIndex]
      if (item) {
        event.preventDefault()
        this.pick(item)
      }
    }
  }

  private move(delta: number) {
    if (this.items.length === 0) return
    this.activeIndex = Math.min(Math.max(this.activeIndex + delta, 0), this.items.length - 1)
  }

  private hide() {
    this.visible = false
    this.activeIndex = -1
  }

  private pick(item: PrincipalItem) {
    this.input.value = item.id
    this.hide()
    this.dispatchEvent(new CustomEvent('suggest-pick', { bubbles: true, composed: true, detail: { id: item.id } }))
  }

  private suggestionMeta(item: PrincipalItem): string {
    const extra: string[] = []
    if (item.principal_type) extra.push(item.principal_type)
    if (item.is_admin) extra.push('admin')
    if (item.memberships.length > 0) extra.push(`${item.memberships.length} groups`)
    return extra.join(' · ')
  }

  override render() {
    return html`
      <input
        id=${this.inputId}
        type="search"
        name=${this.name}
        autocomplete="off"
        placeholder=${this.placeholder}
        aria-controls=${this.listId}
        aria-expanded=${this.visible ? 'true' : 'false'}
        @input=${this.onInput}
        @keydown=${this.onKeydown}
      />
      <ul id=${this.listId} class="admin-suggestions" ?hidden=${!this.visible}>
        ${this.items.map((item, index) => html`
          <li
            data-id=${item.id}
            aria-selected=${index === this.activeIndex ? 'true' : 'false'}
            @mousedown=${(event: MouseEvent) => { event.preventDefault(); this.pick(item) }}
          >
            <span class="admin-suggestion-name">${item.display_name ? `${item.id} · ${item.display_name}` : item.id}</span>
            <span class="admin-suggestion-meta">${this.suggestionMeta(item)}</span>
          </li>
        `)}
      </ul>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'admin-principal-suggest': AdminPrincipalSuggest
  }
}
