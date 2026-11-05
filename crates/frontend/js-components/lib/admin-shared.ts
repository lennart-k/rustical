export const PRINCIPALS_URL = '/frontend/settings/principals'
export const SEARCH_URL = '/frontend/settings/principals/search'
export const SUGGEST_URL = '/frontend/settings/principals/suggest'
const LOGIN_URL = '/frontend/login'

export const ADMIN_CHANGED_EVENT = 'admin-principal-changed'
export const ADMIN_DELETED_EVENT = 'admin-principal-deleted'
export const ADMIN_CREATED_EVENT = 'admin-principal-created'

export interface PrincipalItem {
  id: string
  display_name: string | null
  principal_type: string
  memberships: string[]
  is_admin: boolean
  has_password: boolean
  detail_url: string
}

export interface SearchResponse {
  items: PrincipalItem[]
  page: number
  per_page: number
  total: number
  total_pages: number
  previous_page: number | null
  next_page: number | null
}

export interface PrincipalView {
  id: string
  displayname: string | null
  principal_type: string
  memberships: string[]
  has_password: boolean
  is_admin: boolean
}

export interface PrincipalCardData {
  principal: PrincipalView
  members: PrincipalView[]
  is_current_user: boolean
  allow_password_login: boolean
  oidc_configured: boolean
  admin_group: string | null
}

export function principalPath(id: string): string {
  return `/frontend/settings/principals/${encodeURIComponent(id)}`
}

export function groupPath(id: string): string {
  return `/frontend/settings/groups/${encodeURIComponent(id)}`
}

export async function adminFetch(input: string, init?: RequestInit): Promise<Response> {
  const response = await fetch(input, init)
  if (response.status === 401) window.location.assign(LOGIN_URL)
  return response
}

export function formParams(form: HTMLFormElement): URLSearchParams {
  const params = new URLSearchParams()

  for (const [key, value] of new FormData(form)) {
    params.append(key, value instanceof File ? value.name : value)
  }

  return params
}

export async function submitForm(action: string, data: URLSearchParams): Promise<void> {
  const response = await adminFetch(action, {
    method: 'POST',
    body: data.toString(),
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
  })
  if (!response.ok) {
    throw new Error((await response.text()) || 'The action failed.')
  }
}

let overlay: HTMLDialogElement | null = null
let overlayContent: HTMLElement | null = null

function ensureOverlay() {
  if (overlay) return

  overlay = document.createElement('dialog')
  overlay.className = 'admin-overlay'
  overlay.innerHTML = '<div class="admin-overlay-panel" data-admin-overlay-panel><form method="dialog" class="admin-overlay-controls"><button class="admin-overlay-close" value="close">Close</button></form><div data-admin-overlay-content></div></div>'
  document.body.append(overlay)

  overlayContent = overlay.querySelector<HTMLElement>('[data-admin-overlay-content]')
}

export function openAdminOverlay(content: Node) {
  ensureOverlay()
  overlayContent?.replaceChildren(content)
  if (!overlay?.open) overlay?.showModal()
}

export function closeAdminOverlay() {
  overlay?.close()
}
