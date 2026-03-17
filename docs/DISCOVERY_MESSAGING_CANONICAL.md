# Canonical Messaging Types

This discovery note records the canonical channel messaging contract that every adapter and pack in Greentic shares.

> [!NOTE]
> This page documents shared messaging envelope types. JSON-over-HTTP usage described below is compatibility-oriented wire behavior. New core runtime contracts should follow canonical v0.6 CBOR-first guidance in [`vision/canonical-v0.6.md`](vision/canonical-v0.6.md).

## ChannelMessageEnvelope (universal message)
Every inbound or outbound adapter exchanges a `ChannelMessageEnvelope` so the core runtime can treat all channels the same. The envelope packs the following cross-cutting context:

- `id`: stable identifier for deduplication and tracing.
- `tenant`: the `TenantCtx` that scopes the message.
- `channel`: abstract channel identifier (webhook, telegram, etc.).
- `session_id`: the conversation identifier used to look up flows or sessions.
- `reply_scope`: optional `ReplyScope` used for wait/continue/resume logic (see below).
- `user_id`: optional actor identifier supplied by inbound providers and echoed outbound.
- `correlation_id`: optional outbound identifier that can round-trip transactions or diagnostics.
- `text`, `attachments`: the visible payload delivered by the adapter (attachments are defined separately below).
- `metadata`: `MessageMetadata` map that carries adapter- or flow-specific hints (routing keys, security tags, telemetry knobs, etc.).

### Inbound vs outbound usage
- **Inbound adapters** build the envelope from channel events and hand it to the runtime. They fill `tenant`, `channel`, `session_id`, `reply_scope`, `user_id`, `text`, `attachments`, and any metadata required to route or influence flows. `id` and `correlation_id` typically come from the platform event.
- **Outbound adapters** consume the same envelope after the runtime resolves a response: they read `text`, `attachments`, `metadata`, and the optional correlation IDs before sending downstream. When adapters are triggered by resumable replies, they honor the `reply_scope` so channel threads remain consistent.

## Attachments and MessageMetadata
- `Attachment` is a lightweight typed reference (`mime_type`, `url`, optional `name`, `size_bytes`) so rich media can travel through flow boundaries without embedding blob data.
- `MessageMetadata` aliases `BTreeMap<String, String>` so adapters and packs can encode arbitrary, ordered key/value hints (for example, conversation language, channel tenant, or debugging knobs). The metadata map is always initialized to the empty map, so adapters never encounter a missing field.

## Session identity helpers
- `SessionKey` is a tiny string wrapper around the session identifier that flows through runtime state stores, telemetry, and session routers. It implements `Display`, `serde`, and optional `uuid` generation (`SessionKey::generate`) but otherwise behaves like a semantic string.
- `canonical_session_key(tenant, provider, anchor, user)` builds the deterministic key `tenant:provider:anchor:user`. Most adapters and connectors should call this helper with the provider name and the conversation anchor (default `conversation`) plus the user (default `user`). By standardizing the anchor/user defaults, pause/resume logic stays compatible even when not every adapter emits both values.

## ReplyScope semantics
- `ReplyScope` defines the stable points where replies and resumptions are anchored. It contains `conversation` plus optional `thread`, `reply_to`, and `correlation` fields so adapters can precisely target the right sub-conversation (thread or reply-to) when continuing a session.
- A deterministic `scope_hash` is derived by concatenating the normalized field values and hashing with SHA-256. This hash makes it easy to compare scopes or look up continuations without exposing mutable fields.
- `WaitScope` is a legacy alias for `ReplyScope` so older code paths continue to compile.
- `reply_scope` lives on the envelope so the runtime can emit it for pause points (e.g., waiting for a follow-up message) and outbound adapters can resume the correct thread.

## Format note
All of these structs derive `serde::{Serialize, Deserialize}` and today the adapters exchange them as JSON over HTTP/webhooks. There is a roadmap item to offer CBOR interchange soon for bandwidth-sensitive channels, so document consumers should treat JSON as the current wire format and plan for a CBOR variant in future releases.
