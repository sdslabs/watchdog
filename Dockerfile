# syntax=docker/dockerfile:1
# Build on linux/amd64 for the x86_64-unknown-linux-musl target.
FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools perl make gcc file binutils \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

# Enable bundled OpenSSL only in the container's copy of the manifest.
RUN sed -i '/^\[dependencies\]/a openssl = { version = "0.10", features = ["vendored"] }' Cargo.toml
ENV CC_x86_64_unknown_linux_musl=musl-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static -C relocation-model=static"
# Keep downloaded crates and compiled dependencies across source changes.
# Copy the executable out of the cache mount for the artifact stage.
RUN --mount=type=cache,id=watchdog-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=watchdog-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=watchdog-musl-target,target=/build/target,sharing=locked \
    cargo build --release --target x86_64-unknown-linux-musl --bin watchdog \
    && install -D target/x86_64-unknown-linux-musl/release/watchdog /out/watchdog
# Fail the build if the executable needs a dynamic loader or shared libraries.
RUN file /out/watchdog \
    && readelf -l /out/watchdog > /tmp/program-headers \
    && readelf -d /out/watchdog > /tmp/dynamic-section \
    && ! grep -q INTERP /tmp/program-headers \
    && ! grep -q NEEDED /tmp/dynamic-section

# Export only the executable with Docker Buildx's local exporter.
FROM scratch AS artifact
COPY --from=builder /out/watchdog /watchdog
