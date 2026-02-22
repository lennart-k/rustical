# OpenID Connect

You can set up RustiCal with an OpenID Connect identity provider

## Example: Authelia

```toml title="RustiCal configuration"
[oidc]
name = "Authelia"
issuer = "https://auth.example.com"
client_id = "rustical"
client_secret = "secret..."
claim_userid = "preferred_username"  # (1)!
scopes = ["openid", "profile", "groups"]
require_group = "app/rustical"  # (2)!
allow_sign_up = true

[frontend]
allow_password_login = false  # optional
```

1. Can be `preferred_username`, `email` or `sub`
2. Optional: You can require a user to be in a certain group to use RustiCal

```yaml title="Authelia configuration"
identity_providers:
  oidc:
    clients:
      - client_id: rustical
        client_secret: secret...
        public: false
        consent_mode: implicit
        scopes: [openid, profile, groups]
        token_endpoint_auth_method: client_secret_basic
        redirect_uris:
          - https://rustical.example.com/frontend/login/oidc/callback
```

### With environment variables

```sh
RUSTICAL_OIDC__NAME: "Authelia"
RUSTICAL_OIDC__ISSUER: "https://auth.example.com"
RUSTICAL_OIDC__CLIENT_ID: "rustical"
RUSTICAL_OIDC__CLIENT_SECRET: "secret..."
RUSTICAL_OIDC__CLAIM_USERID: "preferred_username"
RUSTICAL_OIDC__SCOPES: '["openid", "profile", "groups"]'
RUSTICAL_OIDC__REQUIRE_GROUP: "app:rustical"
RUSTICAL_OIDC__ALLOW_SIGN_UP: "true"
RUSTICAL_FRONTEND__ALLOW_PASSWORD_LOGIN: "false"
```

## Assigning memberships based on group claims

You can also assign principal memberships based on the group claim.

This example will add a user to `group.mygroup` if it is in the IdP's `mygroup` group:

```toml title="RustiCal configuration"
[oidc.assign_memberships]
mygroup = ["group.mygroup"]
```

```sh
RUSTICAL_OIDC__ASSIGN_GROUPS__mygroup: '["group.mygroup"]'
```

Note that:

- The group will not be automatically created, you have to do that manually through the CLI
- If adding a membership fails (e.g. because the principal is missing), the error will only be logged and the user can still log in
- Assigning memberships only happens when the user logs in to the frontend using OIDC.
- Assigning memberships ONLY ADDS memberships. You have to revoke them manually.
