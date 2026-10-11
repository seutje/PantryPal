# ADR 0004: Meal drafts, safe fields and library filters

Status: Accepted

Date: 2026-10-11

## Decision

Phase 3 adds a complete meal library, manual editor and custom-field manager. The UI sends quantity strings to a narrow Rust draft command; Rust parses, bounds and validates them before the existing transactional repositories save anything. The editor never calculates recipe quantities in TypeScript. Revisions reject stale saves and errors retain the draft.

Migration 6 adds nullable entered-amount text to meals and ingredient lines. Original wording, source convention, explicit unit IDs, exact rational pairs and stable ordered line IDs remain available. Older rows retain their original pairs and NULL text; an unchanged draft uses those pairs without reducing or rounding them. Raw numeric wording must agree with its quantity. Missing/to-taste quantities remain unknown, and informal units stay explicit. Display rounding follows the saved fraction/decimal preferences and labels rounded results with their exact value.

Field definitions and values use the Phase 1 stable identifiers and tagged Rust validation. Unknown remains the default, including unknown booleans; false and empty known text remain distinct from unknown. Defaults are optional in DESIGN.md and are not assigned automatically. Labels, descriptions, ordering and select-option labels can change safely. Kind, option identifiers, bounds and unit labels remain immutable after creation; the UI explains creating another field for a different structure. Type conversion needs a separate migration strategy and is deliberately unavailable. Confirmed retirement keeps definitions and values readable and filterable; existing retired values remain unchanged when editing a meal. Ingredient-line removal, meal retirement and discarded meal drafts require explicit confirmation.

Library filtering runs in Rust on canonical ingredient IDs, normalized title/ingredient aliases/tags/field text, typed field predicates, and last-cooked local ISO dates. Unknown matches absent or NULL values, never false. Last cooked is the maximum actual cooked-event date. Phase 3 only reads the existing history table; event recording, backfill and planning stay in Phase 4. Household-sized libraries use straightforward repository composition rather than a speculative search index. Filters remain editable when validation fails.

Source URLs remain inert provenance displayed as escaped text. Fetching, external browser opening and imports stay in Phase 6. The two new commands participate in the existing build manifest and local-only main-window capability; remote pages gain no permissions.

## Verification

Tests exercise actual Tauri command deserialization, capabilities and SQLite for a meal with all six field types, original fractions, unknown amounts, ordering, retirement, invalid edits, stale revisions and reopen. Separate-process restart tests locate a saved recipe using combined filters and edit it without losing data. Repository tests cover last-cooked date filters, scheduled entries not implying cooking, legacy unreduced quantities, migrations and raw-text consistency. React flows cover editing, keyboard filters, explicit removal/retirement, invalid-filter recovery and failed-save draft preservation. Windows debug build and native screen inspection complete the local acceptance checks; these do not replace Phase 6 external-WebView testing or Phase 8 release validation.
