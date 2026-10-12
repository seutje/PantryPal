# ADR 0006: Exact grocery generation and preservation on refresh

Date: 2026-10-11. Status: accepted.

[ADR 0009](0009-practical-shopping-measures.md) refines shopping presentation
with upward rounding and alternative measures. Exact generation, source data
and preservation rules below continue to apply.

## Quantities and identity

Rust generates a selected Monday week. Every planned or cooked occurrence,
including unscheduled occurrences, contributes once; skipped occurrences do not.
Cooking does not automatically remove groceries already bought. Optional lines
remain included and visibly labelled. Each source retains the occurrence, meal,
original line, base servings and planned servings.

Checked i128 rational arithmetic scales recipe quantity × planned/base servings.
Totals cross IPC as numerator/denominator strings; decimal rounding is display
only. Overflow fails the entire transaction. Unknown amounts are never zero.
Unresolved canonical identities, missing units, informal measures and unknown
amounts remain separate occurrence/line rows with explanations. Fuzzy aliases
are never grouped. Historical temperature assignments remain separate, labelled
as non-grocery quantities and never scaled with servings. Ingredient aliases
only match when they resolve to the same canonical ID. Different count units
require an applicable ingredient rule.

For numeric canonical lines, choose a deterministic compatible anchor: g, mL,
then other registered unit IDs in lexical order. Only unique universal or
single ingredient-rule conversion paths qualify. The aggregation preference can
instead restrict grouping to identical source units. Group identity depends on
canonical ingredient and anchor, independent of display preferences and ordering.
An ingredient's preferred shopping unit takes precedence over Metric (g/mL) or
US (oz/explicit US cup) preferences; Mixed keeps the anchor. An unavailable
preference falls back with an explanation. Every original source must have a
unique **direct** conversion to the display target: anchor conversion must not
accidentally chain ingredient rules. Approximation rule IDs are retained.

## Refresh and editing

Read operations never generate or change saved amounts. A versioned source
fingerprint covers plans, meal revisions/data, catalog, rules, units, section
order and measurement preferences. Differences show “needs refresh”; copying
requires refresh first. Fingerprints conservatively include the full catalog,
so an unrelated catalog edit can also request refresh.

Explicit refresh matches stable generation keys and retains amount overrides,
checkmarks and exclusions. Manual items persist unchanged. Unknown overrides
are explicit, distinct from no override. A confirmed reset removes an override.
Exclude/include is reversible; no shopping deletion command is provided.

When a generated key disappears, unadjusted unchecked included rows are removed
as derived output. Rows with overrides, checkmarks, exclusions or legacy data
remain visible as orphaned rows, omitted from copy. Reappearance restores their
edits. Changes to conversion rules or aggregation can split/merge identities:
edits are retained on orphaned rows instead of guessed onto new groups. Users
can add a manual item to carry an orphan quantity into their current shop.

All source reads, generation, merge and writes share one SQLite transaction.
Every edit uses a list revision plus durable action receipt. Exact retries
return current state without repeating the action; reused IDs with another
payload and stale revisions fail. Validation failures retain editable UI drafts;
uncertain write failures retain the exact action for retry or explicit reload.

## Versioned persistence and existing data

Migration 8 adds a revision and versioned document JSON to `shopping_list`.
This stores typed sources, string rational totals, unresolved rows and
adjustments without rebuilding the earlier strict shopping table or discarding
records. Documents are generated snapshots; authoritative recipe and catalog
data stay in their relational repositories. New references are validated against
the transaction's catalog. Existing records are retired rather than deleted.
Unsupported document versions fail closed.

Earlier `shopping_item` rows remain intact. Their basis and override JSON had
no implemented format: a compatibility adapter carries every row/flag/field
into a manual review row, retains its original payload verbatim and labels its
quantity for review. The first explicit edit saves this carried document. It
does not guess a numeric quantity from legacy JSON.

## Clipboard and security

The backend produces deterministic section ordering and item ordering, CRLF
plain text and checkmarks. Excluded/orphaned rows are omitted. Labels collapse
whitespace so embedded newlines cannot fabricate list structure. Approximate,
rounded and optional markers are included. Text is rendered inertly in React.

Three commands (get/change/copy shopping) participate in the existing local-main
ACL. Copy accepts a week and revision, reads saved backend text, and writes
CF_UNICODETEXT through a small Windows module; it exposes neither generic
clipboard reads nor arbitrary writes. Native allocation/locking and ownership
follow [Microsoft's clipboard contract](https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-the-clipboard).
Allocation precedes clearing, failures close/free owned resources, and a busy
clipboard returns a visible retryable error. No new dependency or remote-origin
permission is needed. Copy has a 4 MB text cap; it does not truncate silently.

The normal suite avoids changing a user's clipboard. An explicitly ignored
Windows acceptance test generates and edits a temporary recipe plan, writes its
text, reads CF_UNICODETEXT back exactly, and launches a separate Windows Forms
text control to paste while the fixture writer remains alive. This covers
fractions and CRLF without relying on clipboard stability between verification
jobs. IPC tests exercise command
deserialization, real SQLite and remote denial with a mock OS runtime; frontend
tests exercise the Copy action. Native UI inspection is separate from those
test boundaries.
