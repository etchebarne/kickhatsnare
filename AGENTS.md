# Implementation Guide

## Repository Structure

- `core/`: Rust library containing all application and domain logic. It has no knowledge of Electron or IPC.
- `protocol/`: Rust-owned IPC DTOs, method definitions, protocol version, TypeScript generation, and JSON Schemas.
- `server/`: Thin Rust binary that translates newline-delimited JSON messages into core commands.
- `desktop/`: Electron shell and React renderer. The main process owns the server lifecycle; the renderer only uses the API exposed by preload.

## Feature Modules

Core and server code are organized as matching feature slices:

```text
core/src/audio/               # audio domain state and operations
core/src/workspace/           # workspace domain state and operations
core/src/system/              # application-level operations
protocol/src/audio/           # audio IPC messages and local registry
protocol/src/workspace/       # workspace IPC messages and local registry
protocol/src/system/ping.rs   # one file per command/query/event
server/src/api/audio/         # audio endpoint adapters
server/src/api/workspace/     # workspace endpoint adapters
server/src/api/system/ping.rs # one adapter per protocol message
```

Add business behavior to its `core` feature first, define the message in the matching `protocol` feature, then add its thin adapter under `server/src/api`. Each feature's `mod.rs` registers only that feature's messages, so the root contract registry grows by domains rather than accumulating every endpoint.

Namespace IPC methods as `<domain>.<action>`, such as `system.ping`.

## Architectural Boundaries

```text
React renderer -> Electron preload -> Electron main -> Rust server -> Rust core
```

- Put transport DTOs in `protocol/`.
- Put business concepts and operations in `core/`.
- Put renderer state in Zustand stores, but do not make it a second source of domain truth.
- Keep the server as a thin adapter between IPC messages and core commands.

## IPC Contracts

Rust is the source of truth for IPC. Each method implements `IpcMethod` in the matching `protocol/src` feature module and declares its parameter and result DTOs.

Run `bun run generate:ipc` from `desktop/` to produce:

```text
desktop/src/shared/generated/ipc.ts
desktop/src/shared/generated/ipc.schema.json
```

The generated TypeScript method map enforces request and response types at compile time. Electron uses the generated JSON Schemas with Ajv to validate parameters before sending them and results before exposing them to the renderer. Every envelope carries `PROTOCOL_VERSION`, and mismatched clients and servers reject each other.

After adding or changing an endpoint, run:

```sh
cd desktop
bun run generate:ipc
bun run check
```

Do not edit generated contract files. `bun run check:ipc` fails when committed output is stale, while development and production builds regenerate it automatically.

## Development

Run the desktop application from `desktop/`:

```sh
bun install
bun run dev
```

Set `KICKHATSNARE_SERVER_PATH` to run Electron against a custom server binary.

## Verification

Run all checks from the repository root:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd desktop && bun run check && bun run build
```

## Packaging

`bun run build` compiles the Linux Rust server in release mode and stages it alongside the Electron output. `bun run package` creates an AppImage containing the server at `resources/bin`; Electron starts that binary during application startup and stops it before quitting. Use `bun run package:dir` for an unpacked Linux build.
