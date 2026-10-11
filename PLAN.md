# PantryPal — Implementation Plan

**Progress convention:** Coding agents change `[ ]` to `[x]` **only** when implemented, tested, and verified. Leave incomplete tasks unchecked. Mark a blocked task as `- [ ] ... — BLOCKED: <reason>` and note follow-up in the phase log. Complete phases in order unless dependencies justify parallel work. Follow [DESIGN.md](DESIGN.md) and [AGENTS.md](AGENTS.md).

## Status dashboard

| Phase | Outcome                                               | Status      |
| ----- | ----------------------------------------------------- | ----------- |
| 0     | Repository scaffold and quality gates                 | Complete    |
| 1     | SQLite schema, seed catalog, and core domain          | Complete    |
| 2     | Measurement engine and ingredient management          | Complete    |
| 3     | Meal library and custom fields                        | Complete    |
| 4     | Weekly planner, cooking history                       | Complete    |
| 5     | Shopping list engine and copy/edit UX                 | Not started |
| 6     | Authenticated browser feasibility and import pipeline | Not started |
| 7     | Backup, preferences, security and polish              | Not started |
| 8     | End-to-end validation and Windows release             | Not started |

> **Rule:** Update this dashboard and checkboxes in the same commit as completed work. Do not mark a phase complete until its acceptance gate passes.

## Phase 0 — Foundation

**Goal:** A runnable Windows Tauri/React project with a repeatable build and test workflow.

- [x] Initialize Tauri 2 + React + TypeScript workspace, lockfiles and Rust tooling.
- [x] Configure Tailwind and accessible UI component primitives (shadcn/ui or equivalent).
- [x] Define frontend folders, typed IPC wrapper, backend module structure and error conventions.
- [x] Configure formatting/linting: `cargo fmt`, `cargo clippy`, TypeScript strict type checking, ESLint/Prettier.
- [x] Set up Rust unit tests and frontend test runner; add a smoke test in each layer.
- [x] Configure Tauri 2 capabilities with least privilege and no remote-origin privileged commands.
- [x] Document dev commands and Windows prerequisites in `README.md`.
- [x] Verify developer can run the app on Windows and produce a debug build.

**Acceptance gate:** Windows dev app opens, navigation shell renders, lint/typecheck/tests pass, and the release/build pipeline is documented.

**Phase log (2026-10-11):** Phase 0 acceptance gate passed on Windows x64. Added the five-section placeholder shell, Tailwind/native accessible primitives, TanStack Query, typed identity IPC and structured error conventions; Rust domain/service/persistence/importer boundaries; committed dependency lockfiles; pinned Rust tooling; Windows quality workflow; README and ADR 0001. Later-phase data and importer behavior remain unimplemented.

- Passed `npm ci` and `npm audit` (zero vulnerabilities after updating initial advisory-affected Vitest/tooling dependencies).
- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (8 tests, including keyboard/focus, IPC success/error and static capability/CSP regression checks).
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (2 Rust tests).
- Passed `npm run tauri build -- --debug --no-bundle` (including TypeScript/Vite production build); executable at `target/debug/pantrypal.exe`.
- Ran `npm run tauri dev -- --no-watch`; visually verified the native Windows window, navigation shell, Tailwind styling and successful Rust IPC (`Desktop · v0.1.0`). Also launched and visually verified the bundled debug executable without Vite. Local verification captures are ignored under `.verification/`.
- Resolved environment/setup failures: sandbox denied npm registry access and native app initialization; reran with approved network/runtime access. Clean-installed the updated compatible npm dependency set after stale peer resolution failed. Closed leftover project processes after a failed dev launch caused a port conflict and locked the executable; subsequent dev launch and debug build passed. Corrected initial heading autofocus found during visual inspection, with a StrictMode regression assertion.
- No Phase 0 blocker remains. In-app browser automation was unavailable; native window captures and deterministic UI tests supplied this phase's verification. GitHub CI execution, release/NSIS packaging and clean-machine installation are **not verified**; their commands are documented. Actual external-page IPC isolation remains a Phase 6 runtime spike; Phase 0 verifies the local-only capability configuration and command manifest.

