# KoDoGu

KoDoGu ("props" in Japanese) is an app that collects tools I want to use. Built with Tauri (React, TypeScript + Rust)
for GUI apps and Rust for CLI apps, it provides lightweight and fast operation.

## Main Features

- Creation of key pairs, encryption with public keys, signing with private keys (planned)
- Creation of CA certificates for localhost, server certificates (planned)
- Creation of resume and work history documents (planned)

## インストール

## Installation

```bash
# Clone the repository
git clone https://github.com/ryotan/kodogu.git
cd kodogu/app

# Install dependencies
bun install

# Run the application
bun run tauri:dev
```

## System Requirements

- System that meets [Tauri requirements](https://tauri.app/start/prerequisites/)
- Windows 10/11, macOS 11 or later, or Linux

## Information for Developers

For information on how to develop in this project, please see [CONTRIBUTING.md](./CONTRIBUTING.md).

## Project Structure

```
kodogu/
├── app/               # Tauri application
│   ├── src/           # Frontend source code
│   ├── src-tauri/     # Rust backend source code
│   └── dist/          # Built frontend
├── cli-app/           # Tauri application
└── docs/              # Project documentation
```

## License

[Apache-2.0](./LICENSE)
