# requiem-plugin-api

WIT package `requiem:plugin@1.0.0`, in `wit/`. No Rust. `requiem-plugin-host` and `requiem-plugin` generate their bindings from these files.

`1.0.0` is unreleased and may change until the first plugin ships. After that, a minor version only adds interfaces, functions, and resource methods. It does not change an existing record, variant, enum, flags, or function signature.

Data that can grow is a `resource` with getters. Open sets (permissions, event kinds, error codes, setting types) are strings. The host only calls the guest hooks a plugin's manifest declares.

Host imports: `log`, `storage`, `timers`, `client`, `actions`, `navigation`, `discord`, `presence`, `voice`, `notify`, `tweaks`, `files`, `net`, `overlay`, `composer`, `ui`, `slots`, `registry`.

Guest exports: `lifecycle`, `outgoing`, `incoming`, `events`, `interactions`, `views`, `providers`. Shared types live in `types`, `model`, and `drafts`.