## Phase 1 — Database and core models

**Goal:** Durable household data with versioned, reproducible schema.

- [x] Choose SQLite/SQLx integration and local app-data path; enable foreign-key constraints.
- [x] Create migrations for ingredients, aliases, categories, shopping sections and unit registry.
- [x] Create migrations for meals, meal ingredient lines, custom field definitions and values.
- [x] Create migrations for plan entries, cooked events, shopping lists/items, import records and settings.
- [x] Implement Rust domain models, validators, repository boundaries and typed IPC commands.
- [x] Create a modest, idempotent seed catalog of common ingredients, units, types and sections.
- [x] Implement transaction-safe CRUD, referential integrity and safe deletion/retirement behavior.
- [x] Add migration tests from an empty DB and an existing DB; test process restart persistence.
- [x] Add deterministic demo/fixture data for UI development.

**Acceptance gate:** A fresh app initializes exactly once, CRUD persists after restart, and the migration suite passes without data loss.

**Phase log (2026-10-11):** Phase 1 acceptance gate passed on Windows x64. Added SQLx/SQLite per-user storage, foreign keys/WAL, four embedded migrations, 16 ingredient seeds, six categories, seven sections and 36 explicit unit definitions. Rust repositories provide validated catalog/vocabulary/meal/custom-field CRUD with transactional child updates, stable IDs, meal revision checks and confirmed retirement. Quantities preserve exact entered rational pairs, units, raw text and source convention; null remains distinct from boolean false. Added typed local-only IPC, a read-only Ingredients catalog, storage status and a visible startup error state. Architecture and limits are recorded in ADR 0002; the deterministic meal fixture never populates household data.

- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (16 tests).
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (20 unit tests plus one process-restart integration test). The integration test executes the normally ignored child harness in separate write/read processes; it is not an unexecuted acceptance check.
- Migration tests verify empty initialization, upgrades from existing catalog and meal/field/shopping data, seed edits retained across reopening, failed migration rollback, incompatible/checksum rejection, corruption preservation and referential integrity. Repository tests verify rollback after failed child edits, retirement references, explicit field type validation, null/false and manual shopping adjustments in storage. No shopping regeneration behavior is claimed.
- Passed `npm run tauri build -- --debug --no-bundle` (including TypeScript/Vite production compilation).
- Launched `target/debug/pantrypal.exe` twice. Verified `%APPDATA%\ca.pantrypal.desktop\pantrypal.sqlite3` exists, and checked it read-only with `sqlite3 -readonly`: four applied migrations, 16 ingredients, zero demo meals, `PRAGMA integrity_check` = `ok`, and no `PRAGMA foreign_key_check` violations. After reopening, visually verified the native shell reports `Local data ready` through the actual storage-status IPC. Native verification captures are ignored under `.verification/`.
- Resolved the initial SQLite C build failure (`stdarg.h` include path missing) by importing the installed Visual Studio SDK environment with `. ./scripts/Enter-DevEnvironment.ps1`; the helper is documented and used in CI. Subsequent checks/builds passed. Native keyboard automation could not focus the WebView; catalog/status UI behavior is covered by deterministic UI tests, while native startup/reopen/storage IPC was verified directly.
- No Phase 1 blocker remains. Full ingredient/meal/custom-field editors, measurement arithmetic, planning/cooking workflows, shopping regeneration, importing and backups remain in their planned phases. Planning/shopping/import/settings tables are storage contracts only. GitHub CI execution, release installer packaging and remote-page runtime isolation remain **not verified**. Later-phase checkboxes were not changed.

## Phase 2 — Units and ingredients

**Goal:** Correct mixed-unit handling, ingredient identity, and practical catalog editing.

