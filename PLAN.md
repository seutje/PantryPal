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
| 5     | Shopping list engine and copy/edit UX                 | Complete    |
| 6     | Authenticated browser feasibility and import pipeline | Complete    |
| 7     | Backup, preferences, security and polish              | Complete    |
| 8     | End-to-end validation and Windows release             | In progress |

> **Rule:** Update this dashboard and checkboxes in the same commit as completed work. Do not mark a phase complete until its acceptance gate passes.

## Phase 0 — Foundation

**Goal:** A runnable Windows Tauri/React project with a repeatable build and test workflow.

- [x] Initialize Tauri 2 + React + TypeScript workspace, lockfiles and Rust tooling.
- [x] Configure Tailwind and accessible UI component primitives (shadcn/ui or equivalent).
- [x] Define frontend folders, typed IPC wrapper, backend module structure and error conventions.
- [x] Configure formatting/linting: `cargo fmt`, `cargo clippy`, TypeScript strict type checking, ESLint/Prettier.
- [x] Set up Rust unit tests and frontend test runner; add a smoke test in each layer.
- [x] Configure Tauri 2 capabilities with least privilege and no remote-origin privileged commands.
- [x] Document dev commands and Windows prerequisites in `DEVELOPMENT.md`.
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

### Phase 4 follow-up — Meal picker for each day

- [x] Add an Add meal action to each dated day, opening the existing Meals search and filters in a modal.
- [x] Add the selected recipe as a separate occurrence on that day, using base servings and preserving multiple/repeated meals.
- [x] Verify keyboard selection, focus restoration, cancellation, pending-save protection and identical-action retry after failure.

**Follow-up log (2026-10-12):** Added a selection mode to MealsPage and a native
modal dialog in Week. Search, ingredient/tag/custom-field/history filters and
clear filters share the existing library query and Rust validation; selecting
the full meal card uses the existing revision-checked, durable plan mutation.
The modal closes only after successful saving and restores the day's button
focus; Escape/Cancel cannot dismiss an in-progress save. Failure keeps the
search and exposes retry/reload inside the modal. The unscheduled form and
occurrence editing remain available. Multiple meals per day were already
supported; no Rust, migration, capability or data-rule change was needed.
README documents the base-servings default and separate-occurrence behavior.

- Passed focused `npm test -- src/features/week/WeekFlows.test.tsx src/features/meals/MealFlows.test.tsx`
  (12 tests before the additional pending-save test), then `npm run check`
  (format/lint/typecheck and all 62 tests) and
  `npm run tauri build -- --debug --no-bundle` on Windows.
- Tests cover all seven day buttons, shared filter/search payloads, two distinct
  meals on the same day, fractional base servings, unique occurrence IDs,
  keyboard activation, focus, Escape/Cancel, unchanged cooking status, repeat
  selection/dismissal prevention while saving and identical retry. Corrected
  the initial test's ingredient ID to use the actual catalog fixture. Rust tests
  and native visual inspection were not rerun for this UI-only change. No
  blockers; phase dashboard and later-phase checkboxes remain unchanged.

### Phase 4 follow-up — Occurrence servings and scaled ingredients

- [x] Expose Edit servings directly on Week meal cards without changing recipe base servings or other occurrences.
- [x] Show backend-derived ingredient amounts for the occurrence's saved servings, preserving units, unknowns and original recipe data.
- [x] Verify fractional scaling, independent repeated occurrences, unchanged recipes after reopen and shopping totals after explicit refresh.

**Follow-up log (2026-10-12):** Planned servings were already separate from
recipe base servings and used by Shopping. Added a dedicated servings-only
action reusing the validated plan edit, plus expandable ingredient amounts on
each Week card. Rust builds view-only quantity strings with checked rational
scaling, source units and configured fraction/decimal formatting. Unknown
amounts remain unknown; temperatures are not scaled. Recipe editing and
occurrence scheduling remain separate actions. No migration or regeneration
policy change; README and ADR 0005 document the display and refresh behavior.

