# KickHatSnare

KickHatSnare is a Linux desktop digital audio workstation organized around strict process boundaries.

## Packages

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

Add business behavior to its `core` feature first, define the message in the matching `protocol` feature, then add its thin adapter under `server/src/api`. Each feature's `mod.rs` registers only that feature's messages, so the root contract registry grows by domains rather than accumulating every endpoint. IPC methods are namespaced as `<domain>.<action>`, such as `system.ping`.

## Data flow

```text
React renderer -> Electron preload -> Electron main -> Rust server -> Rust core
```

Transport DTOs belong in `protocol/`. Business concepts and operations belong in `core/`. Renderer state belongs in Zustand stores and must not become a second source of domain truth.

## IPC Contracts

Rust is the source of truth for IPC. Each method implements `IpcMethod` in the matching `protocol/src` feature module and declares its parameter and result DTOs. `bun run generate:ipc` produces:

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

From `desktop/`:

```sh
bun install
bun run dev
```

Run all checks from the repository root:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd desktop && bun run check && bun run build
```

Set `KICKHATSNARE_SERVER_PATH` to run Electron against a custom server binary.

## Packaging

`bun run build` compiles the Linux Rust server in release mode and stages it alongside the Electron output. `bun run package` creates an AppImage containing the server at `resources/bin`; Electron starts that binary during application startup and stops it before quitting. Use `bun run package:dir` for an unpacked Linux build.
