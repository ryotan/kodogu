# KoDoGu

KoDoGu (小道具, "props" in Japanese) is a toolbox of small tools I want to use. The desktop app is built with Tauri
(React + TypeScript, Rust), and the command-line tools are written in Rust.

## Tools

- Tsunagi: manages port-forwarding processes (cloud-sql-proxy, `ssh -L`, `aws ssm`, ...) in one place (in development)
- Key pairs: creation, encryption with public keys, signing with private keys (planned)
- Certificates: CA and server certificates for localhost (planned)
- Resume and work history documents (planned)

## Repository Layout

- `app/`: the desktop app ([README](./app/README.md))
- `kernel/`: Rust libraries shared by the tools
- `cli-app/`: command-line tools
- `docs/`: ADRs and development guides

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT License](./LICENSE-MIT) at your option.
