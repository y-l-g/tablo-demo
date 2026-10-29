# syntax=docker/dockerfile:1

FROM rust:1.99-bookworm AS builder

# The CLI bundles the assets the panel loads at runtime. It must come from the
# same Topcoat commit as the `[patch.crates-io]` section of Cargo.toml, or the
# bundle it writes is unreadable.
ARG TOPCOAT_REV=d84362b72bc1034a8342e00c1cdf1a36d68dd9cf

WORKDIR /src/tablo-demo
COPY . .

# The Topcoat CLI builds the app and writes the asset bundle next to the
# executable, which is where the panel looks for it.
RUN cargo install --git https://github.com/tokio-rs/topcoat --rev "${TOPCOAT_REV}" topcoat-cli --locked
RUN topcoat asset bundle --release

# Distroless keeps the runtime to the C library the binary needs.
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /src/tablo-demo/target/release/tablo-demo /app/tablo-demo
COPY --from=builder /src/tablo-demo/target/release/assets /app/assets

ENV HOST=0.0.0.0 \
    PORT=3000 \
    TABLO_DEMO_UPLOAD_DIR=/data/uploads \
    TABLO_DEMO_BASE_URL=https://demo.tablo.y-l.fr

EXPOSE 3000

USER nonroot:nonroot
WORKDIR /app

ENTRYPOINT ["/app/tablo-demo"]
