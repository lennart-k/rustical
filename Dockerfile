FROM --platform=$BUILDPLATFORM rust:1.89-alpine AS chef

ARG TARGETPLATFORM
ARG BUILDPLATFORM

# the compiler will otherwise ask for aarch64-linux-musl-gcc
ENV CC_aarch64_unknown_linux_musl="clang"
ENV AR_aarch64_unknown_linux_musl="llvm20-ar"
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"

# Stupid workaound with tempfiles since environment variables
# from RUN commands don't persist across stages
RUN case $TARGETPLATFORM in \
  "linux/amd64") echo x86_64-unknown-linux-musl > /tmp/rust_target;; \
  "linux/arm64") echo aarch64-unknown-linux-musl > /tmp/rust_target;; \
  *) echo "Unsupported platform ${TARGETPLATFORM}"; exit 1;;  \
  esac

RUN apk add --no-cache musl-dev llvm20 clang perl pkgconf make \
  && rustup target add "$(cat /tmp/rust_target)" \
  && cargo install cargo-chef --locked \
  && rm -rf "$CARGO_HOME/registry"

WORKDIR /rustical

FROM chef AS planner
COPY . .
RUN cargo chef prepare

FROM chef AS builder
# We need to statically link C dependencies like sqlite, otherwise we get
# exec /usr/local/bin/rustical: no such file or directory
# x86_64-unknown-linux-musl does static linking by default
WORKDIR /rustical
COPY --from=planner /rustical/recipe.json recipe.json
RUN cargo chef cook --release --target "$(cat /tmp/rust_target)"

COPY . .
RUN cargo install --target "$(cat /tmp/rust_target)" --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/rustical /usr/local/bin/rustical
CMD ["/usr/local/bin/rustical"]

ENV RUSTICAL_DATA_STORE__SQLITE__DB_URL=/var/lib/rustical/db.sqlite3

LABEL org.opencontainers.image.authors="Lennart K github.com/lennart-k"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
EXPOSE 4000