- [x] Implement unit dimension/system registry and parse amount forms including `½`, `1/3`, `1 1/2`.
- [x] Implement precise mass conversions for mg/g/kg/oz/lb.
- [x] Implement explicit regional volume variants and conversions (US, metric, Imperial).
- [x] Implement temperature display conversions separately from grocery quantities.
- [x] Implement a `cannot convert` result for incompatible or ambiguous dimensions.
- [x] Add optional ingredient-specific conversions (count/mass, volume/mass, package sizes), approximation metadata.
- [x] Add tests for round-trips, boundaries, ambiguous cups, fractional scaling and inconvertible cases.
- [x] Build ingredient browse/search/create/edit screens for categories, shopping sections, aliases, season months, default unit and notes.
- [x] Enforce canonical alias/name uniqueness and display candidate matches for duplicates.
- [x] Build measurement preference UI and simple inline conversion control.

**Acceptance gate:** Test vectors in DESIGN.md yield correct results without inventing conversions, and ingredient CRUD works end to end.

**Phase log (2026-10-11):** Phase 2 acceptance gate passed on Windows x64. Added checked exact Rust rational parsing/arithmetic, compatible mass and explicit regional volume conversions, isolated affine temperature display, explicit ambiguity/incompatibility results and ingredient-specific conversion rules with original text and approximation provenance. Ingredients now supports alias search, create/edit/confirmed retirement, category/section management, season months, preferred units and notes. Duplicate names/aliases show existing canonical candidates, including reserved retired names. Settings saves validated measurement mode, cup review convention, fraction/decimal formatting and future aggregation preferences; inline conversion controls preserve original values. Migration 0005 is additive and retains existing rules/preferences. ADR 0003 documents precision bounds, single-rule paths, ambiguity policy and the test boundary.

- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (23 tests). Keyboard/form flows cover ingredient create/search/edit/retirement, canonical duplicate review, failed-save draft retention, independent vocabularies, ingredient rules, explicit cup selection, rounded/approximate labels, temperature isolation and preference saves.
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (27 unit tests and three integration tests). The process-restart test runs its normally ignored child twice in separate processes and now checks ingredient rules/preferences as well as earlier data. Measurement tests cover DESIGN.md quantity vectors, all compatible mass/volume unit-pair round-trips, fractional scaling, input/overflow boundaries, Unicode/unknown parsing, signed temperatures, scoped/reversed/ambiguous rules, stable rule edits and confirmed retirement.
- The Tauri integration test uses actual command deserialization, the capability manifest and temporary SQLite storage with only the OS runtime mocked. Ingredient create/edit/retire, alias candidates, rule conversion and preference persistence pass through IPC. Simulated remote-origin catalog/conversion requests are denied. The schema 4-to-5 integration test retains edited ingredients, legacy rule rational pairs and existing mode/fraction settings; reopening does not reseed or overwrite them. Migrations 0001–0004 were not modified.
- Passed `npm run tauri build -- --debug --no-bundle`, including TypeScript/Vite production compilation. Launched the bundled Windows executable, visually verified the live catalog, new-ingredient form/focus and measurement settings, and executed the native temperature converter (`1 °C` → `33⅘ °F`) through real IPC. Closed/reopened the app and verified `Local data ready`. Read-only `sqlite3` checks confirm five applied migrations, 16 ingredients, zero demo meals, integrity `ok` and no foreign-key violations. Native verification used scoped Win32 navigation/input and captures under ignored `.verification/`; it did not save test records or alter household preferences.
- Resolved initial verification failures: corrected an unreduced expected fraction; fixed TypeScript closure narrowing and updated frontend mocks for the new query/mutation APIs; repaired a test fixture encoding introduced by a PowerShell read; renamed checked arithmetic methods to satisfy Clippy. The new Tauri mock-runtime integration executable initially failed with `STATUS_ENTRYPOINT_NOT_FOUND` because its Common Controls v6 manifest was absent. Added a Windows test-only linker manifest and a test icon; all subsequent checks passed.
- No Phase 2 blocker remains. Exact arithmetic is bounded and rejects overflow; multiple ingredient-rule paths require user review and do not chain automatically. Measurement mode/aggregation preferences will be consumed by grocery generation in Phase 5. Native CRUD writes are covered by the command/SQLite integration and React flow tests, rather than writing disposable recipes/ingredients into household data. GitHub CI, release/installer packaging and actual external-page WebView isolation remain **not verified**. Later-phase checkboxes were not changed.

