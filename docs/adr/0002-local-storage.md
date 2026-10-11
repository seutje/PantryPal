# ADR 0002: Local SQLite storage and durable records

Date: 2026-10-11

Status: Accepted

## Context

Phase 1 must establish durable storage without implementing the measurement engine, planner, grocery generator or import browser. Recipe wording, household edits and history must survive restarts and schema upgrades.

## Decision

Use SQLx 0.8.6 with only the SQLite, Tokio runtime, migration and macro features. This established SQLx release supplies embedded migrations, parameter binding and bundled SQLite; adopting the newer 0.9 query API is unnecessary for this slice. No frontend SQL plugin is installed. SQLite's bundled C library requires the MSVC/Windows SDK environment; `scripts/Enter-DevEnvironment.ps1` imports it for local builds and CI.

Resolve `app.path().app_data_dir()` in Rust and place `pantrypal.sqlite3` there. On Windows this is `%APPDATA%\ca.pantrypal.desktop`. UI commands never accept database paths, SQL or table names. Enable foreign keys, WAL, a five-second busy timeout and full synchronous writes. A single managed connection serializes household operations; aggregate reads and multi-row writes use transactions.

Four numbered embedded migrations establish catalog, meals/fields, later workflow storage and the seed catalog. SQLx's migration ledger and checksums gate reopening; unknown future migrations and changed scripts fail safely. Initialization failure remains visible through storage error states while the shell stays usable. Never recreate, delete or silently downgrade an existing database. These migrations are additive; a future destructive migration requires a backup and a separately reviewed strategy.

Store recipe quantities and servings as exact integer numerator/denominator pairs, retaining the entered pair, unit identifier, raw wording and source convention. Missing quantities remain SQL NULL. IPC amounts are bounded to nonnegative numerators and positive denominators of at most one billion; servings require a positive numerator. Phase 2 will add parsing, checked arithmetic and conversion, rather than moving calculations into TypeScript.

The unit registry stores exact affine factors relative to grams, millilitres or Celsius, with distinct metric/US/Imperial variants. Count/informal units have no universal factor. The modest seed is 16 common groceries, six categories, seven shopping sections and 36 explicit units, with a few deterministic aliases and no density rules or regional season assumptions. It is migration 4, applied once; startup does not overwrite renamed, edited or retired seed records. New seed additions must use new migrations, never edit migration 4. No demo meals are inserted into household data.

Canonical names and aliases share a normalized identity namespace enforced by uniqueness constraints and cross-table triggers. Normalization trims/collapses whitespace and lowercases; it does not apply fuzzy matching or invent plural rules. Retired records keep their names reserved. Category and section vocabularies are durable, independent data.

Ordinary deletion is retirement with an explicit confirmation flag. Ingredients, meals, vocabularies and custom fields remain available to existing references; foreign keys restrict physical removal. Pickers in later phases will filter retired options, while historical labels remain readable. New records cannot choose retired references, but existing recipes and ingredients remain editable when their references have been retired.

Custom field values use tagged JSON validated in Rust against their definition. SQL NULL is unknown and a tagged boolean false remains false. Stable option IDs allow label changes. Phase 1 permits safe definition renaming and retirement, while rejecting kind, bound, option-ID or unit changes that require a future migration/confirmation workflow. Meal updates preserve omitted custom fields, including retired definitions; an explicit null changes an active value to unknown. Meal revisions reject stale updates. No default assignment behavior is introduced here.

Planning, cooked history, grocery overrides, import provenance and settings have storage tables only. Their workflows remain in the assigned phases. In particular, scheduled meals never create cooked events, and no shopping regeneration or authenticated extraction is implemented. Dates use local ISO calendar dates; stored audit timestamps use UTC. Cooking and plan semantics are finalized in Phase 4.

The native capability explicitly grants only named, validated catalog/meal/field repository commands to the local main window. It still grants no remote-origin, generic filesystem, SQL, shell or HTTP access. Full editing screens remain in Phases 2–3. A read-only ingredient catalog, local-data status and a global startup error state make Phase 1 storage visible in the desktop app.

## Verification and consequences

Tests cover empty/existing database migration, checksums/future versions, failed migration rollback, corrupt-file preservation, seed idempotency after edits, transactional CRUD rollback, foreign keys, retirement, typed fields and null/false, stored unknown quantities, and a write/read test executed in separate child processes. The deterministic meal fixture is `tests/fixtures/demo-meal.json` and is used only by tests/development.

This phase has no backup/export UI or destructive database operation. The database is unencrypted local household data protected by the Windows user account. Do not copy only the SQLite main file while the app is running; WAL may contain committed data. Verified export/backup belongs to Phase 7. Runtime isolation of remote pages remains Phase 6 work.

References: [SQLx 0.8.6 migrations](https://docs.rs/sqlx/0.8.6/sqlx/migrate/struct.Migrator.html), [Tauri app-data resolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html#method.app_data_dir), [SQLite foreign keys](https://www.sqlite.org/foreignkeys.html).
