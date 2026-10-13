# PantryPal development

PantryPal uses Rust, Tauri 2, React, TypeScript and SQLite. For installation and
everyday usage, see [README.md](README.md).

Before changing product behavior or architecture, read [DESIGN.md](DESIGN.md),
[PLAN.md](PLAN.md) and [AGENTS.md](AGENTS.md). DESIGN defines product and data
rules; PLAN tracks verified work and acceptance gates; AGENTS defines repository
working practices. Architectural decisions are recorded in [docs/adr](docs/adr).
Phases 0–7 are complete; Phase 8 remains in progress with clean-machine and
production-feed update acceptance pending. Consult PLAN for current status.

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

`npm run check` runs the four frontend checks. `npm run format` and `cargo fmt --all` apply formatting. Vitest + Testing Library cover navigation, keyboard/form flows, weekly planning and cooking history, shopping edits/copying/retry, meal and field editing, filters and invalid-filter recovery, ingredient/rule/vocabulary editing, preferences, conversion displays, typed IPC payloads/errors and capability regression checks. Rust tests cover exact parsing/arithmetic, compatible unit pairs, ambiguous quantities, temperatures, ingredient rules, all custom field types, retirement, history-derived filters, duplicate occurrences, date boundaries, action retries, mixed-unit grocery generation, source preservation, shopping regeneration/legacy upgrades, migrations and transactional persistence. IPC integration tests exercise actual command serialization, capabilities and SQLite with only the OS runtime mocked. `crud_survives_real_process_restart` launches its normally ignored child harness twice, verifying ingredient edits, rules, preferences and a meal with custom fields, repeated plans, cooking history and shopping adjustments through a genuine process exit/reopen, combined filters, another edit and a safe original-action retry. Simulated origin-denial and static security checks do not replace the Phase 6 real external-WebView isolation test.

An explicit Windows clipboard acceptance check writes harmless fixture grocery text and verifies its UTF-16 paste representation (the ordinary test suite leaves the clipboard alone):

```powershell
cargo test -p pantrypal --lib windows_clipboard_roundtrip --locked -- --ignored --exact services::clipboard::windows::tests::windows_clipboard_roundtrip
```

The Windows release workflow in `.github/workflows/release.yml` runs frontend
checks, Rust formatting, Clippy and tests before building and publishing the
signed installer on version tags. Run the commands above locally for ordinary
development; there is no separate push/pull-request quality workflow.

## Build

```powershell
# Debug executable with bundled frontend; no installer.
npm run tauri build -- --debug --no-bundle
# Optimized executable without an installer.
npm run tauri build -- --no-bundle
# Release executable and an unsigned NSIS installer.
npm run tauri build
```

All builds first run TypeScript checking and Vite production compilation. With the root Cargo workspace, executables are `target/debug/pantrypal.exe` and `target/release/pantrypal.exe`; NSIS output is under `target/release/bundle/nsis/`. Tauri may download NSIS tools during the first installer build. Use `npm run release:build` for an updater-signed NSIS installer. Clean-machine installation and production-feed acceptance remain pending Phase 8 gates; a documented installer command is not a verified release.

Commit both lockfiles. Use `npm ci` and Cargo's `--locked` option for repeatable checks. Update dependencies and the pinned Rust toolchain deliberately, with the quality suite and Windows build rerun.

