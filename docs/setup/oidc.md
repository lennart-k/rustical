# OpenID Connect

You can set up RustiCal with an OpenID Connect identity provider

## Example: Authelia

```toml title="RustiCal configuration"
[frontend.oidc]
name = "Authelia"
issuer = "https://auth.example.com"
client_id = "rustical"
client_secret = "secret..."
claim_userid = "preferred_username"  # (1)!
scopes = ["openid", "profile", "groups"]
require_group = "app/rustical"  # (2)!
allow_sign_up = false
```

1. Can be either `preferred_username` or `sub`
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
