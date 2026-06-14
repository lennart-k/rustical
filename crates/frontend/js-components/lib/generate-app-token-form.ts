import { html, LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js';


@customElement("generate-app-token-form")
export class GenerateAppTokenForm extends LitElement {
  @property()
  user: string = ''
  
  @property()
  token: string

  @property()
  uaApple: boolean = navigator.userAgent.includes('Apple') || navigator.userAgent.includes('macOS') || navigator.userAgent.includes('Macintosh')

  form: Ref<HTMLFormElement> = createRef()

  protected override createRenderRoot() {
    return this
  }

  async onSubmit(e: SubmitEvent) {
    if (e.submitter?.name === 'apple') return;
    e.preventDefault();
    const form = this.form.value
    const data = new URLSearchParams(new FormData(form));
    const res = await fetch(form.action, {
      method: form.method,
      body: data,
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
    });
    if (!res.ok) {
      alert('Error: ' + await res.text());
      return;
    }

    const token = await res.text();
    this.token = token
    form.reset();
  }

  async copyToken(e: PointerEvent) {
    await navigator.clipboard.writeText(this.token)
    e.target.textContent = 'Copied!';
  }

  override render() {
    return html`
      <form method="POST" action=${`/frontend/user/${this.user}/app_token`} @submit=${this.onSubmit} ${ref(this.form)}>
        <input type="text" name="name" placeholder="App name" required />
        <div class="generate-actions">
          <button type="submit" class="primary">Generate</button>
          ${this.uaApple ? html`
            <button type="submit" name="apple" value="true">Apple Configuration Profile (contains token)</button>
          ` : null}
        </div>
      </form>

      <div class="token-result" ?hidden="${!this.token}">
        <p class="token-result-warning">This token will only be shown once. Copy it now and keep it secret.</p>
        <div class="token-result-row">
          <code class="token-value">${this.token}</code>
          <button type="button" @click=${this.copyToken}>Copy</button>
        </div>
        <button @click=${() => location.reload()}>Done</button>
      </div>
    `
  }
}
