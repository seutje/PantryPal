# ADR 0011: Signed Windows updates and release acceptance

Date: 2026-10-13. Status: accepted implementation; clean-machine and published-feed
acceptance pending.

The owner extended Phase 8 with the WorDrop updater workflow. This intentionally
adds one startup network check to the offline-first baseline. We reviewed
WorDrop's local updater and release workflow and the current official
[Tauri updater documentation](https://v2.tauri.app/plugin/updater/), then tested
the installed SDK and Windows installer with isolated fixtures.

Use Tauri's maintained Rust updater SDK, a dedicated PantryPal signing key and
the fixed `seutje/PantryPal` GitHub release feed. Do not reuse WorDrop's key.
Only the public key is committed. Require the signed version as well as the
artifact signature, preventing an endpoint from relabelling an older genuine
installer as a newer release. CLI 2.12.2 records versions in signed comments;
SDK 2.13.3 verifies them. Only newer semantic versions are offered.

React reviews plain-text notes, postpones with Not now/Escape, shows progress
and explicitly requests installation. A cached startup promise supports React
StrictMode without losing the result or making duplicate checks. Development
and browser preview disable checks; installed release checks fail silently at
startup. Settings provides a deliberate retry and visible safe errors.

The shell receives two individually granted commands. The native state retains
the reviewed Update; the UI supplies only its version, consent and a progress
channel. No general updater plugin IPC permissions are granted. Remote pages
remain unable to invoke either command. Downloads require a credential-free
HTTPS GitHub URL under PantryPal's release path and `.exe` extension. Redirects
are limited to five and exact GitHub/CDN HTTPS hosts; private/IP endpoints and
other origins are rejected. System proxy discovery is disabled. Checks have a
15-second timeout and downloads 120 seconds; download progress aborts beyond
128 MiB. A busy guard serializes checks/install attempts and resets on failure.

The SDK verifies bytes before the application exports a complete JSON recovery
backup. Failure to download, verify or create that backup prevents installer
handoff. SQLite commits remain durable; do not close its pool before launching
because a failed handoff must leave the running app usable. The modal makes the
underlying UI inert during installation. Unsaved drafts require choosing Not now
and saving before installation. The SDK exits after successful Windows installer
launch; NSIS upgrades in place using passive mode and restarts the application.
No automatic install, scheduled retry or download occurs after postponement.

Release automation validates consistent stable project/tag versions, runs checks
and builds per-user NSIS installers, signatures and `latest.json`. The owner
subsequently requested WorDrop-style automatic publication (2026-10-13): pushing
a matching stable version tag now authorizes the workflow to create a published
release and upload its assets, using `releaseDraft: false`. This supersedes the
initial draft/review policy; no separate publish click is required. CI secrets/key
custody and the live feed cannot be verified by a local build. Authenticode remains optional
and currently absent; a valid updater signature does not suppress SmartScreen.

Limits: the SDK parses the fixed trusted feed before the application's 1 MiB
manifest/16,000-byte notes check. These are post-parse validation, not a streaming
manifest memory cap. The deadline bounds time; signed binaries and origin checks
protect installation, but unusually large trusted-feed responses can temporarily
allocate memory. Revisit if the SDK adds bounded manifest reads. Redirect host
validation is not a custom DNS resolver: HTTPS certificate checks and the fixed
GitHub allowlist apply. Household schema rollback still requires a compatible
version and its recovery backup. There is no automatic downgrade.

The owner explicitly requested isolated local testing and leaving clean-machine
acceptance pending. Acceptance installer variants use `PantryPalAcceptance` and
`ca.pantrypal.acceptance`, with version overrides in ignored files; they never
replace the household product. Owned loopback HTTP is enabled only in SDK fixture
tests/examples. The production configuration has no insecure flags or dynamic
endpoint inputs. These tests prove local installer/SDK behavior, not a fresh OS,
published GitHub feed, production signing-secret custody or screen-reader audit.
