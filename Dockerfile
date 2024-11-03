FROM rust:1.82-alpine AS builder
# We need to statically link C dependencies like sqlite, otherwise we get
# exec /usr/local/bin/rustical: no such file or directory
# x86_64-unknown-linux-musl does static linking by default
RUN apk add --no-progress build-base; \
    rustup target add x86_64-unknown-linux-musl
WORKDIR /rustical
COPY . .
RUN --mount=type=cache,target=target cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/rustical /usr/local/bin/rustical
CMD ["/usr/local/bin/rustical"]

LABEL org.opencontainers.image.authors="Lennart K github.com/lennart-k"
EXPOSE 4000
