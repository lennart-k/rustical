# Installation

## Docker

You can start a RustiCal container using the following command:

```sh
docker run \
  -p 4000:4000 \
  -v YOUR_DATA_DIR:/var/lib/rustical/ \
  -v OPTIONAL_YOUR_CONFIG_TOML:/etc/rustical/config.toml \ # (1)!
  -e RUSTICAL_CONFIG_OPTION="asd" \  # (2)!
  ghcr.io/lennart-k/rustical
```

1. Mount config file
2. Alternatively specify configuration using environment variables

!!! info
    Note that you are expected to run RustiCal behind a reverse proxy with HTTPS. (The frontend will only work on non-localhost addresses with https) and clients like Apple Calendar also expect HTTPS.

## User management

In case you already have an OIDC server set up, see [here](setup/oidc.md) how to set up OIDC login and maybe skip this section.
Otherwise you will have to use the `rustical principals` command.
In Docker you can run this with

```sh
docker run --rm -it -v YOUR_DATA_DIR:/var/lib/rustical/ ghcr.io/lennart-k/rustical rustical principals
```

This is also the place to set up **groups**.
Groups and rooms are also just principals and you can specify them as such using the `--principal-type` parameter.
To assign a user to a group you can use the `rustical membership` command. Being a member to a principal means that you can completely act on their behalf and see their collections.
**Note:** Many clients don't support autodiscovery of principals a user is a member of. In that case you'd have to set up multiple CalDAV profiles in your client with the respective principal URLs.

## Password vs app tokens

The password is optional (if you have configured OpenID Connect) and is only used to log in to the frontend.
Since it's sensitive information, a secure but slow hash algorithm (`argon2`) is chosen.

App tokens are used by your CalDAV/CardDAV client (which can be managed through the frontend).
I recommend to generate random app tokens for each CalDAV/CardDAV client.
Since the app tokens are random they use the faster `pbkdf2` algorithm.

## Manual

```sh
cargo install --locked --git https://github.com/lennart-k/rustical
```

## NixOS (community-maintained by [@PopeRigby](https://github.com/PopeRigby))

!!! warning
    The NixOS package is not maintained by myself but since I appreciate [@PopeRigby](https://github.com/PopeRigby)'s work on it I want to mention it.
    Since rustical's development is still quite active I **strongly** recommend installing from the `nixpkgs-unstable` branch.

In the `nixpkgs-unstable` you'll find a `rustical` package you can install.

There's also a service that has not been merged yet. If you know how to add modules from PRs in Nix
you can already install it <https://github.com/NixOS/nixpkgs/pull/424188>
and then setup rustical as a service:

```nix title="In your configuration.nix"
services.rustical = {
    enable = true;
    package = inputs.rustical.legacyPackages.${pkgs.stdenv.hostPlatform.system}.rustical;
    settings = {
        # Settings the same as in config.toml but in Nix syntax
        # http.port = 3002;
    };
};
```
