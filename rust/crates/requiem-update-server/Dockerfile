# The update server as a container. Build from the repository root (the workspace is the context):
#   docker build -f crates/requiem-update-server/Dockerfile -t update-server .
# The official public keys are compiled in (update_core::official_keys), so a key rotation means a new image.

FROM rust:bookworm AS build
WORKDIR /src
COPY . .
# .cargo/config.toml pins the target dir for a Windows machine; the environment wins over it.
ENV CARGO_TARGET_DIR=/src/target
# rust-toolchain.toml picks the toolchain the repository builds with.
RUN cargo build --release --locked -p requiem-update-server

FROM debian:bookworm-slim
# CA certificates: the server fetches GitHub's token signing keys over https.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home updates \
    && mkdir /data && chown updates /data
COPY --from=build /src/target/release/requiem-update-server /usr/local/bin/update-server
USER updates
VOLUME /data
ENV UPDATE_SERVER_ROOT=/data \
    UPDATE_SERVER_LISTEN=0.0.0.0:8080
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD ["/usr/local/bin/update-server", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/update-server"]