- Passed `npm run check` (format/lint/typecheck and 63 frontend tests),
  `cargo test --workspace --locked` (53 unit plus eight integration tests),
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check` and
  `npm run tauri build -- --debug --no-bundle` on Windows.
- New database coverage changes one of two dated occurrences, checks exact
  fractional ingredient displays and combined shopping totals, verifies stale
  shopping detection, and reopens to confirm independent servings and unchanged
  recipe/source quantities. UI coverage verifies the direct action, saved scaled
  ingredients and cancellation. Fixed an initial test query that matched the
  unknown ingredient in both occurrences by scoping it to the expanded one.
  Native visual/clipboard acceptance was not rerun. No blockers; phase dashboard
  and later-phase checklists remain unchanged.

## Phase 5 — Grocery generation

**Goal:** A trustworthy, editable and copyable list for the selected week.

- [x] Implement recipe scaling using planned/base servings with precise arithmetic.
- [x] Aggregate only canonical identical ingredients and compatible unit dimensions/conversion paths.
- [x] Honor per-ingredient preferred shopping units and global unit preferences.
- [x] Show separate rows for unknown or incompatible units, with clear explanation.
- [x] Flag approximate conversions and unknown/to-taste ingredient quantities.
- [x] Group by shopping section, with configurable section ordering and deterministic item order.
- [x] Implement manual items, amount overrides, checkmarks and exclusions.
- [x] Define and implement regenerate/merge behavior preserving user edits; add regression tests.
- [x] Indicate stale list when underlying meal/plan data changes, with explicit refresh.
- [x] Add one-click plain-text copy to clipboard, with a helpful empty-list state.
- [x] Test two recipes with mixed units, duplicate meal instances and a partly checked/manual list.

**Acceptance gate:** Multiple planned meals yield a correct, grouped list; edits survive refresh per policy; clipboard output pastes cleanly into another app.

**Phase log (2026-10-11):** Phase 5 acceptance gate passed on Windows. Added exact Rust planned/base-serving scaling, canonical aggregation through unique universal or ingredient-specific paths, ingredient/global display preferences, separate unknown/informal/missing/incompatible rows, approximation/rounding provenance, configurable section ordering and deterministic copy text. Shopping now provides manual additions, editable quantities, checkmarks, reversible exclusions, confirmed override reset, explicit refresh and a stale-source indicator. Stable keys preserve adjustments across refresh and display-unit changes; disappeared adjusted rows remain visible for review and are omitted from copy. Planned/cooked occurrences count once each; skipped occurrences do not. Original line values, wording, units and serving pairs remain in source provenance. Migration 8 adds versioned documents/revisions while retaining every earlier shopping row; legacy payloads are carried forward with explicit review labels. ADR 0006 documents these decisions and the conservative full-catalog refresh fingerprint.

- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (40 frontend tests). Shopping flows exercise explicit refresh, checked overrides, manual unknown items, exclusion/orphan labels, confirmed reset, an explicit no-unit override, selected-week copying, stale-copy blocking, validation correction with retained input, identical-action retry, empty guidance and clipboard errors. Existing planner/library/catalog/security flows still pass.
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked` (41 unit tests plus seven integration tests). The restart parent executes its normally ignored child twice and now verifies manual/check/override/exclusion persistence followed by refresh after a recipe edit. New tests cover two mixed-unit recipes, fractional duplicate occurrences, explicit regional cups, scoped/ambiguous rules, prevention of implicit rule chaining, unknown/unresolved/informal/missing units, historical temperature non-scaling, preferences, deterministic sections, totals beyond JavaScript integer precision, overflow rollback, disappearing/reappearing keys, exact retries and stale revisions. IPC tests use actual deserialization, ACL and temporary SQLite, including remote-origin denial of get/change/copy commands. Schema 7-to-8 integration verifies legacy basis/override metadata and flags remain intact.
- Passed `cargo test -p pantrypal --lib windows_clipboard_roundtrip --locked -- --ignored --exact services::clipboard::windows::tests::windows_clipboard_roundtrip`. This explicitly invoked test generates a temporary two-occurrence plan, applies an override/checkmark, writes backend text through the production Windows UTF-16 writer, reads it back exactly and pastes into a separate Windows Forms text control. The normal suite leaves the clipboard alone. No generic clipboard read/write IPC or new dependencies were introduced.
- Passed `npm run tauri build -- --debug --no-bundle`, including TypeScript/Vite compilation. Launched/reopened the final bundled executable; visually verified the native Shopping empty state and manual form. Read-only household checks confirm schema 8, 16 ingredients, zero test meals/plans/shopping lists, integrity `ok` and no foreign-key violations. All fixture writes use temporary databases. Native captures and the paste fixture are ignored under `.verification/`.
- Resolved verification issues: updated migration expectations and new-query mocks, corrected a fixture section ID/unit symbol, fixed strict TypeScript indexing, wrapped the UUID lazy initializer, and corrected explicit null-unit restoration. A separate clipboard recheck encountered changed contents after its writer process exited; the final acceptance harness verifies another-process paste while the fixture writer remains alive and passes. No Phase 5 blocker remains. Unrelated catalog edits can conservatively request refresh; changed conversion group identities retain edits as review rows rather than guessing their reassignment. GitHub CI execution, actual remote-WebView isolation, release installer/clean-machine checks and backup workflows remain not verified or in their planned later phases. Later-phase checkboxes were not changed.

### Phase 5 follow-up — Practical shopping measures

- [x] Round shopping displays upward to readable quantities and scale large metric amounts to L/kg without changing exact stored totals.
- [x] Show same-dimension alternative measures in both the UI and copied list, explicitly distinguishing US, metric and Imperial volumes.
- [x] Verify upward-rounding invariants, the reported olive-oil fraction, unknown amounts, overrides, refresh and restart preservation.

**Follow-up log (2026-10-12):** Added a Rust-only shopping display formatter
with checked rational ceiling and registered same-dimension conversion factors.
The reported exact oil total now reads `60 mL`; alternatives include `0.06 L`,
`¼ US cup` and `2 US fl oz`. Metric small-unit/count displays use whole amounts,
complex fractions use simple upward steps, and large metric amounts use L/kg.
UI and backend clipboard text share the alternatives and rounding labels.
Original totals, overrides, source quantities and unknown wording remain intact;
no migration or regeneration is required. ADR 0009 documents the deliberate
shopping precision policy and independently rounded alternatives. Other
measurement/recipe displays retain their existing precision behavior.

- Passed `npm run check` (format/lint/typecheck and 59 tests),
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked` (51 unit tests and eight integration tests),
  and `npm run tauri build -- --debug --no-bundle` on Windows.
- Tests cover never rounding below the requirement across fraction/decimal
  preferences and registered quantity units, zero/tiny/large values, regional
  cup/fluid-ounce variants, metric thresholds, dimension rejection, retained
  exact oil totals/overrides after refresh/reopen, copy-text formatting and
  frontend alternative labels. The restart parent runs its ignored child;
  the separate OS clipboard test was not rerun because the writer is unchanged.
- Corrected the existing rounding-label expectation, a test-only parser bound
  for large generated integers and Clippy test-module ordering. One frontend
  flow timed out during simultaneous native compilation; a serial full check
  passed without changing its timeout. No blockers remain. Phase 5 remains
  Complete; later-phase status/checklists were not changed.

- [x] Show only one metric scale per shopping row and remove the explanatory alternatives heading from the UI and copied text.

**Display refinement (2026-10-12):** Rust selects g/mL below one kilogram/litre
and kg/L at or above it from the exact effective quantity, including when a
preferred L/kg unit contains a small amount or the primary uses another system.
Cup/fluid-ounce and other customary alternatives remain; the redundant metric
measure and “alternatives, rounded up as needed” heading are removed. Exact
stored totals, overrides and regeneration behavior remain unchanged. Updated
README and ADR 0009 to document the owner's requested refinement.

- Passed `npm run check` (format/lint/typecheck and 59 frontend tests),
  `cargo test --workspace --locked` (52 unit and eight integration tests),
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check` and
  `npm run tauri build -- --debug --no-bundle` on Windows.
