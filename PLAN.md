# PantryPal — Implementation Plan

**Progress convention:** Coding agents change `[ ]` to `[x]` **only** when implemented, tested, and verified. Leave incomplete tasks unchecked. Mark a blocked task as `- [ ] ... — BLOCKED: <reason>` and note follow-up in the phase log. Complete phases in order unless dependencies justify parallel work. Follow [DESIGN.md](DESIGN.md) and [AGENTS.md](AGENTS.md).

## Status dashboard

| Phase | Outcome                                               | Status      |
| ----- | ----------------------------------------------------- | ----------- |
| 0     | Repository scaffold and quality gates                 | Complete    |
| 1     | SQLite schema, seed catalog, and core domain          | Not started |
| 2     | Measurement engine and ingredient management          | Not started |
| 3     | Meal library and custom fields                        | Not started |
| 4     | Weekly planner, cooking history                       | Not started |
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

- [ ] Choose SQLite/SQLx integration and local app-data path; enable foreign-key constraints.
- [ ] Create migrations for ingredients, aliases, categories, shopping sections and unit registry.
- [ ] Create migrations for meals, meal ingredient lines, custom field definitions and values.
- [ ] Create migrations for plan entries, cooked events, shopping lists/items, import records and settings.
- [ ] Implement Rust domain models, validators, repository boundaries and typed IPC commands.
- [ ] Create a modest, idempotent seed catalog of common ingredients, units, types and sections.
- [ ] Implement transaction-safe CRUD, referential integrity and safe deletion/retirement behavior.
- [ ] Add migration tests from an empty DB and an existing DB; test process restart persistence.
- [ ] Add deterministic demo/fixture data for UI development.

**Acceptance gate:** A fresh app initializes exactly once, CRUD persists after restart, and the migration suite passes without data loss.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 2 — Units and ingredients

**Goal:** Correct mixed-unit handling, ingredient identity, and practical catalog editing.

- [ ] Implement unit dimension/system registry and parse amount forms including `½`, `1/3`, `1 1/2`.
- [ ] Implement precise mass conversions for mg/g/kg/oz/lb.
- [ ] Implement explicit regional volume variants and conversions (US, metric, Imperial).
- [ ] Implement temperature display conversions separately from grocery quantities.
- [ ] Implement a `cannot convert` result for incompatible or ambiguous dimensions.
- [ ] Add optional ingredient-specific conversions (count/mass, volume/mass, package sizes), approximation metadata.
- [ ] Add tests for round-trips, boundaries, ambiguous cups, fractional scaling and inconvertible cases.
- [ ] Build ingredient browse/search/create/edit screens for categories, shopping sections, aliases, season months, default unit and notes.
- [ ] Enforce canonical alias/name uniqueness and display candidate matches for duplicates.
- [ ] Build measurement preference UI and simple inline conversion control.

**Acceptance gate:** Test vectors in DESIGN.md yield correct results without inventing conversions, and ingredient CRUD works end to end.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 3 — Meals and extensible fields

**Goal:** Create, find and edit meals without imports.

- [ ] Build meal list/detail/editor with title, base servings, times, instructions, notes, tags, and source link.
- [ ] Implement ordered ingredient lines with quantity, unit, ingredient chooser, raw wording, optional flag and preparation note.
- [ ] Support unknown amounts and informal units without discarding information.
- [ ] Create/manage custom field definitions: boolean, select, number, date, text and rating.
- [ ] Implement typed custom-field validation and values, including unknown boolean state.
- [ ] Allow meal search/filter by title, ingredients, custom fields and derived metadata.
- [ ] Implement safe custom-field rename/retire and confirmation for destructive changes.
- [ ] Add tests for meal CRUD, field values, invalid input and ingredient deletion behavior.

**Acceptance gate:** A user can create a complete meal with custom fields, restart, locate it by filters, and edit it without losing data.

**Phase log:** _Add date, commands run, and blockers here._

## Phase 4 — Week planner and history

**Goal:** Select a useful week of meals and record what was actually cooked.

- [ ] Implement Monday-start local weeks, previous/next navigation and unscheduled-week pool.
- [ ] Add/remove/reorder planned meals and choose servings per occurrence.
- [ ] Track plan status (`planned`, `cooked`, `skipped`) with explicit user actions.
- [ ] Record cooked events exactly once per action; implement undo/correction semantics.
- [ ] Derive and display `last eaten` from cooked events; allow backfill if appropriate.
- [ ] Connect planner to meal detail and shopping-list generation action.
- [ ] Test repeated meals, date edges, invalid servings and plan changes.

**Acceptance gate:** The weekly view works across month/year boundaries and only intentionally cooked meals update eating history.

**Phase log:** _Add date, commands run, and blockers here._

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
