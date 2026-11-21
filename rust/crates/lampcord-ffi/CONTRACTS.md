# Lampcord FFI contracts

`lampcord-ffi` is the boundary between the Rust core and non-Rust frontends
(Kotlin/Compose today, Swift later). The contracts below are stable enough for
a frontend to code against while the core evolves.

## Snapshots

* `ClientSnapshot.schema_version` is the compatibility key. The current value
  is `SNAPSHOT_SCHEMA_VERSION`. Consumers must check it before reading newer
  fields; unknown fields are additive, known fields are never repurposed.
* `ClientSnapshot.seq` is monotonic per client process. Frontends can ignore
  snapshots with a sequence they have already applied.
* Projections are bounded: messages are capped at 200 per selected channel,
  and members/presences/read states at 200 each. Use paginated commands for
  complete lists.
* Snapshots are level-triggered. Re-applying a snapshot with the same `seq` is
  safe and should be idempotent.

## Events

* `drain_events()` returns `ClientEventEnvelope` values ordered by a
  process-wide monotonic `sequence`.
* `emitted_at_ms` is a UTC Unix timestamp for observability only; do not use it
  for ordering.
* The queue is bounded (512). When full, the oldest 64 events are dropped, so
  consumers must treat events as hints and reconcile from snapshots.
* At most one pending `SnapshotUpdated` is kept. Typing and presence events
  are last-write-wins per user/channel while queued.

## Errors

* Commands return `Result<(), FfiClientError>`; UniFFI exposes this as
  `FfiClientException`.
* `FfiErrorKind` is a stable enum for UI decisions (for example
  `CaptchaRequired`, `MfaRequired`, `RateLimited`). Message strings are for
  humans only and must not be parsed.
* Asynchronous failures are surfaced as `ClientEvent::CommandFailed` with the
  same error kinds.

## Auth

* Password, MFA, SMS, forgot-password and remote-auth flows are commands, not
  blocking return values. The outcome arrives as `AuthChanged` /
  `RemoteAuthChanged` and is mirrored in `ClientSnapshot.auth`.
* A captcha challenge includes `sitekey`, `service`, `rqdata`, `session_id`
  and `invisible`; pass the solved key back through `login_password`.
* Token persistence goes through `PlatformSecretStore` when injected by the
  platform. Without an injected store the runtime uses a plain file fallback
  intended for development only.

## Gateway dispatch

Typed UI events are derived from gateway payloads in the FFI runtime:
message edits/deletes, reactions, typing, presence and notifications. Raw
payloads never cross the FFI boundary. Partial `MESSAGE_UPDATE` payloads are
overlaid onto the state cache before emission, so `MessageUpdated` snapshots
remain complete.
