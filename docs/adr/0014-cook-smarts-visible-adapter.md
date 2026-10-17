# ADR 0014: Cook Smarts visible recipe adapter

Date: 2026-10-17. Status: Accepted. Supersedes ADR 0007/0008's manual-only
Cook Smarts decision for the supplied layout; their browser security boundary remains.

The owner supplied saved authenticated HTML and explicitly requested an adapter.
Implement that request using the established fixed host-side capture script,
with the existing user confirmation of permission and mandatory review before save.
The request is not evidence of a publisher agreement or a guarantee of permitted
live automation. No login/access control is bypassed, and no new browser API,
network fetch, IPC permission or dependency is introduced.

Only the exact host `mealplans.cooksmarts.com` is registered. Its recipe URLs may
include exactly one `version=standard`, `version=gf`, `version=paleo` or
`version=vegetarian` query on `/days/<numeric id>` or `/recipes/<id-slug>`.
Other queries, credentials and fragments remain rejected. Rust compares the
complete requested URL before and after capture; the fixed script compares
origin, path and query during capture. Selecting a different URL variant requires
the requested source to match it. Remote pages still have no capabilities.

Read the visible `day-title`, current servings, ingredient lists and section
headings, and the single visible cooking method's Prep/Make lists. Hidden rows,
collapsed blocks, missing content and ambiguous methods fail with the existing
manual fallback. Hidden alternate methods are excluded. Read only visible
Active/Total timing and selected diet into source notes: these are not equivalent
to prep/cook durations. Cook Smarts capture does not inspect JSON-LD or application
state, account fields, reviews, CSRF values, cookies or credentials.

Rust parses Cook Smarts' `ingredient - quantity, preparation` ordering while
retaining the captured source line, fractions, optional markers and section notes.
Measurement preferences select regional units through the shared matcher.
Compound/divided and range quantities remain unknown with an explicit review
warning; never silently use their first amount. Multi-night section quantities
are retained exactly, with review required rather than guessing which portion
belongs to another meal. Canonical ingredient matches remain exact and reviewable.
Existing preview/transaction/provenance and duplicate policies apply unchanged.

Fixtures reproduce the supplied DOM structure using synthetic instructions and
selected ingredient facts. The private full saved page is not committed. Test
capture exclusions and failures, URL policy, all measurement modes and confirmed
save/reopen. Saved HTML inspection can verify structure, not actual WebView2
layout, authenticated login or publisher access. Those live checks remain open.
