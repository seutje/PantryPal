# AGENTS.md — Instructions for Codex and other coding agents

These instructions apply to the entire PantryPal repository. Read **[DESIGN.md](DESIGN.md)** and **[PLAN.md](PLAN.md)** before changing product behavior or architecture. Treat them as the baseline; if code and docs disagree, stop and identify the discrepancy, then document an intentional decision rather than silently changing scope.

## Mission and priorities

Build **PantryPal**, a reliable, personal, offline-first Windows desktop meal planner and grocery-list generator for a Canadian household. The chosen stack is **Rust + Tauri 2 + React + TypeScript + SQLite**. Prioritize these, in order:

1. Correctness and data safety, especially quantities and shopping calculations.
2. Security of local data and authenticated external recipe browsing.
3. Fast, simple user workflows with accessible UI.
4. Small maintainable modules and useful tests.
5. Visual polish and optional enhancements.

Do not add cloud services, accounts, telemetry, AI dependencies, sync, subscriptions or extra product features unless explicitly requested.

## Source of truth and workflow

- `DESIGN.md`: product scope, data rules, architecture, security requirements.
- `PLAN.md`: execution phases, acceptance gates and live progress checklist.
- `AGENTS.md`: constraints and agent working practices.
- `docs/adr/`: create short ADRs for impactful architectural choices or unresolved technical feasibility.

**At the start of a work session:** inspect repository state and relevant files; identify the next unchecked, unblocked task in PLAN.md; state the intended slice if communicating with the user. Do not redo completed items without reason. Prefer completing a small end-to-end feature to introducing scaffolding across many features.

**At the end of a slice:** run focused tests plus relevant formatting/lint/type checks, report the exact checks run and failures, update checkboxes and phase dashboard **only for verified work**, and note blockers in PLAN.md. Never claim an unexecuted test passed. Do not mark a whole phase done until its acceptance gate holds.

## Architectural boundaries

- **React:** presentation, input collection, state and keyboard interactions. Use typed APIs; do not implement authoritative shopping/measurement rules only in TypeScript.
- **Rust domain/services:** business invariants, measurement conversion, recipe scaling, shopping grouping, field validation and matching.
- **Persistence:** all SQLite operations via repositories/migrations/parameterized SQL. Preserve referential integrity and perform multi-row changes in transactions.
- **Tauri commands:** narrow, validated, least-privilege commands. Never offer arbitrary SQL, shell, filesystem paths or HTTP proxies to UI/remote pages.
- **Importers:** isolated adapters that return untrusted draft data for user review. Importers do not write directly into production meal records.

Keep dependencies proportionate. Prefer stable ecosystem packages, explicit errors and focused modules. Avoid giant god-objects, hidden singleton state, magical heuristics or speculative abstractions.

## Non-negotiable domain invariants

- Original recipe values, unit identifiers and raw text must remain available after normalization/conversion.
- Distinguish metric, US and Imperial variants where they are not equivalent, especially cups and fluid units.
- No implicit mass↔volume or piece↔mass conversion without an applicable ingredient-specific rule; never use a universal food-density assumption.
- Count units (`can`, `package`, `piece`) are not universally convertible. Unknown/to-taste amounts must survive import and shopping calculations.
- Do not silently merge fuzzy ingredient matches; use canonical ingredient IDs and require confirmation when uncertain.
- Shopping generated quantity must reflect every plan entry and its servings exactly once.
- Preserve manual shopping items and user overrides on regeneration per documented policy. Never silently erase user changes.
- A meal is “last eaten” only when an actual cooked event is recorded, not merely when scheduled.
- Custom field data is typed and validated; unknown is distinct from false for booleans.

## Security and recipe-import rules

External recipe websites are **untrusted origins**. Protecting the native application is more important than extraction convenience.

