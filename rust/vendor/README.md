# vendor/

Patched crate copies, used through `[patch.crates-io]` because the published crate still has the bug.

## wreq (6.0.0-rc.31)

`src/client/ws.rs` used to overwrite a caller's `Connection` and `Upgrade` headers with lowercase `upgrade`/`websocket`. It now fills them only when absent. Chromium sends `Connection: Upgrade`, and the gateway handshake has to match that. Drop the patch when upstream stops overriding the value.

HTTP/3, used by `crates/requiem-h3`. Nothing changes unless a client is given an HTTP/3 transport.

- `src/client/layer/h3.rs`: `Http3Transport`, `Http3Session`, and `ResBody`. The `Incoming` aliases in `client.rs`, `layer/retry.rs`, and `layer/client.rs` now name `ResBody`.
- `src/client/layer/client.rs`: `HttpClient::request` asks the transport before it touches the request. `ClientBuilder::http3_transport` sets it. Not used for proxies, `CONNECT`, or requests pinned to HTTP/1.
- `src/tls/conn.rs` and `src/tls.rs`: `tls::quic_ssl` returns the BoringSSL configuration the TCP connections use, with ALPN and ALPS `h3`.

`tls::QuicSessionStore` saves each ticket with the server's transport parameters. A later connection resumes with one and enables early data. Needs `foreign-types` in `vendor/wreq/Cargo.toml`.

`tls::seen_intermediates` remembers the non-root certificates of every verified TCP handshake. `quic_ssl` verifies against the webpki roots plus those, because a client that sends `trust_anchors` gets leaf-only chains from Google. The HTTP/3 transport saves them between runs.

## quinn-proto (0.11.16)

Chromium's first flight: `MIN_INITIAL_SIZE` and `INITIAL_MTU` are 1250, packet numbers start at 1 and run as one sequence, the second client Initial packet gets a two-byte packet number, and `connection/first_flight.rs` lays the ClientHello across the first two Initial packets. Used by `crates/requiem-h3`. Not a workspace member, so its unit tests do not run from here.

## ASIOSDK (Steinberg ASIO SDK 2.3.4)

Not a crate. `vendor/ASIOSDK/` is fetched from `download.steinberg.net` and gitignored. `asio-sys` reads it when the `asio` feature is on: it compiles `host/` and `common/` into `libasio.a` and bindgens the headers from `CPAL_ASIO_DIR`.

`.cargo/config.toml` always sets `CPAL_ASIO_DIR` to `vendor/ASIOSDK`. That stops `asio-sys` from downloading the SDK itself, so the directory has to exist before an `asio` build:

```
cargo requiem asio
```

`build`, `run`, `test`, and `cargo` fetch it on Windows when `host/` and `common/` are missing. The zip is `ASIO-SDK_2.3.4_2025-10-15.zip`, checked against `d5ebf0c20dd2c5f43771fd0c1418f4b361bf52434ee670097cfa6b3a335e2eca`, then the directory that contains `host/` and `common/` is installed at `vendor/ASIOSDK`.
