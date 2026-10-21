# syntax=docker/dockerfile:1

# The demo builds Tablo from the checkout beside it (see its Cargo.toml), so the
# image clones both repositories: Tablo under the path the demo points at, and
# the demo itself.
FROM rust:1.99-bookworm AS builder

# Pinned so an image always builds against the same Tablo: the demo follows
# Tablo's master, and a newer Tablo can break it. Bump it with the demo's own
# dependency.
ARG TABLO_REF=8570f1f237fb112dc7950a227cb5306d01c47b4d
# The CLI bundles the assets the panel loads at runtime. It must match the
# Topcoat the demo depends on, or the bundle it writes is unreadable.
ARG TOPCOAT_CLI_VERSION=0.10.0

RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

RUN git init filament-topcoat/tablo \
    && cd filament-topcoat/tablo \
    && git remote add origin https://github.com/y-l-g/tablo.git \
    && git fetch --depth 1 origin "${TABLO_REF}" \
    && git checkout FETCH_HEAD

COPY . tablo-demo
WORKDIR /src/tablo-demo

# The Topcoat CLI builds the app and writes the asset bundle next to the
# executable, which is where the panel looks for it.
RUN cargo install topcoat-cli --version "${TOPCOAT_CLI_VERSION}" --locked
RUN topcoat asset bundle --release

# Distroless keeps the runtime to the C library the binary needs.
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /src/tablo-demo/target/release/tablo-demo /app/tablo-demo
COPY --from=builder /src/tablo-demo/target/release/assets /app/assets

ENV HOST=0.0.0.0 \
    PORT=3000 \
    TABLO_DEMO_UPLOAD_DIR=/data/uploads

EXPOSE 3000

USER nonroot:nonroot
WORKDIR /app

ENTRYPOINT ["/app/tablo-demo"]
