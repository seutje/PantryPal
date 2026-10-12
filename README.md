# PantryPal

An offline-first Windows desktop meal planner for a Canadian household. Built with Rust, Tauri 2, React, TypeScript and, beginning in Phase 1, SQLite. See [DESIGN.md](DESIGN.md), [PLAN.md](PLAN.md) and [AGENTS.md](AGENTS.md) before changing behavior.

Phases 0–7 supply the local SQLite foundation, ingredient editing, exact measurements, manual meal library and Monday-start weekly planner. Week supports repeated meals, independent fractional servings, dated/unscheduled occurrences, keyboard ordering and explicit planned/cooked/skipped actions. Cooking history supports backfill, confirmed date correction and undo, with last eaten derived only from actual records. Meals supports original amount wording, unknown quantities, six custom field kinds, typed filters and safe retirement. Ingredients supports aliases, categories, shopping sections, season months, notes, preferred units and optional conversion rules. Settings saves measurement preferences and converts temperatures. Shopping generates exact grouped quantities for the selected week, keeps incompatible and unknown lines visible, preserves manual items, overrides, checkmarks and exclusions on refresh, and copies plain text to the Windows clipboard. Stale source data requires explicit refresh before copying. Imports support authorized website capture and JSON-LD/manual entry with review before saving. Settings now provides reviewed JSON backup export/restore, aisle ordering and opt-in clipboard URL prefill.

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
- `src-tauri/src/importers`: bounded authorized website/JSON-LD/manual draft adapters and URL validation.

## Adding meals to a day

In Week, each day has an **Add meal** button. It opens the Meals search and
filters in a modal; click a meal card or activate it with the keyboard to add it
to that day. The picker closes after saving. Each selection creates a separate
planned occurrence, so a day can contain multiple meals or repeated meals.
Servings start at the recipe's base servings and can be changed with **Edit
servings** on the planned meal. This changes only that occurrence. Expand
**Ingredients for … servings** to see its scaled amounts; the saved recipe and
other occurrences retain their servings. Refresh Shopping to use the new totals.
**Edit occurrence** also allows changing the day. Escape or Cancel closes the picker without adding a meal; failed
saves retain the search and offer the same-action retry.

## Shopping quantity displays

Shopping quantities round upward to readable measures while keeping their exact
calculated totals and overrides. Each row uses just one metric scale: g/mL below
one kilogram/litre, kg/L at or above it. Volume rows also
show clearly labelled US, metric and Imperial cup/fluid-ounce alternatives.
Copied grocery text includes the same alternatives. Unknown amounts remain
unknown, and alternatives never assume an ingredient density. For example, an
exact `59 1176473/8000000 mL` requirement displays as **60 mL**, with alternatives
including **¼ US cup** and **2 US fl oz**. See
[ADR 0009](docs/adr/0009-practical-shopping-measures.md) for the rounding policy.

## Recipe importing and website sessions

On Meals, choose **Import recipe**, paste a public HTTPS recipe URL and choose
**Check website**. **Open recipe website** opens a separate WebView2 window;
sign in on the website itself. Its local profile retains sessions best-effort.
Cross-origin redirects, popups, downloads and device permissions are blocked. If
a login, MFA or page needs another origin, use your regular browser instead.
Close the previous recipe window before opening a different recipe.

For Budget Bytes, Allrecipes, Spend With Pennies, RecipeTin Eats and NYT Cooking,
return to PantryPal once the recipe is visible, confirm permission and choose
**Read recipe from browser**. The authorized adapter reads visible recipe text
into a preview. Alternatively enter permitted recipe text, or paste an authorized
schema.org Recipe JSON-LD export/object (without script tags).
Check servings, choose explicit regional units,
review canonical ingredient suggestions and confirm saving. Unknown quantities,
ranges and package sizes retain their original wording. Imported HTML markup is
inert text. The saved meal remains editable; its original import payload,
attribution, source and timestamp remain in a durable import record.

For a missing ingredient, choose **Add ingredient** beside its ingredient
selector in recipe review. The modal saves the ingredient to your catalog and
selects it on that recipe line, retaining quantities and original wording.
Existing name/alias matches require an explicit **Use existing** choice.
Ingredients saved this way remain in the catalog even if you later cancel the
recipe. The Ingredients page's **Add ingredient** uses the same modal; Cancel
or Escape closes it and restores focus when no save is in progress.

Duplicate source URLs require an explicit **Save another meal** choice and never
overwrite an existing recipe. Failed saves retain the draft; identical retries
reuse the original action ID. Cancel/discard does not create a meal.

**Clear website sessions** is available in Import and Settings. Confirming it
closes recipe browsing and clears cookies, cache, local storage and saved browser
data in `%APPDATA%\ca.pantrypal.desktop\recipe-browser-profile`. Recipes and other
household data are unchanged. Clearing waits for WebView2 completion; an error
does not claim success. Website passwords/tokens are never collected by the app.

The owner confirmed agreements for these five publishers. All five supplied URLs
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

## Backups and everyday settings

In **Settings → Backups and restore**, choose **Export JSON backup**. The
completed path appears on screen. Files are stored under
`%APPDATA%\ca.pantrypal.desktop\backups`; copy them elsewhere with Windows Explorer
for off-device protection. They contain personal household data in plain JSON,
including retired recipes, original quantities, custom fields, plans, cooking
history, shopping adjustments/manual items, import provenance and preferences.
Website profiles and login sessions are excluded. Files remain until you remove
them; there is no automatic retention cleanup.

To restore, choose a JSON file, select **Merge** or **Replace**, then **Review
backup**. Merge adds new IDs and keeps identical records; conflicts in recipes,
catalogs, settings or references stop the entire import. It does not overwrite
or remap IDs. Replace restores the entire snapshot, removing current records
absent from it. Review the counts and confirmation, then choose **Confirm merge**
or **Confirm replacement**. Cancel leaves household data unchanged. A recovery
JSON backup is saved before either mode writes; backup failure stops the import.
If household data changes after review, review again before retrying.

Only format 1 / schema 8 JSON backups are currently supported, up to 50 MiB and
100,000 rows. Old/future or damaged files are rejected without resetting data.
Use a matching application version for other schemas. To undo a successful
restore, review and replace with its displayed recovery JSON file.

Pending database upgrades automatically create a consistent
`backups/before-migration-*.sqlite3` copy before migrating. These are native SQLite
files, not JSON imports. For manual migration recovery: close every PantryPal
process, preserve the current database and any WAL/SHM files in a separate safety
folder, then put a copy of the chosen recovery file at `pantrypal.sqlite3` in the
app-data directory with no old WAL/SHM companions. Open it with the matching
application version. Never overwrite your only recovery copy. Ask for help if
the compatible version or file is uncertain.

Settings also saves measurement conventions/formatting, shopping aisle order
(keyboard-accessible Up/Down controls), and website session clearing. Clipboard
URL prefill is off by default. Enabling it reads a bounded public HTTPS URL only
when **Import recipe** is clicked; it never monitors the clipboard or opens a
website automatically. Invalid clipboard contents leave manual entry available.

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

If storage is unavailable or incompatible, Settings and Ingredients display an error and the app never silently resets data. Use Settings to export and restore household JSON backups as described below. For a manual database safety copy, close all PantryPal processes and copy the database and any remaining WAL/SHM companions; do not copy only the main file while the app is writing. Keep website profiles separate from household backups.

`tests/fixtures/demo-meal.json` provides a deterministic meal with an exact fractional amount, raw wording and an unknown/to-taste line. Rust and frontend tests use it without seeding production meals. Tests create isolated temporary databases and never use the household app-data database.