For release signing, key custody, version tags, automatic publication and installer
acceptance, see [Building and publishing](docs/releases.md#building-and-publishing)
and [Isolated Windows acceptance](docs/releases.md#isolated-windows-acceptance).
Updater signatures and Windows publisher signatures are distinct; Authenticode
publisher signing is currently absent. See [ADR 0011](docs/adr/0011-signed-windows-updates.md).

## Boundaries

- `src/app`: navigation shell and Query provider.
- `src/features`: section-specific presentation.
- `src/components`: native semantic UI primitives with shared focus styling.
- `src/lib/ipc.ts`: named typed operations and safe normalized errors.
- `src-tauri/src/commands`: narrow IPC adapters; `domain` and `services` own business rules.
- `src-tauri/src/persistence` and `src-tauri/migrations`: SQLx repositories and numbered embedded migrations in per-user app data.
- `src-tauri/src/importers`: bounded authorized website/JSON-LD/manual draft adapters and URL validation.

## Recipe browser security and verification

The owner confirmed agreements for Budget Bytes, Allrecipes, Spend With Pennies,
RecipeTin Eats and NYT Cooking. All five supplied URLs
passed native recipe-preview checks, including denial of remote capture commands
and zero meal writes before confirmation. Hidden/gated content is rejected even
if recipe metadata exists; no login, paywall or anti-bot bypass is provided.
Future site layout changes or cross-origin login prompts may require manual
entry. Actual subscriber login/MFA was not verified. Cook Smarts remains manual;
no authorized automated integration was established. See
[ADR 0008](docs/adr/0008-authorized-website-adapters.md) for the adapter decision
and [ADR 0007](docs/adr/0007-reviewed-imports-and-browser.md) for session isolation.

Explicit Windows acceptance harnesses (each uses isolated fixture storage, never
the household database):

```powershell
. ./scripts/Enter-DevEnvironment.ps1
cargo run -p pantrypal --example browser_spike --locked -- write
cargo run -p pantrypal --example browser_spike --locked -- read
cargo run -p pantrypal --example browser_spike --locked -- cleared
# Live authorized recipe previews and remote capture denial, no meal saves:
cargo run -p pantrypal --example adapter_sites --locked
# Optional live compatibility visits, no recipe extraction:
cargo run -p pantrypal --example recipe_sites --locked
```

The three-process owned fixture verifies native evaluation, ACL rejection of
remote application commands, reviewed local-IPC save, meal persistence after
process restart, login-cookie/local-storage retention and the actual session reset
command. Loopback HTTP is permitted only in this isolated test fixture. Results
and disposable profiles are ignored under `.verification/`. The live site check
`recipe_sites` records only load state and page title. `adapter_sites` exercises
production open/capture commands, records only recipe counts and pass/fail,
and verifies previewing leaves its isolated database empty. Neither logs
credentials or saves publisher recipe text in the repository. Saved adapter
fixtures use original synthetic recipe text and inspected publisher structures.

Only the local `main` window can invoke the explicitly granted identity, storage-status, measurement, catalog/meal/field and planner/history, shopping, reviewed backup and preference operations. There are no generic filesystem, SQL, shell, HTTP, clipboard or remote-origin permissions. New commands must be included in the build-time command manifest and granted individually. See [ADR 0001](docs/adr/0001-foundation.md), [ADR 0002](docs/adr/0002-local-storage.md), [ADR 0003](docs/adr/0003-exact-measurements.md), [ADR 0004](docs/adr/0004-meal-library.md), [ADR 0005](docs/adr/0005-week-planner-history.md), [ADR 0006](docs/adr/0006-shopping-generation.md) and [Tauri capabilities](https://v2.tauri.app/security/capabilities/).

## Backup and offline acceptance

User-facing export, restore, format limits and migration recovery instructions
are in [README.md](README.md#backups-and-everyday-settings).

Offline Windows acceptance, using bundled assets and isolated fixture storage:

```powershell
npm run build
. ./scripts/Enter-DevEnvironment.ps1
cargo run -p pantrypal --example phase7_spike --features tauri/custom-protocol --locked
```

This verifies native planning/shopping, Settings labels/navigation, file review,
explicit confirmation, export/restore, recovery and reopening. The shell CSP
blocks remote fetches; only deliberate recipe browsing needs internet. See
[ADR 0010](docs/adr/0010-backups-and-everyday-settings.md) and the
[Phase 7 audit](docs/phase7-security-audit.md) for format and verification limits.

## Local data and fixtures

The desktop app creates `%APPDATA%\ca.pantrypal.desktop\pantrypal.sqlite3` using Tauri's app-data resolver. Foreign keys and WAL are enabled. Eight additive migrations initialize schema, seed, preferences, original amount text and planner/history metadata exactly once; reopening preserves user edits and retirement. SQLite's `-wal` and `-shm` files may be present during use. This directory is outside the repository and contains personal data; do not commit it or delete it to repair a migration error.

Plan/history writes use revisions and durable action receipts, so a retry cannot repeat an add or cooking action, including after restart or undo. Cancelled cooking records and date-change audit data are retained. Removing a plan entry preserves its actual cooking history. Dates use the computer's local calendar; future cooking is rejected, and scheduled dates never imply actual cooking. Shopping generates quantities through the week-aware handoff.

Household database backup/recovery instructions are in
[README.md](README.md#local-data). Keep website profiles separate from household
backups. Never use the production app-data database for tests.

`tests/fixtures/demo-meal.json` provides a deterministic meal with an exact fractional amount, raw wording and an unknown/to-taste line. Rust and frontend tests use it without seeding production meals. Tests create isolated temporary databases and never use the household app-data database.