- Added metric-scale coverage below/at/above the threshold for metric and
  customary primary units; UI/copy tests verify removal of the heading.
  Updated the old preferred-unit expectation from `½ kg` to `500 g` after its
  initial failure; the final suite passes. Native clipboard acceptance and
  visual inspection were not rerun. No blockers; phase dashboard unchanged.

## Phase 6 — Recipe importing and secure browsing

**Goal:** Import supported recipes where permitted without compromising desktop security.

### Feasibility spike (required before implementation)

- [x] Research and document Tauri 2/WebView2 options for persistent browser profiles and session retention.
- [x] Prototype a distinct website browser window that can log in (on a test/authorized site) and retain sessions after app restart.
- [x] Validate **safe supported** access to recipe data from remote pages; prove remote content has no privileged Tauri commands.
- [x] Investigate Cook Smarts official export/API/permission and embedding compatibility; record supported/not-supported outcome.
- [x] Write an ADR with chosen extraction mechanism, security boundary, site constraints, and fallback if blocked.

### Import pipeline

- [x] Define `ImportedRecipeDraft` and `RecipeImporter` interface with source/provenance metadata.
- [x] Implement URL/domain validation, redirect policy, bounded payload/time, and non-secret logging.
- [x] Implement generic authorized JSON-LD (`schema.org/Recipe`) extractor using saved fixtures.
- [x] Evaluate conditional website-specific adapters after permission and feasibility checks — initial manual fallback (ADR 0007); owner-authorized adapters for the five supplied publishers now implemented and verified (ADR 0008).
- [x] Build paste URL → browser/login if needed → preview → ingredient matching → confirm save flow.
- [x] Match aliases and existing ingredient IDs; explicitly resolve uncertain names, amounts or units.
- [x] Store source URL, original ingredient strings, normalized values and timestamp; support editing the saved recipe.
- [x] Handle expired login, MFA, unavailable metadata, unsupported site, duplicate import and cancellation gracefully.
- [x] Provide clear-session/reset controls and manual-entry fallback.
- [x] Add importer unit/fixture tests and verify an external page cannot invoke privileged commands.

**Acceptance gate:** An allowed test-site import works with review and persistent session where supported; no remote page gains native privileges. If Cook Smarts importing is prohibited/infeasible, document it and leave the core app functional.

**Phase log (2026-10-11):** Phase 6 acceptance gate passed on Windows using an owned, authorized fixture and the documented browser-assisted/manual fallback. Added a dedicated unprivileged WebView2 browser/profile, HTTPS URL/origin validation, popup/download/device-permission denial, confirmed session clearing, bounded Rust JSON-LD/manual adapters, typed review drafts, exact canonical/alias suggestions, explicit quantity/unit/servings review, duplicate confirmation, transactional meal/provenance saves and durable retry receipts. Existing schema 8 import storage is sufficient; no migration or earlier schema edit was needed. Original payload, ingredient strings, yield, attribution, reviewed values and import timestamp survive saving/editing. ADR 0007 records the pre-implementation API research/native spike, site constraints and fallback decision.

- Passed `npm run check`: `npm run format:check`, `npm run lint`, `npm run typecheck`, `npm test` (42 frontend tests). New flows cover URL checking/browser handoff, authorized JSON-LD/manual input, review before writes, explicit US cup/ingredient selection, inert markup, duplicate choice, failed-save input retention and identical-action retry, confirmed cancellation and session reset. Existing keyboard/library/planner/shopping/security flows pass.
- Passed `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked` (45 unit tests plus eight integration tests). The existing restart parent executes its normally ignored child twice. Import tests cover graph/type arrays, instruction sections/steps, Unicode/mixed fractions, ambiguous servings/cups/ranges/package sizes, unknown/to-taste lines, bounded malformed/deep/multiple payloads, inert HTML text, all five requested URLs, secret-bearing/dangerous URLs, missing references/transaction rollback, duplicate confirmation, exact retries, retained provenance/timestamp after editing/reopening and actual IPC ACL rejection. The separately ignored clipboard acceptance test was not rerun; clipboard implementation is unchanged.
- Passed `cargo run -p pantrypal --example browser_spike --locked -- write`, then `read`, then `cleared`, each in a separate Windows process. Native evaluation reads only the owned fixture. Actual ACL errors reject remote identity/catalog/preview/confirm/reset commands. Local IPC previews and confirms the reviewed recipe; subsequent processes read back its original ingredient wording from isolated SQLite. An HttpOnly fixture login and local storage survive restart. The actual reset IPC closes browsing, awaits WebView2 clear completion in a hidden unprivileged window and a third process verifies both cookie and local storage are absent. The HTTP loopback exception and all fixture databases/profiles are confined to this explicit harness under ignored `.verification/`; production rejects loopback URLs.
- Passed `cargo run -p pantrypal --example recipe_sites --locked`: normal native browsing of all five owner-supplied URLs loaded their expected recipe page titles. This check reads only page title/load state, never recipes or credentials, and does not verify full content access or authenticated subscriber access. Deterministic fixtures use original test content, not redistributed publisher recipes.
- Site-specific automation is intentionally unavailable: Budget Bytes' reviewed terms prohibit scraping; automated integration permission was not established for Allrecipes, Spend With Pennies, RecipeTin Eats or NYT Cooking. Cook Smarts' terms restrict collection/downloads beyond normal caching without permission; no sanctioned recipe API/export integration was established. Manual entry or supplied authorized JSON-LD remains functional. The conditional-adapter checkbox records this verified outcome, not an implemented publisher scraper. Cross-origin login/MFA/popups can require the user's regular browser; expiry and unavailable metadata use manual fallback. No paywall/anti-bot bypass was attempted.
- Passed `npm run tauri build -- --debug --no-bundle`, including TypeScript/Vite production compilation. Launched the final bundled executable, visually verified the enabled Meals Import action, import form and URL focus. Read-only household SQLite checks confirm schema 8, zero fixture meals/import records, integrity `ok` and no foreign-key violations. Import writes use isolated databases; native captures are ignored under `.verification/`.
- Resolved verification issues: updated the old disabled-Import expectation, wrapped Query mutation functions to avoid test-context argument mismatch, fixed a Clippy needless borrow and forced the isolated native main window to the local app origin rather than an unavailable Vite server. The session-reset review found that Tauri's convenience clear API discards asynchronous completion; the final implementation waits for the official WebView2 completion callback, reports failures/timeouts and prevents concurrent browser operations. Two direct Windows dependencies reuse exactly the already-locked Tauri package versions; no new resolved packages. Example-only missing icon and Chromium window-class shutdown warnings did not fail the native checks. No Phase 6 blocker remains. Automated publisher integration remains conditional on future permission evidence; GitHub CI, installer/clean-machine validation and backups remain in their later phases. Later-phase checkboxes were not changed.

