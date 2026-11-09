// Submits the login page's OIDC form right away (frontend.oidc_auto_redirect).
// The flow still starts with the form's POST; without JavaScript the button stays.
document.querySelector("form[data-auto-submit]")?.requestSubmit();
