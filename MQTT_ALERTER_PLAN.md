# MQTT Alerter Plan

## Minimum Files To Modify

- `client/core/rs/src/entities/alerter.rs`
  - Add a new `AlerterEndpoint::Mqtt` variant and `MqttAlerterEndpoint` config struct.
  - Define defaults (topic `komodo/events`, QoS default) and optional auth/client fields.
  - Keeps API/resource model aligned for backend + UI.

- `client/core/ts/src/types.ts`
  - Add generated-type equivalents for the new Rust endpoint variant and params.
  - Ensures UI can type-check new MQTT config fields.

- `bin/core/src/alert/mod.rs`
  - Register `mod mqtt;`.
  - Add endpoint dispatch arm for `AlerterEndpoint::Mqtt`.

- `bin/core/src/alert/mqtt.rs` (new)
  - Implement MQTT transport: connect, serialize existing `Alert` JSON unchanged, publish, and return success/failure.

- `Cargo.toml` and `bin/core/Cargo.toml`
  - Add MQTT client crate dependency needed by core alert transport.

- `ui/src/resources/alerter/config/endpoint.tsx`
  - Add `MQTT` to endpoint selector.
  - Render MQTT-specific config fields: broker URL, topic, username, password, client ID, QoS, retain.

- `docsite/docs/resources.md`
  - Small docs update to include MQTT as a native Alerter destination.

## Recommended Rust MQTT Library

Use `rumqttc`.

Why:
- Pure Rust + Tokio-native async flow (fits Komodo core runtime).
- Lightweight for one-shot publish operations.
- Actively used in Rust MQTT integrations.
- Supports auth, QoS levels, retain flag, and broker URL parsing patterns we need.
