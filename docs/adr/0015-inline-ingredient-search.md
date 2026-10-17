# 0015: Search and create ingredients within the meal editor

Date: 2026-10-17

Status: Accepted — owner-requested UI change.

The owner requested a fuzzy ingredient field that adds missing ingredients and
accounts for typos. The previous editor split selection, filtering and creation
between a dropdown and a separate dialog. DESIGN.md requires explicit confirmation
of uncertain identities; approximate similarity cannot silently merge ingredients.

Replace the dropdown with a keyboard-accessible search field in both ordinary
meal editing and import review. Rust ranks normalized canonical names and aliases:
exact, prefix, substring, then edit distance including adjacent transpositions.
Queries of at least three characters allow one edit; seven or more allow two.
Return at most twenty canonical records, excluding retired ingredients. These
are suggestions, never automatic assignments. Original imported wording, amounts,
units, preparation and optional flags remain available.

Arrow keys and Enter select a suggestion; Escape dismisses the list. Enter alone
reuses an exact canonical/alias match. With no suggestions, Enter creates and selects
the typed ingredient without a dialog. If possible matches exist, the owner must
select one or explicitly choose Add as a new ingredient. Typing or leaving the
field alone does not create catalog data; unresolved typed wording remains in
the meal draft. Failed searches prevent inline creation until a successful search.

Creation rechecks exact identities, including retired names, through the existing
validated IPC and uses existing transactional CRUD/unique constraints. New records
have no guessed category, section, unit, aliases or conversions. Creation writes
immediately to the catalog, as the existing Add ingredient dialog does. Cancelling
the meal does not remove those catalog records; the field explains this behavior.
Meal saving and leaving the editor are blocked while creation is pending. The
full ingredient dialog remains available for entering additional metadata.

The new read-only search command is allowed only for the local main window.
Remote recipe windows receive no additional capabilities. No dependency, schema,
import matching policy or shopping calculation changes are needed.
