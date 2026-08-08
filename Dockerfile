# Build from source in a container.
#
# Base image choice: rust:1-slim (Debian) instead of Alpine/musl. The bot
# would likely build fine on musl (rusqlite is bundled, TLS is rustls), but
# the glibc toolchain needs no extra target setup and matches CI, so it is
# the more reliable option. rust:1-slim already ships gcc + libc6-dev, which
# is all the bundled SQLite needs.

FROM rust:1-slim AS builder

WORKDIR /build

# Warm the dependency cache: manifests + a dummy main first, so this layer
# is reused as long as Cargo.toml/Cargo.lock do not change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# Now the real sources. Drop the dummy build artifacts of the bin crate so
# cargo rebuilds it (COPY keeps host mtimes, which can be older than the
# dummy build).
COPY src ./src
RUN rm -f target/release/deps/foukobot* target/release/foukobot \
    && cargo build --release --locked

# Runtime: minimal Debian. ca-certificates is required by rustls.
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user. /data holds the SQLite DB and logs (see docker-compose.yml).
RUN useradd --system --uid 10001 --create-home --home-dir /home/fouko fouko \
    && mkdir -p /data \
    && chown fouko:fouko /data

WORKDIR /app
COPY --from=builder /build/target/release/foukobot /app/foukobot

USER fouko
VOLUME ["/data"]

ENTRYPOINT ["/app/foukobot"]