### Phase 6 follow-up — Authorized publisher adapters

- [x] Implement adapters for Budget Bytes, Allrecipes, Spend With Pennies, RecipeTin Eats and NYT Cooking after the owner confirmed website agreements.
- [x] Connect deliberate browser extraction to existing review, confirmation, duplicate handling and durable provenance; preserve manual fallback.
- [x] Verify saved fixtures, gated/hidden-content rejection, remote IPC isolation and live previews from all five supplied recipe URLs.

**Follow-up log (2026-10-11):** The owner's website agreements supersede the
original manual-only decision for these five publishers. Added exact-host adapter
selection, fixed bounded native DOM capture, versioned website import envelopes
and **Read recipe from browser**. Visible ingredient amounts, current yield and
instructions feed the existing Rust parser/review flow; JSON-LD contributes only
attribution/times after visible content passes checks. Missing or hidden rows/steps,
recognized access gateways, unexpected URLs and oversized payloads fail safely.
Remote capabilities, session/reset policy and all database migrations are unchanged.
ADR 0008 documents authorization, mechanism and limits; Cook Smarts remains manual.

- Passed `npm run check` (format/lint/typecheck and 55 frontend tests),
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked` (47 unit tests and eight integration tests).
  Saved synthetic HTML/JSON fixtures exercise all five adapters, hidden metadata,
  access barriers, ambiguous fractions, unknown amounts, original provenance and
  save/reopen persistence. UI coverage verifies capture errors retain manual input
  and capture success does not skip review or write a meal. The restart parent
  still executes its ignored child; the separate clipboard acceptance test was
  not rerun because clipboard behavior is unchanged.
- Passed `cargo run -p pantrypal --example adapter_sites --locked` on Windows
  with actual production open/capture commands and isolated profile/SQLite storage:
  Budget Bytes 14 ingredient lines, Allrecipes 19, Spend With Pennies 15,
  RecipeTin Eats 22 and NYT Cooking 14. Each returned visible instructions and
  denied the remote page's capture command at ACL; previewing saved zero meals.
  Repeated after partial-hidden-step rejection was added. Results retain only
  counts/flags under ignored `.verification/`; household data was not used.
- Passed `npm run tauri build -- --debug --no-bundle` on Windows, including
  TypeScript/Vite production compilation. Fixed initial lint configuration and
  fixture-generation Unicode encoding failures; final checks passed. Native
  example icon/window-class shutdown warnings did not affect acceptance results.
- No adapter implementation blocker remains. Website layouts/access can change;
  actual subscriber login/MFA is **not verified**. No access-control bypass or
  credential extraction was attempted. Manual entry/session clearing remain
  available. Phase 6 remains Complete; Phase 7 and later checkboxes were not changed.

### Phase 6 follow-up — Ingredient creation during recipe review

- [x] Show ingredient creation in a modal from the Ingredients page and recipe ingredient lines, including scraped recipe review.
- [x] Select a newly saved ingredient on the requested line without losing recipe quantities, original wording or import provenance; require explicit selection of existing duplicates.
- [x] Verify initial/return focus, Escape/cancellation, failed-save retention, duplicate choice and separate ingredient/recipe saves.

**Follow-up log (2026-10-12):** Added a shared native HTML modal dialog rendered
through a portal, reusing the validated ingredient commands/editor. Native modal
behavior supplies the inert background and focus containment; Escape and Cancel
restore focus and cannot dismiss an in-progress save. Recipe editors maintain
their current draft, immediately include newly created catalog IDs in their
selectors and select only the chosen line. Catalog and meal-library queries are
refreshed. Existing duplicates require an explicit choice; ingredient saving
does not save the recipe. The modal and README explain that a saved catalog
ingredient remains if the recipe is later cancelled. No Rust, schema, capability
or importer-extraction change was needed; later phases remain unstarted.

- Passed `npm run check`: formatting, lint, strict type checking and 58 frontend
  tests. New flows cover modal focus/cancel, scraped review creation, rejected
  save/retry, original amount/unit/wording and source retention, duplicate choice,
  unchanged sibling lines and confirmation before recipe writes.
- Passed `npm run tauri build -- --debug --no-bundle` on Windows, including
  TypeScript/Vite compilation. Visually verified the rebuilt native dialog,
  initial name focus, dimmed/inert background and cancellation to the catalog.
  No test data was saved in household storage; captures remain ignored under
  `.verification/`. Native recipe creation writes are covered by the UI mocks
  and the existing Rust persistence contracts; those unchanged Rust tests were
  not rerun in this UI-only slice. No blocker remains; Phase 6 remains Complete.

- [x] Place import review and duplicate-save checkboxes immediately above Save meal and Cancel editing.

**Layout follow-up (2026-10-12):** Moved both import confirmations into the meal
editor immediately before its action buttons, retaining the existing review and
duplicate-save state/validation. Passed `npm run check` (formatting, lint, type
checking and 58 tests) and `npm run tauri build -- --debug --no-bundle` on Windows.
Rust logic is unchanged and its tests were not rerun. No blockers or later-phase
status changes.

## Phase 7 — Robustness and usability

**Goal:** Safe, polished everyday operation.

- [x] Implement versioned JSON export/import with validation and clear merge/replace behavior.
- [x] Implement backup before risky migrations/imports; document backup location and restore process.
- [x] Add settings for measurement conventions, formatting, aisle order and browser data reset.
- [x] Improve Windows keyboard navigation, focus, contrast, labels, error states and empty states.
- [x] Add optional clipboard URL prefill on deliberate Import action (not continuous clipboard monitoring).
- [x] Confirm app works offline apart from deliberate recipe website visits.
- [x] Audit URL handling, Tauri capabilities, stored browser profiles, third-party dependencies and log redaction.
- [x] Test resilience to malformed import data, interrupted writes and corrupted/old backups.
- [x] Add reasonable performance tests with a realistic personal dataset.

**Acceptance gate:** Data can be exported, restored and used offline; security and accessibility smoke checks pass.

**Phase log (2026-10-13):** Phase 7 acceptance gate passed on Windows x64.
Settings exports versioned household JSON, reviews Merge/Replace, requires
explicit confirmation and shows pre-change recovery files. Restore validates
bounded input in isolated migrated SQLite before transactionally replacing
household tables, checking stale reviews and preserving original quantities,
legacy temperature warnings, typed false/null values, history, receipts and
shopping adjustments/manual items. Merge rejects conflicts and keeps identical
records, ignoring creation/update timestamps when comparing. Pending upgrades
first save a consistent SQLite recovery copy. Backup failures stop writes.
Added durable aisle ordering, opt-in bounded URL prefill only on deliberate
Import, navigation Arrow/Home/End focus and meal-search retry. Existing
measurement preferences, session clearing, labels, focus styles and empty states
remain available. README documents backup location, retention and recovery;
ADR 0010 and `docs/phase7-security-audit.md` record decisions and audit limits.
No project dependencies, lockfiles or migrations were changed.

- Passed `npm run check`: formatting, lint, strict type checking and 68 frontend
  tests. New flows cover export/review/confirmation/cancel, failed import review,
  keyboard aisle ordering, draft retention and deliberate clipboard prefill.
- Passed `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked` (58 unit tests and nine integration tests).
  The restart parent executes its ignored child in separate processes. New tests
  cover exact export/restore/reopen, profile-cookie exclusion, merge conflicts,
  stale previews, malformed/oversized/old/future backups, bad references and
  typed values, numeric coercion, legacy preservation, failed recovery creation
  and injected mid-restore rollback. Migration recovery preserves schema-1
  household data; failed backup prevents upgrade. A 200-recipe/21-occurrence
  search/generation/export/validation smoke test met its 20-second budget.
- Passed explicit Windows clipboard acceptance:
  `cargo test -p pantrypal --lib windows_clipboard_roundtrip --locked -- --ignored --exact services::clipboard::windows::tests::windows_clipboard_roundtrip`.
  It verifies grocery paste, valid URL reading, credential/local/text rejection
  and size bounds using fixture contents. Bounded test-only retries handle
  temporary Windows clipboard listener contention; product errors remain visible.
- Passed `npm run tauri build -- --debug --no-bundle` after importing the Visual
  Studio SDK environment. Passed
  `cargo run -p pantrypal --example phase7_spike --features tauri/custom-protocol --locked`
  using bundled React, production IPC and isolated data/profile: offline
  planning/shopping, Settings labels/navigation, export, native file review,
  required confirmation, replacement/recovery and reopen. Resource URLs were
  local and the shell CSP blocked an explicit remote fetch. Household data was
  not used. New remote-command denials pass real Tauri ACL integration with the
  OS runtime mocked; the reviewed capability manifest remains local-only.
- `npm audit --json` passed with zero vulnerabilities. `cargo-audit 0.22.2 audit
--json` exited 1 for RUSTSEC-2023-0071 (`rsa 0.9.10`), with informational
  RUSTSEC-2024-0429 (`glib 0.18.5`). Windows dependency trees, including bundled
  protocol features, confirm neither affected package is compiled. Findings and
  re-audit obligations are documented; no blanket advisory ignore was added.
- Resolved verification failures: initial fixture identity/merge expectations,
  one UI label mismatch, missing SDK environment on an initial build, a Rust
  async iterator lifetime error, Windows clipboard contention, sandbox native
  launch restrictions, shared WebView2 profile conflict and missing bundled
  assets in the first example run. Final checks passed. Example-only icon and
  Chromium shutdown warnings did not affect acceptance.
- No Phase 7 blocker remains. Backups currently require matching format/schema,
  are unencrypted and retained until manually removed. Fault injection verifies
  rollback, not physical power loss. This is an accessibility smoke check, not a
  complete screen-reader audit. Installer, clean-machine, signing, GitHub CI and
  final release acceptance remain unverified Phase 8 work; its checklist is unchanged.

## Phase 8 — Validation and distribution

**Goal:** A reliable Windows deliverable for its intended user.

**Scope extension (2026-10-13):** The owner requested WorDrop-style automatic
updates: a bounded startup check of the PantryPal GitHub release feed, release
notes, Not now, explicit download/install, progress and signature verification.
Core usage remains offline. PantryPal retains its local-only IPC/CSP boundary
and creates a household recovery backup before handing off to the installer.

- [x] Implement signed Tauri 2 updates from the fixed PantryPal HTTPS release feed, following WorDrop's startup/prompt/progress workflow.
- [x] Add accessible update review/postpone/retry states, Settings check and recovery backup before installation; handle offline startup without interruption.
- [x] Verify updater ACL, newer-version handling, invalid signatures, failed downloads, cancellation/postponement and household data preservation.
- [x] Configure a dedicated PantryPal signing key, version consistency checks and tag-triggered Windows GitHub release workflow automatically publishing NSIS artifacts/signatures/latest.json.
- [x] Verify an installed old-to-new signed update and document signing-key custody, release prerequisites and rollback.

- [x] Run `cargo fmt`, `cargo clippy`, Rust tests, TypeScript lint/typecheck, frontend tests and production build.
- [x] Test isolated local install, app restart, migrations, and upgrade from previous seeded data.
- [ ] Test installer and happy path on a clean Windows machine — pending, explicitly deferred by the owner.
- [x] Verify GitHub Actions signing-secret check, Windows release build and draft asset uploads.
- [ ] Verify published update feed and a production-feed UI upgrade — pending external release acceptance.
- [x] Exercise native Windows end-to-end: ingredient → meal → custom field → plan → shopping → copy.
- [x] Exercise native Windows conversion/ambiguity cases, list override regeneration and backup restore.
- [x] Test authorized import/login flow or clearly document unavailable integration and fallback.
- [x] Package Windows installer (Tauri/NSIS or equivalent) and note any signing warnings.
- [x] Document install, uninstall, local-data directories, backups and how to reset website sessions.
- [x] Resolve critical defects and capture known limitations in `README.md`.
- [x] Tag a first usable local version once the MVP core is verified; publication and clean-machine acceptance remain pending.

**Acceptance gate:** Installer runs on a clean Windows environment and the main
happy path succeeds without developer intervention. A reviewed signed update can
upgrade an installed version without losing household data; unavailable updates
never block offline use. External release/clean-machine checks remain unchecked
until actually executed.

**Phase log (2026-10-13):** Implemented signed updates, local-only review/install
commands, startup/postpone/progress/retry UI and Settings check, dedicated key,
pre-install recovery backup, per-user NSIS packaging, stable version checks and
draft GitHub release automation. DESIGN.md intentionally includes the requested
startup check; ADR 0011 records architecture and SDK limits. README and
`docs/releases.md` cover installation, data retention, sessions, key custody and
recovery. Full commands/evidence and audit findings are in
`docs/phase8-validation.md`.

- Passed frontend format/lint/typecheck and 75 tests, Rust format/Clippy and
  59 unit/11 integration tests, stable version check and locked signed production
  NSIS build. Actual SDK tests reject tampering, wrong signed versions and failed
  downloads; actual IPC denies remote updater and generic SDK commands.
- Passed native bundled acceptance with production IPC, exact servings,
  regional/ambiguous/incompatible units, real Windows clipboard, preserved
  manual items/overrides, file review/confirmed restore, recovery and reopening.
  Deterministic scripted Windows acceptance plus UI tests implements the planned
  manual smoke checks; no comprehensive human/screen-reader audit is claimed.
- Passed isolated NSIS installation/restart, signed SDK 0.1.0 → 0.1.1 upgrade,
  exact exported-table preservation and uninstall retaining data. Fixtures use
  their own app identity/profile; production household data was not touched.
- Passed npm audit (zero vulnerabilities). RustSec audit exits 1 for the existing
  RSA advisory and reports GTK/unmaintained informational findings; dependency
  trees confirm those packages are absent on Windows. No advisory was ignored.
- Clean-machine acceptance is pending at the owner's explicit instruction:
  “Use isolated local testing for now; leave clean-machine acceptance pending.”
  Live Actions/feed acceptance and signing-secret custody are also pending;
  no secrets, commits, tags or releases have been pushed/published. Keep Phase 8
  In progress until its remaining external gates actually pass.
- Created and verified the annotated local `v0.1.0` tag at implementation commit
  `33dc847`. Its annotation explicitly states that clean-machine/published-feed
  acceptance is pending; no tag or release was pushed or published.

**Release workflow follow-up (2026-10-13):** The owner pushed `v0.1.0`.
Verified GitHub run `38385585450` succeeded, including its signing-secret check,
quality checks, signed installer build and draft uploads of the setup executable,
signature and `latest.json`. Corrected the unsupported `uploadUpdaterJson` input
to the pinned action's documented `includeUpdaterJson`; its default is already
true, so the warning did not prevent those uploads. Updated the existing release
configuration test. Passed focused release tests (2), ESLint and formatting.
The published “Initial release” currently has no assets; review the action-created
draft containing the three uploaded files before publication. Published-feed/UI
upgrade and clean-machine gates remain pending; no release was changed or tag
moved by this follow-up.

**Automatic publication follow-up (2026-10-13):** At the owner's request,
changed `releaseDraft` to false, matching WorDrop: pushing a stable matching
version tag runs the existing quality/signing/build gates and automatically
publishes the release with NSIS installer, signature and `latest.json`.
Updated release instructions, ADR 0011's initial draft policy and the existing
workflow regression test. Passed `npm test -- tests/release-config.test.ts`
(2 tests), `npm run lint`, focused Prettier check and `git diff --check`.
No application code changed, so Rust tests/build were not rerun. No tag was
moved, workflow dispatched or release published by the agent. A new tag must
include this workflow change; the old v0.1.0 tag retains its draft behavior.
Published-feed/UI upgrade and clean-machine acceptance remain pending, so the
phase dashboard remains In progress.

**Version 0.1.1 preparation (2026-10-13):** Updated npm, Cargo and Tauri versions
and both lockfiles; dependency versions are unchanged. Updated release docs and
made the SDK same-version fixture follow the package version, while explicitly
pinning the installed-upgrade example's old fixture identity to 0.1.0. Verified
`npm run version:check -- v0.1.1`, `npm run check` (75 tests),
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings` and
`cargo test --workspace --locked` (59 unit/11 integration tests). The commit is
ready for the annotated local v0.1.1 tag and includes automatic publication.
Production packaging/publication will run when the owner pushes the tag; no
remote workflow was dispatched here. Phase 8 checkboxes/dashboard are unchanged.

