# Phase 7 security and usability audit

Reviewed on Windows x64, 2026-10-13. Scope is everyday backup/preferences and the
existing local core/import boundary. Release and clean-machine checks are Phase 8.

## Security boundary

- Seven new named commands are individually registered and granted only to the
  local `main` window. There are no remote capability origins, wildcard command
  grants, arbitrary SQL/path/HTTP commands or general clipboard access.
  `tests/security-config.test.ts` verifies the full manifest/CSP;
  `src-tauri/tests/data_ipc.rs` rejects every new command from a remote origin
  using Tauri's actual ACL with its OS runtime mocked.
- Recipe URLs require HTTPS and reject credentials, queries, fragments, custom
  ports, IP addresses and local host forms. Native recipe browsing retains the
  existing navigation/popup/download/permission restrictions and authorized
  visible-content adapters. Rust does not fetch recipe URLs, so no native HTTP
  proxy or redirect fetch surface was introduced. Arbitrary public DNS names
  are not resolved/classified by the URL validator; remote content remains
  unprivileged even if DNS changes. Existing Phase 6 real-WebView isolation and
  session-reset verification remains the runtime evidence for that boundary.
- Website sessions remain in the dedicated WebView2 profile. Only household
  SQLite tables enter JSON exports; a sentinel cookie fixture verifies profile
  exclusion. No credentials, clipboard contents or imported payloads are logged
  by product code. Error messages are normalized and do not echo untrusted URL
  secrets. Explicit successful backup paths are intentionally shown in Settings.
- Backup input is bounded, versioned and validated in isolated SQLite before
  replacement. Supported unit definitions cannot change. Transactions and
  pre-change recovery copies preserve existing records on storage failures.
  Snapshot checksums/signatures and encryption are not provided; a backup is
  untrusted input, not an authenticated document.

## Dependencies

`npm audit --json`: **passed, zero vulnerabilities**, including development
dependencies. No project dependencies or lockfile versions changed in this phase.

`cargo-audit 0.22.2 audit --json`: **exit 1**, with RustSec database updated
2026-10-12. The full lockfile has:

| Advisory                                                                                                   | Locked package | Windows applicability                                                                        |
| ---------------------------------------------------------------------------------------------------------- | -------------- | -------------------------------------------------------------------------------------------- |
| [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), Marvin RSA timing attack       | rsa 0.9.10     | Not compiled in the Windows SQLite feature graph; no RSA private-key operation in PantryPal. |
| [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), informational unsound iterator | glib 0.18.5    | GTK/Linux dependency; not compiled for Windows.                                              |

Verified with `cargo tree --locked --target x86_64-pc-windows-msvc -i rsa` and
the same command for `glib`: both report nothing to print. SQLx explicitly uses
`default-features = false` with SQLite. Findings were documented rather than
globally ignored or optional lockfile entries manually removed. Audit results
are not a claim that the entire cross-platform lockfile is vulnerability-free.
Review these findings again when changing targets/features/dependencies.
The audit tool and raw output are verification artifacts under ignored
`.verification/`, not application dependencies.

## Usability and offline checks

The UI uses semantic buttons, labelled inputs/fieldsets, status/error regions,
local Segoe UI/system fonts, a skip link and visible 3 px keyboard outlines.
Navigation adds Arrow/Home/End focus movement without automatically changing
pages. Existing modal Escape/return-focus behavior remains tested. Aisle order
has labelled Up/Down controls; imports require a reviewed confirmation. Failed
saves keep drafts; meal search now has a retry action. New frontend flows test
keyboard interactions, confirmation/cancel, failed file review and opt-in prefill.
Dark stone/green text on white/light backgrounds and the green focus outline
remain the shared styles; this smoke review is not a full screen-reader audit.

The `phase7_spike` example runs actual bundled React assets and production IPC
with a dedicated shell profile and temporary household database. It verifies
offline plan/shopping generation, Settings labels/navigation, exported JSON,
file review, confirmation, replacement, recovery and reopen. Core resource URLs
must be local, and an explicit remote fetch must be rejected by the shell CSP.
It does not change the computer's network adapter or household database.

Resilience tests cover malformed/oversized/old/future JSON, bad references,
normalized identities, unsupported unit edits, exact integer bounds, invalid
typed data, stale review, merge conflicts, migration backup failure and a fault
injected mid-restore. The transaction retains the complete original snapshot.
This is deterministic interruption/rollback testing, not physical power-loss
testing. A 200-meal/21-occurrence fixture exercises search, grouped generation,
export and staged validation within a generous 20-second budget.
