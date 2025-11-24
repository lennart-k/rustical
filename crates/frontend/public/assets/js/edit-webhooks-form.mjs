import { i, x } from "./lit-DkXrt_Iv.mjs";
import { n, t } from "./property-B8WoKf1Y.mjs";
import { e, n as n$1 } from "./ref-BwbQvJBB.mjs";
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
function r(r2) {
  return n({ ...r2, state: true, attribute: false });
}
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __decorateClass = (decorators, target, key, kind) => {
  var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc(target, key) : target;
  for (var i2 = decorators.length - 1, decorator; i2 >= 0; i2--)
    if (decorator = decorators[i2])
      result = (kind ? decorator(target, key, result) : decorator(result)) || result;
  if (kind && result) __defProp(target, key, result);
  return result;
};
let EditWebhooksForm = class extends i {
  constructor() {
    super(...arguments);
    this.resource_type = "";
    this.resource_id = "";
    this.subscriptions = [];
    this.editingId = null;
    this.target_url = "";
    this.secret_key = "";
    this.dialog = e();
    this.form = e();
  }
  createRenderRoot() {
    return this;
  }
  render() {
    return x`
      <button @click=${() => {
      this.dialog.value.showModal();
      this.load();
    }}>Webhooks</button>
      <dialog ${n$1(this.dialog)}>
        <style>
          form .field { margin-bottom: .85rem; }
          form .field label { display:block; font-weight:600; margin-bottom:.35rem; }
          form .field input[type=url],
          form .field input[type=text] { display:block; width:100%; box-sizing:border-box; padding:.45rem .55rem; }
          form .actions-row { margin-top:.5rem; display:flex; gap:.5rem; flex-wrap:wrap; }
        </style>
        <h3>Manage webhooks</h3>
        <div class="subscriptions">
          ${this.subscriptions.length ? x`
          <table>
            <thead>
              <tr>
                <th>Target URL</th>
                <th>Secret?</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              ${this.subscriptions.map((sub) => x`
                <tr>
                  <td>${sub.target_url}</td>
                  <td>${sub.secret_key ? "Yes" : "No"}</td>
                  <td>
                    <button @click=${() => this.startEdit(sub)}>Edit</button>
                    <button @click=${() => this.delete(sub.id)}>Delete</button>
                  </td>
                </tr>`)}
            </tbody>
          </table>` : x`<p>No webhooks yet.</p>`}
        </div>
        <hr>
        <h4>${this.editingId ? "Edit subscription" : "Create subscription"}</h4>
        <form @submit=${this.submit} ${n$1(this.form)}>
          <div class="field">
            <label>Target URL</label>
            <input type="url" name="target_url" .value=${this.target_url} @input=${(e2) => this.target_url = e2.target.value} required />
          </div>
          <div class="field">
            <label>Secret (optional)</label>
            <input type="text" name="secret_key" .value=${this.secret_key} @input=${(e2) => this.secret_key = e2.target.value} />
          </div>
          <div class="actions-row">
            <button type="submit">${this.editingId ? "Update" : "Create"}</button>
            <button @click=${(e2) => {
      e2.preventDefault();
      this.clearForm();
    }} type="button">New</button>
            <button @click=${(e2) => {
      e2.preventDefault();
      this.dialog.value.close();
    }} type="button" class="cancel">Close</button>
          </div>
        </form>
      </dialog>
    `;
  }
  async load() {
    if (!this.resource_type || !this.resource_id) return;
    const resp = await fetch(`/webhooks/subscriptions/${this.resource_type}/${this.resource_id}`);
    if (resp.ok) {
      const data = await resp.json();
      this.subscriptions = data.subscriptions || [];
    } else {
      alert(`Failed loading subscriptions: ${resp.status}`);
    }
  }
  startEdit(sub) {
    this.editingId = sub.id;
    this.target_url = sub.target_url;
    this.secret_key = sub.secret_key || "";
  }
  clearForm() {
    this.editingId = null;
    this.target_url = "";
    this.secret_key = "";
  }
  async delete(id) {
    if (!confirm(`Delete webhook ${id}?`)) return;
    const resp = await fetch(`/webhooks/subscriptions/delete/${encodeURIComponent(id)}`, { method: "DELETE" });
    if (!resp.ok && resp.status !== 204) {
      alert(`Failed deleting: ${resp.status}`);
      return;
    }
    await this.load();
    if (this.editingId === id) this.clearForm();
  }
  async submit(e2) {
    e2.preventDefault();
    if (!this.target_url) {
      alert("Missing target url");
      return;
    }
    if (!this.resource_type || !this.resource_id) {
      alert("Missing resource info");
      return;
    }
    const payload = {
      target_url: this.target_url,
      resource_type: this.resource_type,
      resource_id: this.resource_id,
      secret_key: this.secret_key ? this.secret_key : null
    };
    if (this.editingId) payload.id = this.editingId;
    const resp = await fetch("/webhooks/subscriptions/upsert", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload)
    });
    if (!resp.ok) {
      alert(`Upsert failed: ${resp.status} ${await resp.text()}`);
      return;
    }
    const j = await resp.json();
    if (!this.editingId) this.editingId = j.id;
    await this.load();
  }
};
__decorateClass([
  n()
], EditWebhooksForm.prototype, "resource_type", 2);
__decorateClass([
  n()
], EditWebhooksForm.prototype, "resource_id", 2);
__decorateClass([
  r()
], EditWebhooksForm.prototype, "subscriptions", 2);
__decorateClass([
  r()
], EditWebhooksForm.prototype, "editingId", 2);
__decorateClass([
  r()
], EditWebhooksForm.prototype, "target_url", 2);
__decorateClass([
  r()
], EditWebhooksForm.prototype, "secret_key", 2);
EditWebhooksForm = __decorateClass([
  t("edit-webhooks-form")
], EditWebhooksForm);
export {
  EditWebhooksForm
};
