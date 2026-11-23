# KoDoGu

KoDoGu (小道具, "props" in Japanese) is a toolbox of small tools I want to use. The desktop app is built with Tauri
(React + TypeScript, Rust).

## Tools

- Tsunagi: manages port-forwarding processes (cloud-sql-proxy, `ssh -L`, `aws ssm`, ...) in one place (in development)
- Key pairs: creation, encryption with public keys, signing with private keys (planned)
- Certificates: CA and server certificates for localhost (planned)
- Resume and work history documents (planned)

## Repository Layout

```text
kernel/<function>/       functions that depend on no application and hold no instance; Rust only
apps/<application>/      the applications: the resident, the desktop app (apps/desktop/), the CLI
tools/<tool>/            everything of one tool
tooling/<package>/       shared settings of development tools and the checks of the dependency directions
```

The desktop app is in [`apps/desktop/`](./apps/desktop/README.md).

## Development

- Desktop app: see [apps/desktop/README.md](./apps/desktop/README.md)

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT License](./LICENSE-MIT) at your option.
