# KickHatSnare

KickHatSnare is an intuitive Linux digital audio workstation.

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