**CI consolidation (2026-10-13):** At the owner's request, removed the separate
Windows quality workflow after the owner reported a failed test there alongside
a successful release. The release workflow retains `npm run check`, Rust format,
Clippy and workspace tests before packaging/publication. README now documents
release CI and local development checks. No test was removed or changed, and
the reported separate-run failure was not investigated in this slice. Phase
checkboxes and dashboard are unchanged; clean-machine/updater acceptance remains
pending. Passed `npm test -- tests/release-config.test.ts` (2 tests), focused
Prettier checks and `git diff --check`. Reviewed the remaining release workflow's
four quality/test commands; Rust checks were not rerun for this workflow deletion.

**UX/UI design review (2026-10-13):** At the owner's request, inspected the
five primary sections and 30 authored screen/state families through current
components and six historical Windows captures. Created ten built-in imagegen
proposal boards and an offline review gallery at `design/index.html`, including
screen-by-screen findings, priorities, exact prompts and artwork limitations.
Corrected generated unit/reference and editor semantics before delivery. These
are review artifacts; no app behavior, data semantics or architecture changed.
Browser discovery returned no available browser, so live interaction and native
rendering of the proposals are not verified. Historical screenshots contain
earlier-phase copy; current source is the behavior baseline. The stale
measurement aggregation copy is identified for a future implementation slice.
Passed `npm run typecheck`, focused `npx prettier --check` for the gallery,
prompt JSON and PLAN.md, inline Node image/link/JSON validation, and
`git diff --check`. Python validation could not launch due to a Windows logon
session error; Node supplied the completed artifact check instead. Rust tests,
native build and live UI checks were not run for this artifact-only change.
Release acceptance checkboxes and the phase dashboard remain unchanged.