## Phase 3 — Meals and extensible fields

**Goal:** Create, find and edit meals without imports.

- [x] Build meal list/detail/editor with title, base servings, times, instructions, notes, tags, and source link.
- [x] Implement ordered ingredient lines with quantity, unit, ingredient chooser, raw wording, optional flag and preparation note.
- [x] Support unknown amounts and informal units without discarding information.
- [x] Create/manage custom field definitions: boolean, select, number, date, text and rating.
- [x] Implement typed custom-field validation and values, including unknown boolean state.
- [x] Allow meal search/filter by title, ingredients, custom fields and derived metadata.
- [x] Implement safe custom-field rename/retire and confirmation for destructive changes.
- [x] Add tests for meal CRUD, field values, invalid input and ingredient deletion behavior.

**Acceptance gate:** A user can create a complete meal with custom fields, restart, locate it by filters, and edit it without losing data.

**Phase log (2026-10-11):** Completed the meal list/detail/editor, ordered ingredient lines, all six custom field definitions and typed values, Rust search with canonical ingredients/aliases/tags/field predicates and actual last-cooked date filters, safe rename/confirmed retirement, draft-preserving validation errors, and migration 6 for original amount text. ADR 0004 documents immutable field shapes, unknown defaults, raw quantity retention and read-only history scope. Verified `npm run check` (format/lint/typecheck and 28 frontend tests), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (29 unit tests and four integration tests; the parent executes the normally ignored restart child twice), and `npm run tauri build -- --debug --no-bundle` on Windows. Actual IPC/SQLite coverage saves all six field kinds, rejects invalid/stale edits, renames/retires safely and reopens without loss; the process restart gate locates a complete recipe by combined filters and edits it. Native Windows screen inspection verified the library/editor and title focus, startup schema 6, database integrity and foreign keys; no test meals were added to household data. Initial historical-fixture and UI-provider test failures were fixed. One rerun encountered the open native executable lock; after graceful close, the suite passed. No Phase 3 blockers. History recording remains Phase 4; importing/browser opening remains Phase 6; release installer checks remain Phase 8.

## Phase 4 — Week planner and history

**Goal:** Select a useful week of meals and record what was actually cooked.

- [x] Implement Monday-start local weeks, previous/next navigation and unscheduled-week pool.
- [x] Add/remove/reorder planned meals and choose servings per occurrence.
- [x] Track plan status (`planned`, `cooked`, `skipped`) with explicit user actions.
- [x] Record cooked events exactly once per action; implement undo/correction semantics.
- [x] Derive and display `last eaten` from cooked events; allow backfill if appropriate.
- [x] Connect planner to meal detail and shopping-list generation action (week-aware Shopping handoff; calculations remain Phase 5).
- [x] Test repeated meals, date edges, invalid servings and plan changes.

**Acceptance gate:** The weekly view works across month/year boundaries and only intentionally cooked meals update eating history.

**Phase log (2026-10-11):** Phase 4 acceptance gate passed on Windows. Added Monday–Sunday local weeks, previous/current/next and calendar navigation, an unscheduled pool, repeated occurrences, positive exact fractional servings, day moves and keyboard ordering. Explicit planned/cooked/skipped actions use transactional repositories, per-week revisions and durable action receipts. Cooking creates one linked event; retries do not repeat it, including after undo or restart. Backfill, confirmed correction and undo retain voided records and an audit trail. Last eaten uses only non-voided cooked events; scheduling, skipping and moving never imply cooking. Removing a plan preserves detached cooking history and its origin ID. Historical retired recipes and their ingredient labels remain readable. Migration 7 preserves existing plans, exact legacy serving pairs, events, recipes and shopping adjustments. ADR 0005 records local-date/timezone policy, retry/undo semantics and the Phase 5 handoff boundary.

- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (35 frontend tests). New flows cover independent repeated meals, fractions, day/pool moves, keyboard ordering, skip/cook/confirmed undo/removal, backfill/date correction, failed-input preservation and same-action retry, historical detail labels, recipe navigation and week-aware Shopping handoff.
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked` (34 unit tests and five integration tests). The parent runs its normally ignored child in separate write/read processes, retaining repeated plans and actual cooking and replaying the saved original action safely. Actual Tauri command deserialization/ACL/SQLite tests cover planner/history and reject simulated remote-origin requests. Repository tests cover month/year/leap-day/DST edges, invalid servings/dates/future cooking, stale revisions, exact reordering membership, action reuse, rollback, correction/undo audit, retirement and removal preservation. Schema 6-to-7 migration coverage retains prior cooked plans and manual shopping overrides; migrations 1–6 remain unchanged.
- Passed `npm run tauri build -- --debug --no-bundle`, including TypeScript/Vite production compilation. Native Windows inspection verified local current-week defaults, repeated next-week navigation to 28 December 2026–3 January 2027, and Shopping receiving the exact Monday `2026-12-28`. Read-only household database checks show seven applied migrations, 16 ingredients, zero test meals/plans/history, integrity `ok` and no foreign-key violations. Writes and history corrections are verified with isolated command/database tests and UI flows; no disposable records were added to household data.
- Initial verification issues were resolved: removed an unused Rust import, fixed historical migration fixture filenames, and made the date-label UI test respect local display order rather than assuming US month/day ordering. Final checks pass. No Phase 4 blocker remains. The Shopping action carries week context to the generation screen; calculation/regeneration/clipboard functionality stays in Phase 5. GitHub CI execution, release packaging and actual external-WebView isolation remain not verified. Later-phase checkboxes were not changed.

## Phase 5 — Grocery generation

**Goal:** A trustworthy, editable and copyable list for the selected week.

- [ ] Implement recipe scaling using planned/base servings with precise arithmetic.
- [ ] Aggregate only canonical identical ingredients and compatible unit dimensions/conversion paths.
- [ ] Honor per-ingredient preferred shopping units and global unit preferences.
- [ ] Show separate rows for unknown or incompatible units, with clear explanation.
- [ ] Flag approximate conversions and unknown/to-taste ingredient quantities.
- [ ] Group by shopping section, with configurable section ordering and deterministic item order.
- [ ] Implement manual items, amount overrides, checkmarks and exclusions.
- [ ] Define and implement regenerate/merge behavior preserving user edits; add regression tests.
- [ ] Indicate stale list when underlying meal/plan data changes, with explicit refresh.
- [ ] Add one-click plain-text copy to clipboard, with a helpful empty-list state.
- [ ] Test two recipes with mixed units, duplicate meal instances and a partly checked/manual list.

**Acceptance gate:** Multiple planned meals yield a correct, grouped list; edits survive refresh per policy; clipboard output pastes cleanly into another app.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 6 — Recipe importing and secure browsing

**Goal:** Import supported recipes where permitted without compromising desktop security.

### Feasibility spike (required before implementation)

- [ ] Research and document Tauri 2/WebView2 options for persistent browser profiles and session retention.
- [ ] Prototype a distinct website browser window that can log in (on a test/authorized site) and retain sessions after app restart.
- [ ] Validate **safe supported** access to recipe data from remote pages; prove remote content has no privileged Tauri commands.
- [ ] Investigate Cook Smarts official export/API/permission and embedding compatibility; record supported/not-supported outcome.
- [ ] Write an ADR with chosen extraction mechanism, security boundary, site constraints, and fallback if blocked.

### Import pipeline

- [ ] Define `ImportedRecipeDraft` and `RecipeImporter` interface with source/provenance metadata.
- [ ] Implement URL/domain validation, redirect policy, bounded payload/time, and non-secret logging.
- [ ] Implement generic authorized JSON-LD (`schema.org/Recipe`) extractor using saved fixtures.
- [ ] Implement allowed website-specific adapter(s) only after permission and feasibility checks.
- [ ] Build paste URL → browser/login if needed → preview → ingredient matching → confirm save flow.
- [ ] Match aliases and existing ingredient IDs; explicitly resolve uncertain names, amounts or units.
- [ ] Store source URL, original ingredient strings, normalized values and timestamp; support editing the saved recipe.
- [ ] Handle expired login, MFA, unavailable metadata, unsupported site, duplicate import and cancellation gracefully.
- [ ] Provide clear-session/reset controls and manual-entry fallback.
- [ ] Add importer unit/fixture tests and verify an external page cannot invoke privileged commands.

**Acceptance gate:** An allowed test-site import works with review and persistent session where supported; no remote page gains native privileges. If Cook Smarts importing is prohibited/infeasible, document it and leave the core app functional.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 7 — Robustness and usability

**Goal:** Safe, polished everyday operation.

- [ ] Implement versioned JSON export/import with validation and clear merge/replace behavior.
- [ ] Implement backup before risky migrations/imports; document backup location and restore process.
- [ ] Add settings for measurement conventions, formatting, aisle order and browser data reset.
- [ ] Improve Windows keyboard navigation, focus, contrast, labels, error states and empty states.
- [ ] Add optional clipboard URL prefill on deliberate Import action (not continuous clipboard monitoring).
- [ ] Confirm app works offline apart from deliberate recipe website visits.
- [ ] Audit URL handling, Tauri capabilities, stored browser profiles, third-party dependencies and log redaction.
- [ ] Test resilience to malformed import data, interrupted writes and corrupted/old backups.
- [ ] Add reasonable performance tests with a realistic personal dataset.

**Acceptance gate:** Data can be exported, restored and used offline; security and accessibility smoke checks pass.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 8 — Validation and distribution

**Goal:** A reliable Windows deliverable for its intended user.

- [ ] Run `cargo fmt`, `cargo clippy`, Rust tests, TypeScript lint/typecheck, frontend tests and production build.
- [ ] Test clean install, app restart, migrations, and upgrade from previous seeded data.
- [ ] Manually test end-to-end: ingredient → meal → custom field → plan → shopping → copy.
- [ ] Manually test conversion/ambiguity cases, list override regeneration and backup restore.
- [ ] Test authorized import/login flow or clearly document unavailable integration and fallback.
- [ ] Package Windows installer (Tauri/NSIS or equivalent) and note any signing warnings.
- [ ] Document install, uninstall, local-data directories, backups and how to reset website sessions.
- [ ] Resolve critical defects and capture known limitations in `README.md`.
- [ ] Tag a first usable version once the MVP definition of done in DESIGN.md is met.

**Acceptance gate:** Installer runs on a clean Windows environment and the main happy path succeeds without developer intervention.

**Phase log:** _Add date, commands run, and blockers here._

## Future ideas — explicitly out of MVP

- [ ] “What should we eat?” recommendations based on family ratings, last eaten, prep time, seasonality, and variety.
- [ ] One-click weekly meal suggestions and manual reshuffle.
- [ ] Enhanced shopping history and favorite manual items.
- [ ] Optional nutrition information or pantry inventory.
- [ ] Optional second-device synchronization if there is a real household need.

## Agent completion protocol

For every substantive task: (1) implement a small vertical slice, (2) add/update tests, (3) run appropriate checks, (4) update the exact PLAN.md checkbox and dashboard, (5) document deviations/ADRs and remaining blockers, and (6) summarize changed files and verification results. **No checkbox for planned-but-unverified code.**
