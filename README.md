# PantryPal

An offline-first Windows desktop meal planner for a Canadian household. Built with Rust, Tauri 2, React, TypeScript and, beginning in Phase 1, SQLite. See [DESIGN.md](DESIGN.md), [PLAN.md](PLAN.md) and [AGENTS.md](AGENTS.md) before changing behavior.

Phases 0–4 supply the local SQLite foundation, ingredient editing, exact measurements, manual meal library and Monday-start weekly planner. Week supports repeated meals, independent fractional servings, dated/unscheduled occurrences, keyboard ordering and explicit planned/cooked/skipped actions. Cooking history supports backfill, confirmed date correction and undo, with last eaten derived only from actual records. Meals supports original amount wording, unknown quantities, six custom field kinds, typed filters and safe retirement. Ingredients supports aliases, categories, shopping sections, season months, notes, preferred units and optional conversion rules. Settings saves measurement preferences and converts temperatures. The planner opens Shopping with the selected week; grocery generation, imports and backups remain in later phases.

## Windows prerequisites

- Windows 10/11 x64 and the [Microsoft Edge WebView2 Runtime](https://v2.tauri.app/start/prerequisites/#webview2).
- Visual Studio 2022 Build Tools (or Community), with **Desktop development with C++** and a Windows SDK.
- Rust through [rustup](https://rustup.rs/), using the MSVC host (`x86_64-pc-windows-msvc`). `rust-toolchain.toml` pins Rust 1.98.1, rustfmt and Clippy; rustup downloads them on the first Cargo command.
- Node.js 24 and npm 11. Run commands from the repository root in PowerShell. If a build cannot locate MSVC or the SDK, use Developer PowerShell for Visual Studio.

Official setup reference: [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).

## Develop

```powershell
npm ci
. ./scripts/Enter-DevEnvironment.ps1
npm run tauri dev
```

This starts Vite on `http://127.0.0.1:1420` and opens the native Windows app. The first Rust build can take several minutes. Port 1420 must be free; Vite fails instead of selecting a different port. Close the app and stop the dev command with Ctrl+C when finished.

The environment helper locates Visual Studio through `vswhere` and imports the x64 compiler and SDK environment. Merely having `cl.exe` on PATH is insufficient to compile bundled SQLite: it also needs the SDK include/library paths. Run the helper before Cargo checks or native builds in an ordinary PowerShell terminal; a configured Developer PowerShell already supplies those paths.

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

`npm run check` runs the four frontend checks. `npm run format` and `cargo fmt --all` apply formatting. Vitest + Testing Library cover navigation, keyboard/form flows, weekly planning and cooking history, meal and field editing, filters and invalid-filter recovery, ingredient/rule/vocabulary editing, preferences, conversion displays, typed IPC payloads/errors and capability regression checks. Rust tests cover exact parsing/arithmetic, compatible unit pairs, ambiguous quantities, temperatures, ingredient rules, all custom field types, retirement, history-derived filters, duplicate occurrences, date boundaries, action retries, migrations and transactional persistence. IPC integration tests exercise actual command serialization, capabilities and SQLite with only the OS runtime mocked. `crud_survives_real_process_restart` launches its normally ignored child harness twice, verifying ingredient edits, rules, preferences and a meal with custom fields, repeated plans and cooking history through a genuine process exit/reopen, combined filters, another edit and a safe original-action retry. Simulated origin-denial and static security checks do not replace the Phase 6 real external-WebView isolation test.

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
- `src-tauri/src/persistence` and `src-tauri/migrations`: SQLx repositories and numbered embedded migrations in per-user app data.
- `src-tauri/src/importers`: reserved for reviewed draft-only import adapters.

Only the local `main` window can invoke the explicitly granted identity, storage-status, measurement, catalog/meal/field and planner/history operations. There are no generic filesystem, SQL, shell, HTTP, clipboard or remote-origin permissions. New commands must be included in the build-time command manifest and granted individually. See [ADR 0001](docs/adr/0001-foundation.md), [ADR 0002](docs/adr/0002-local-storage.md), [ADR 0003](docs/adr/0003-exact-measurements.md), [ADR 0004](docs/adr/0004-meal-library.md), [ADR 0005](docs/adr/0005-week-planner-history.md) and [Tauri capabilities](https://v2.tauri.app/security/capabilities/).

## Local data and fixtures

The desktop app creates `%APPDATA%\ca.pantrypal.desktop\pantrypal.sqlite3` using Tauri's app-data resolver. Foreign keys and WAL are enabled. Seven additive migrations initialize schema, seed, preferences, original amount text and planner/history metadata exactly once; reopening preserves user edits and retirement. SQLite's `-wal` and `-shm` files may be present during use. This directory is outside the repository and contains personal data; do not commit it or delete it to repair a migration error.

Plan/history writes use revisions and durable action receipts, so a retry cannot repeat an add or cooking action, including after restart or undo. Cancelled cooking records and date-change audit data are retained. Removing a plan entry preserves its actual cooking history. Dates use the computer's local calendar; future cooking is rejected, and scheduled dates never imply actual cooking. Shopping generation remains Phase 5 and does not run from the current handoff screen.

If storage is unavailable or incompatible, Settings and Ingredients display an error and the app never silently resets data. Backups/export/restore are Phase 7 work. To make a manual safety copy before then, close all PantryPal processes and copy the database directory; do not copy only the main file while the app is writing. Website session data must not be included in future recipe backups.

`tests/fixtures/demo-meal.json` provides a deterministic meal with an exact fractional amount, raw wording and an unknown/to-taste line. Rust and frontend tests use it without seeding production meals. Tests create isolated temporary databases and never use the household app-data database.