- Never enable privileged Tauri APIs or broad IPC permissions for remote website pages. Audit `capabilities/`, window labels and allowed origins whenever browser code changes.
- Browser sessions can use a stable dedicated WebView2 profile when supported; never collect site passwords or log tokens/cookies. Provide a user-accessible session reset.
- Verify Tauri 2 APIs and WebView2 limitations through official current documentation and an isolated spike **before** implementing cross-origin extraction; do not assume a convenient API exists.
- Do not bypass website logins, paywalls, MFA, CAPTCHAs, anti-bot controls, or terms. Only build site-specific automation where permitted. Cook Smarts integration is conditional, not a mandate to circumvent restrictions.
- For URLs/downloads, validate protocols and destination, handle redirects, cap payload sizes/time, and block private/local endpoints when fetching from Rust. Escape/sanitize imported text and never render arbitrary source HTML with active scripts.
- If safe authenticated extraction cannot be achieved, ship an explicit browser-assisted/manual import fallback and document the limitation.

## Data and migration discipline

- Use SQLite in the per-user app data directory, not the project folder or arbitrary working directory.
- All schema changes use numbered/reproducible migrations, with forward-compatibility considerations and tests on existing data.
- Back up before destructive operations. Never discard recipes, custom fields or shopping adjustments without explicit confirmation.
- Store ingredient aliases and user-defined categories as durable data, not hardcoded frontend enums.
- Favor stable IDs and indexed normalized lookup columns; use parameterized queries only.
- Keep export formats versioned, avoid leaking auth cookies into backups, and verify restore correctness.

## UI conventions

- Target Windows desktop usage; design with React, TypeScript strict mode, Tailwind and accessible primitives.
- Main sections: Week, Meals, Shopping, Ingredients, Settings. Prominent Import action on Meals.
- Optimize for few-click workflows and understandable defaults; complex options belong in progressive disclosure.
- Good keyboard support, consistent focus styling, descriptive error/empty states, clear unit labels and visible approximation/ambiguity badges where meaningful.
- Respect user's unit display preferences while preserving raw/source recipe wording.
- Do not make image assets, remote fonts, analytics or an internet connection mandatory for core usage.

## Tests required for relevant changes

At minimum, add automated tests for:

- Measurement parsing (`1/2`, `1 1/2`, unicode fractions), precision and conversion round-trips.
- US/metric/Imperial cup distinctions, incompatible dimensions, and ingredient-specific approximations.
- Portion scaling, duplicate plan entries, ingredient alias matching and shopping grouping.
- Preservation of shopping overrides and manual additions across regeneration.
- Custom field type validation and null/false semantics.
- SQLite migrations and restart persistence.
- Importer parsing of saved allowed-site fixtures and rejection of dangerous inputs.
- External website page unable to access privileged Tauri commands (via reviewed capability configuration and feasible automated checks).

Prefer deterministic fixtures over flaky live website tests. For UI, test the major vertical flows and keyboard accessibility as tooling permits.

## Commands and verification

Use the scripts and tooling actually present in the repository, not imagined commands. Typical checks after setup may include:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run lint
npm run typecheck
npm test
npm run tauri build
```

Run these from the correct workspace directory and adapt to real `package.json` scripts. On non-Windows systems, call out Windows-specific build/run checks as **not verified**. Do not modify lockfiles casually or introduce dependencies without a clear need.

## When requirements are ambiguous

Follow the specific behavior in DESIGN.md. For important unspecified behavior, choose the safest simple default, document it in a short ADR, and avoid irreversible data actions. Ask the owner only when an unresolved decision would materially alter user experience, security, legal compliance or persisted data semantics.

## Completion summary format

For each task/PR, provide:

- **What changed:** concise behavior summary and key files.
- **Checks:** exact commands and pass/fail/not-run results.
- **Plan:** which checkboxes/dashboard entries were changed.
- **Risks/open items:** unresolved questions, blockers, follow-up tasks.

Never present a speculative importer, untested measurement conversion or incomplete integration as finished. Keep the app usable after each phase.