**Approved UX/UI implementation (2026-10-13):** The owner approved the design
review and requested implementation. Completed the shared shell and all approved
screen families in React/CSS, keeping Rust quantity/data rules and native security
boundaries. ADR 0012 records explicit planner confirmation, mounted settings drafts,
progressive disclosure and the intentional correction of stale measurement copy.

- [x] Apply the approved visual hierarchy across Week, Meals, Shopping, Ingredients
      and Settings, including recipe/import editors and supporting management flows.
- [x] Preserve source wording, regional units, unknown/false values, aliases,
      shopping adjustments, warnings, history and existing confirmation semantics.
- [x] Require explicit planner Add and protect unsaved edits when navigating/cancelling;
      retain settings drafts between task panels and block leaving pending writes.
- [x] Verify automated UI flows and disclosure keyboard behavior; inspect the native
      Windows UI at default and minimum sizes with isolated sample data.
- [x] Save actual implemented-screen captures and verification notes in
      `design/implementation/index.html`, linked from the original proposal gallery.

Passed `npm run check` (78 tests/20 files), `npm run build`,
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --features tauri/custom-protocol --locked`
(59 unit/11 integration passed; two existing opt-in tests ignored),
`npm run tauri build -- --debug --no-bundle`,
`cargo build --example ui_review --features tauri/custom-protocol --locked`,
local gallery link/PNG checks and `git diff --check`. Windows initially locked
an example executable during a test rebuild; stopping only the isolated review
runner resolved it and the rerun passed. Browser discovery had no available
surface; native WebView2 screenshots provided visual review. Full screen-reader
and every-state/DPI coverage, and native-close draft protection are not verified.
No release was published. Release acceptance checkboxes/dashboard remain unchanged.

**Documentation split (2026-10-13):** At the owner's request, separated
user installation, workflows, backups, updates and data safety into README.md,
and source setup, build/check commands, architecture, release references and
isolated acceptance harnesses into DEVELOPMENT.md. Corrected the README's stale
meal-picker instructions to match DESIGN.md, ADR 0012 and the current explicit
Add action. The completed Phase 0 documentation checkbox now names DEVELOPMENT.md;
its completion state and the phase dashboard are unchanged. No product behavior
or release acceptance gate changed.

Passed `npx prettier --check README.md DEVELOPMENT.md PLAN.md`, inline Node
validation of 25 local documentation links/anchors and audience separation, and
`git diff --check`. Application tests/builds were not run for this documentation-only
change.

**Meal ingredient alignment follow-up (2026-10-13):** Fixed the owner's reported
vertical misalignment by moving Find or add ingredient out of the three-column
Ingredient/Amount/Unit row. The disclosure now sits below the controls and can
expand without shifting their alignment. No quantity, matching or persistence
behavior changed.

- [x] Align meal ingredient controls and verify the native Windows editor;
      save the actual capture in `design/implementation/ingredient-alignment.png`.

Passed `npm run check` (78 tests/20 files), `npm run build`,
`cargo build --example ui_review --features tauri/custom-protocol --locked`,
focused Prettier checks and `git diff --check`. Rust domain tests and release
packaging were not rerun for this presentation-only change. Release acceptance
checkboxes and dashboard remain unchanged.

**Version 0.2.0 preparation (2026-10-13):** At the owner's request, updated
npm, Cargo and Tauri versions and both lockfiles to 0.2.0; dependencies are
unchanged. Release documentation now uses the matching v0.2.0 tag. The release
commit is prepared for an annotated local v0.2.0 tag.

Passed `npm run version:check -- v0.2.0`, `npm run check` (78 frontend tests),
`npm run build`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (59 unit and 11 integration tests passed;
two opt-in/child tests ignored) after importing the Visual Studio environment,
and `git diff --check`. Installer packaging, remote publication and production-feed
upgrade were not run for this version-only slice. Phase 8 checkboxes and dashboard
remain unchanged; clean-machine and production-feed acceptance remain pending.

### Import unit matching follow-up (2026-10-16)

- [x] Recognize common measurement abbreviations, singulars and plurals in all
      import formats, including lb/lbs/pound/pounds, tbsp and tsp.
- [x] Default unqualified regional units to US in mixed/US mode and use metric
      cups/spoons in metric mode; preserve explicit source units, amounts and raw text.
- [x] Verify the supplied Philly Cheesesteak Casserole ingredient measurements
      through website import preview with persisted mixed, US and metric settings.

Replaced the short import prefix list with a shared Rust whole-token matcher.
Import review explains the applied default; Settings and README describe it.
Removed the former cup review preference control, retaining its stored value
for backup compatibility. DESIGN.md and ADR 0013 document this intentional change
from mandatory ambiguity resolution. No numeric conversions, migrations, registry
changes, dependencies or remote browser/capability changes were introduced.

Passed `npm run check` (formatting, lint, strict type checking, 78 frontend tests),
`npm run build`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (61 unit and 11 integration tests; two existing
opt-in/child tests ignored), focused documentation Prettier checks and
`git diff --check`. Cargo commands used `scripts/Enter-DevEnvironment.ps1`.
An initial workspace run failed in the unrelated updater socket fixture with
Windows WouldBlock/connection-reset errors; the complete final rerun passed.
The Python executable could not launch; Node completed the source edits.

The example fixture contains ingredient measurement facts and synthetic instructions,
not a full publisher recipe. Live native browser capture, installer packaging and
release publication were not rerun. No blocker remains for unit matching; the
phase dashboard and pending Phase 8 external acceptance gates remain unchanged.

**Piece fallback follow-up (2026-10-16):** At the owner's request:

- [x] Default unresolved or missing imported units to Piece in every measurement
      mode, preserving source wording, review notes and unknown quantities.

Updated the shared Rust import parser, review warning, regression expectations,
README, DESIGN.md and ADR 0013. Verified unknown unit names, bare ingredients,
package-size wording, missing/to-taste/range amounts and all three modes of the
Philly Cheesesteak Casserole fixture. Passed `cargo test --workspace --locked`
(62 unit and 11 integration tests; two existing opt-in/child tests ignored),
`cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`, focused
`npx prettier --check` for changed documentation and `git diff --check`.
Frontend checks and native packaging were not rerun for this Rust/docs-only
follow-up. Existing saved recipes are unchanged; no migration is needed.
No blocker remains, and the phase dashboard/release acceptance gates are unchanged.

**Version 0.2.1 preparation (2026-10-16):** Updated npm, Cargo and Tauri
versions and the npm/Cargo lockfiles to 0.2.1, with matching release documentation.
Formatted the existing pnpm lockfile to unblock the release formatting gate;
verified it matches a formatting-only transformation of HEAD. Dependencies are
unchanged. Pinned the updater test's mock app to 0.1.0 so its signed 0.2.0
fixture remains an upgrade across future app versions. The initial Rust suite
failed because that fixture was older than the real app; the final suite passed.
The release commit is prepared for an annotated local v0.2.1 tag.

Passed `npm run version:check -- v0.2.1`, `npm run check` (78 frontend tests),
`npm run build`, `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo test --workspace --locked` (62 unit and 11 integration tests passed;
two existing opt-in/child tests ignored), and `git diff --check`.
Cargo commands used `scripts/Enter-DevEnvironment.ps1`. The initial frontend
check failed only on the pnpm lockfile formatting, corrected before the final run.
Installer packaging, remote publication and production-feed upgrade were not run.
Phase 8 checkboxes and dashboard remain unchanged; clean-machine and
production-feed acceptance remain pending.

## Future ideas — explicitly out of MVP

- [ ] “What should we eat?” recommendations based on family ratings, last eaten, prep time, seasonality, and variety.
- [ ] One-click weekly meal suggestions and manual reshuffle.
- [ ] Enhanced shopping history and favorite manual items.
- [ ] Optional nutrition information or pantry inventory.
- [ ] Optional second-device synchronization if there is a real household need.

## Agent completion protocol

For every substantive task: (1) implement a small vertical slice, (2) add/update tests, (3) run appropriate checks, (4) update the exact PLAN.md checkbox and dashboard, (5) document deviations/ADRs and remaining blockers, and (6) summarize changed files and verification results. **No checkbox for planned-but-unverified code.**
