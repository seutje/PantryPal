# PantryPal

An offline-first Windows desktop meal planner for a Canadian household. Built with Rust, Tauri 2, React, TypeScript and, beginning in Phase 1, SQLite. See [DESIGN.md](DESIGN.md), [PLAN.md](PLAN.md) and [AGENTS.md](AGENTS.md) before changing behavior.

Phase 0 supplies a runnable navigation shell for Week, Meals, Shopping, Ingredients and Settings. These sections have clear placeholder states; meal storage, grocery calculations and imports are not implemented yet. No database or browser session is created by the foundation.

## Windows prerequisites

- Windows 10/11 x64 and the [Microsoft Edge WebView2 Runtime](https://v2.tauri.app/start/prerequisites/#webview2).
- Visual Studio 2022 Build Tools (or Community), with **Desktop development with C++** and a Windows SDK.
- Rust through [rustup](https://rustup.rs/), using the MSVC host (`x86_64-pc-windows-msvc`). `rust-toolchain.toml` pins Rust 1.98.1, rustfmt and Clippy; rustup downloads them on the first Cargo command.
- Node.js 24 and npm 11. Run commands from the repository root in PowerShell. If a build cannot locate MSVC or the SDK, use Developer PowerShell for Visual Studio.

Official setup reference: [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).

## Develop

```powershell
npm ci
npm run tauri dev
```

This starts Vite on `http://127.0.0.1:1420` and opens the native Windows app. The first Rust build can take several minutes. Port 1420 must be free; Vite fails instead of selecting a different port. Close the app and stop the dev command with Ctrl+C when finished.

`npm run dev` starts a browser preview only. It displays “Browser preview” and does not emulate a native connection. Core assets and fonts are bundled/local; dependency installation requires internet, while the built shell does not.

## Quality checks

```powershell
npm run format:check
npm run lint
npm run typecheck
npm test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

`npm run check` runs the four frontend checks. `npm run format` and `cargo fmt --all` apply formatting. Vitest + Testing Library cover navigation, keyboard activation/focus, IPC responses/errors and capability regression checks. Rust smoke tests cover command identity and the serialized error contract. Static security checks do not prove a remote WebView is isolated at runtime; that test belongs to the Phase 6 feasibility spike.

The Windows quality workflow in `.github/workflows/quality.yml` runs these checks and builds the debug executable. CI configuration must pass on GitHub before treating a CI run as verified.

## Build

```powershell
# Debug executable with bundled frontend; no installer.
npm run tauri build -- --debug --no-bundle
# Optimized executable without an installer.
npm run tauri build -- --no-bundle
# Release executable and an unsigned NSIS installer.
npm run tauri build
```

All builds first run TypeScript checking and Vite production compilation. With the root Cargo workspace, executables are `target/debug/pantrypal.exe` and `target/release/pantrypal.exe`; NSIS output is under `target/release/bundle/nsis/`. Tauri may download NSIS tools during the first installer build. Signing, clean-machine installation and release acceptance are Phase 8 work; a documented installer command is not a verified release.

Commit both lockfiles. Use `npm ci` and Cargo's `--locked` option for repeatable checks. Update dependencies and the pinned Rust toolchain deliberately, with the quality suite and Windows build rerun.

## Boundaries

- `src/app`: navigation shell and Query provider.
- `src/features`: section-specific presentation.
- `src/components`: native semantic UI primitives with shared focus styling.
- `src/lib/ipc.ts`: named typed operations and safe normalized errors.
- `src-tauri/src/commands`: narrow IPC adapters; `domain` and `services` own business rules.
- `src-tauri/src/persistence` and `src-tauri/migrations`: reserved for Phase 1 SQLite repositories and numbered migrations in per-user app data.
- `src-tauri/src/importers`: reserved for reviewed draft-only import adapters.

Only the local `main` window can invoke `get_app_info`. There are no filesystem, SQL, shell, HTTP, clipboard or remote-origin permissions. New commands must be included in the build-time command manifest and granted individually. See [ADR 0001](docs/adr/0001-foundation.md) and [Tauri capabilities](https://v2.tauri.app/security/capabilities/).
