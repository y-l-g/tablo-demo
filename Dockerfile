# syntax=docker/dockerfile:1

# The demo builds Tablo from the checkout beside it (see its Cargo.toml), so the
# image clones both repositories: Tablo under the path the demo points at, and
# the demo itself.
FROM rust:1.99-bookworm AS builder

# Pinned so an image always builds against the same Tablo. Bump it with the
# demo's own dependency.
ARG TABLO_REF=e5d5c5ea0f5c161b6b2c8d3daf7e353602c1113a
# The CLI bundles the assets the panel loads at runtime. It must match the
# Topcoat the demo depends on, or the bundle it writes is unreadable.
ARG TOPCOAT_CLI_VERSION=0.10.0

RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

RUN git clone --depth 1 --branch "${TABLO_REF}" https://github.com/y-l-g/tablo.git filament-topcoat/tablo

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
