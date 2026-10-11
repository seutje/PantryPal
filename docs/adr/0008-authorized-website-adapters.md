# ADR 0008: Authorized adapters for five recipe publishers

Date: 2026-10-11. Status: Accepted. Supersedes ADR 0007's manual-only
publisher decision for the five supplied sites; its security/session boundary
continues to apply.

## Authorization and evidence

The owner stated, “I have deals with the supplied websites, please create
adaptors for them.” This authorizes adapters for Budget Bytes, Allrecipes,
Spend With Pennies, RecipeTin Eats and NYT Cooking. It does not change website
access controls or authorize a Cook Smarts integration.

Before implementation, the Phase 6 native spike proved supported host-side
`eval_with_callback`, persistent isolated WebView2 profiles and remote IPC
denial. The installed Tauri API and
[official documentation](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html)
were checked again. An isolated native structural probe of the supplied pages
identified WPRM cards for three sites, Allrecipes' `mm-recipes` components and
NYT's visible recipe-body/preparation components. The probe retained bounded
selector structures and schema types, not credentials or full publisher HTML.

## Decision

Only the local main window can request `capture_recipe_browser`. The command
uses a fixed bundled script and an exact publisher hostname registry; callers
cannot supply JavaScript, a window label, a profile path or fetch parameters.
It checks the actual browser URL before and after capture, and the script
checks the URL during capture. A canonical trailing slash is tolerated; a
different origin, recipe path, query or fragment fails. Browser open, capture
and reset share an exclusive operation guard.

Adapters read visible ingredient lines, instructions and current yield. Hidden
alternate WPRM units are excluded; hidden ingredient rows or instruction steps,
missing content, ambiguous cards and recognized access gateways cause failure.
Recipe JSON-LD supplies only attribution and timing metadata after the visible
recipe passes checks; hidden recipe ingredients/instructions are never used to
recover gated content. Malformed/unavailable metadata permits a DOM fallback.
All content remains untrusted inert text. Quantities are parsed only by Rust,
and unqualified cups/spoons, unknown amounts and uncertain identities require
review rather than a site-wide regional assumption.

Payloads are capped at 256 KiB in both the capture script and Rust; metadata
scripts, traversal depth, ingredient count/length and instructions are bounded.
The native callback has a ten-second timeout. Capture produces a preview without
writing meals. A versioned `website` envelope records adapter identity and
original captured values. Existing confirmation, duplicate handling, durable
retry and transactional provenance storage are reused without a migration.

Remote windows still have no capabilities. There is no remote event bridge,
Rust HTTP fetch, auth-token copying, hidden application-state extraction or
access-control bypass. Cross-origin sign-in flows/popups remain restricted;
manual paste/entry and confirmed session clearing remain available. Site layout
changes can disable an adapter safely; permission does not guarantee access to
every recipe or successful subscription login.

## Verification

Saved synthetic HTML fixtures reproduce each inspected publisher structure
using original test text, avoiding redistribution of complete publisher recipes.
Tests execute the production capture script, validate Rust drafts, reject gated
and oversized inputs, preserve ambiguous fractions and retain adapter provenance
after review/save/reopen. UI tests verify capture failures retain manual input
and successful captures still require review before saving.

The `adapter_sites` Windows harness uses production open/capture commands with
isolated SQLite/profile storage and visits all five supplied URLs. It records
only counts and pass/fail results, verifies remote capture commands are denied
by actual ACL, and asserts that previewing writes zero meals. It is an explicit
live acceptance check, not a network dependency of the deterministic test suite.
