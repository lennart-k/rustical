FROM rust:1.84 AS chef
RUN apt-get update \
  && apt-get install -y musl-dev musl-tools --no-install-recommends \
  && rustup target add x86_64-unknown-linux-musl \
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
RUN cargo chef cook --release --target x86_64-unknown-linux-musl

COPY . .
RUN --mount=type=cache,target=target cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/rustical /usr/local/bin/rustical
CMD ["/usr/local/bin/rustical"]

LABEL org.opencontainers.image.authors="Lennart K github.com/lennart-k"
EXPOSE 4000
