# ADR 0010: Reviewed household backups and everyday settings

Date: 2026-10-13

Status: Accepted

## Decision

Settings exports a logical JSON snapshot of all household tables, including
retired records, raw recipe values, import provenance, typed fields, cooking
history, action receipts, shopping documents/adjustments and preferences. Browser
profiles, cookies, credentials, migration bookkeeping and arbitrary files are
excluded. The envelope is `PantryPal`, format version 1, schema version 8, with
ordered tables, columns and scalar rows. SQL identifiers come from the compiled
table registry and migrated schema; imported values use bound parameters.

Only the current format/schema is accepted. Older/future files require a matching
application version; no speculative conversion or partial recovery is attempted.
Limits are 50 MiB and 100,000 rows. An isolated migrated database checks actual
constraints, references, supported unit definitions and domain values before
household writes. Supported seeded unit definitions cannot change; legacy units
may remain alongside them.

Historical recipe/shopping temperature assignments are preserved with their
existing warnings, without inventing grocery conversions. New shopping overrides
still require valid grocery units. Integer columns reject fractional numbers and
out-of-range unsigned values instead of relying on SQLite affinity coercion.

Merge adds new stable IDs and retains identical existing records. Creation/update
timestamps are ignored when comparing duplicates and current timestamps remain.
Conflicting values, revisions, settings, normalized names or references reject
the entire merge. IDs are not remapped and fuzzy matches are not combined.
Replace restores the entire household snapshot, including settings and shopping
adjustments. Both modes require review and explicit confirmation. A state token
checks for changes since review again inside the commit transaction; it is a
staleness guard, not an authentication or cryptographic integrity mechanism.

Exports and pre-restore recovery files use unique backend-owned paths under
`app_data/backups`. JSON is written to a new `.partial` file, synchronized, then
renamed. A recovery write failure stops restore before deletion. Household
replacement runs in one SQLite transaction. Pending upgrades of an existing
migrated database first use SQLite `VACUUM INTO` to create a consistent native
recovery copy; failure stops migration. These native copies are recovered with
the matching application version, rather than the JSON import interface.

Backups are retained until the owner removes them. No automatic cleanup silently
discards recovery data. Settings shows the fixed folder and completed file path;
users can copy files elsewhere with Windows Explorer. Backup imports use the
native file input and pass bounded contents to Rust, never an arbitrary path.
No schema migration or new project dependency is needed for this phase.

## Everyday settings

Aisle order is durable `shopping_section.sort_order`, updated transactionally
after verifying that every active section appears exactly once. Existing
measurement formatting and website-session clearing remain available.

Clipboard prefill defaults off. The local command checks the persisted opt-in
before accessing Windows, reads only on an explicit Import click, bounds UTF-16
data to 8 KiB and returns only a validated public HTTPS URL. It never polls or
opens the URL automatically. Failures retain the manual import path. The reader
follows Microsoft's [GetClipboardData contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getclipboarddata)
and [GlobalSize bounds](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globalsize).

## Consequences

Backups preserve data exactly rather than attempting selective recipe migration.
Conservative merge can reject independently edited catalogs/settings; explicit
replacement remains available after review and a recovery copy. Plain JSON
contains personal household information and is not encrypted. Retention and
off-device copies are the owner's responsibility. Installer/clean-machine and
release validation remain Phase 8.
