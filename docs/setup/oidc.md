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
