# KickHatSnare

KickHatSnare is a Linux desktop digital audio workstation built with Rust, Electron, and React.

The interface runs as a familiar desktop application while audio and application logic stay in a native Rust backend. A typed IPC contract connects the two sides, keeping the experience responsive and the codebase reliable as the project grows.

## Technology

- Rust for the audio engine and application logic
- Electron and React for the Linux desktop interface
- TypeScript for a type-safe frontend
- Bun for frontend tooling and packaging

## Run Locally

Install [Rust](https://www.rust-lang.org/tools/install) and [Bun](https://bun.sh/), then run:

```sh
cd desktop
bun install
bun run dev
```

## Build

Create a production build from `desktop/`:

```sh
bun run build
```

Create a Linux AppImage:

```sh
bun run package
```

Use `bun run package:dir` instead for an unpacked Linux build.

## Contributing

See [AGENTS.md](AGENTS.md) for the architecture, implementation conventions, and project checks.
