# Phase 8 verification record

2026-10-13, Windows x64. The owner requested isolated local testing and leaving
clean-machine acceptance pending. No household database/profile or WorDrop files
were changed; no release/tag was pushed or GitHub secret configured.

Passed:

- `npm run check`: Prettier, ESLint, strict TypeScript and 75 frontend tests.
- `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- `cargo test --workspace --locked`: 59 unit and 11 integration tests passed.
  The Windows clipboard unit test is normally ignored; this phase exercised
  the real production clipboard command through the native acceptance harness.
  The restart parent executes its normally ignored child in separate processes.
- `npm run version:check -- v0.1.0`.
- `npm run release:build`: locked production Rust build and signed NSIS installer
  at `target/release/bundle/nsis/PantryPal_0.1.0_x64-setup.exe` (4.66 MiB), with
  `.exe.sig`. Signing proves updater integrity, not Authenticode identity.
- `cargo test -p pantrypal --test updater --locked`: actual SDK newer/same/older
  version handling, signed fixture download, altered bytes, wrong signed version
  and failed download. Actual IPC rejects remote check/install and both remote
  and local generic SDK check; unconfirmed install changes no data.
- `cargo run -p pantrypal --example phase8_spike --features tauri/custom-protocol --locked`:
  bundled WebView2 UI and production IPC; ingredient, typed false field, recipe,
  planned servings (500 g / 4 × 2 = 250 g), shopping, Windows clipboard, retained
  override/manual item after refresh, Settings labels/navigation, file review,
  confirmed restore/recovery, reopening, local-only resources and blocked shell
  network fetch. Native commands distinguish metric/US/Imperial cups and preserve
  ambiguous cups and incompatible mass/volume/pieces. Settings's development check
  reports disabled updates without accessing a feed. Deterministic scripted native
  acceptance plus the UI suite implements the planned Windows happy-path smoke
  checks; this is not a comprehensive human or screen-reader audit.
- Built 0.1.0 and 0.1.1 NSIS acceptance variants using ignored config overrides
  with a separate product/identifier and the real dedicated key. Built
  `cargo build -p pantrypal --example updater_install_spike --locked`, then ran
  `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/Test-Installer.ps1`:
  actual install/start/restart, seeded data, signed SDK download/version verification,
  pre-handoff backup, quiet SDK installer handoff, installed 0.1.1 startup, exact
  replacement of the executable with product version 0.1.1 in the original
  registered installation directory, comparison of every exported table and
  uninstall preserving data. This fixture
  tests the SDK/NSIS handoff separately from the production UI/origin wrapper;
  it does not assert that the live production feed/UI upgrade was exercised.
- `dumpbin /dependents target/release/pantrypal.exe`: Windows system/API CRT DLLs;
  no external VCRUNTIME/MSVCP DLL dependency was observed. A fresh OS may still
  need WebView2; that installation remains unverified.
- `npm audit --json`: zero vulnerabilities after rerunning with registry access.
- `git diff --check`; ignored signing key/generation log and acceptance config
  verified with `git check-ignore`.

Dependency audit: `cargo-audit 0.22.2 audit --json` refreshed RustSec and exited 1
for [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html)
(`rsa 0.9.10`). Informational findings:
[RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
(`glib 0.18.5`) and
[RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html)
(`proc-macro-error 1.0.4`, unmaintained). `cargo tree --locked --target
x86_64-pc-windows-msvc -i <package>` finds none of these in the Windows graph;
RSA/glib also absent with `--features tauri/custom-protocol`. No advisory ignores
were added. The cross-platform lockfile is not claimed vulnerability-free.

Resolved check failures: missing TypeScript declaration/tuple call types, native
dialog cancel simulation in jsdom, missing Node globals imports in the version
script and an initial sandbox registry audit failure. Artifact review caught a
mixed slash/backslash path in the SDK acceptance harness that installed the newer
fixture elsewhere; the fixture was retained as ignored evidence, the path was
corrected, executable-version/installation-directory assertions were added, and
the entire installer test was rerun successfully. Production uses no custom
installer directory argument. Final checks passed. Native
example-only icon/Chromium shutdown warnings did not fail acceptance.

Pending: clean Windows acceptance (explicitly deferred), GitHub signing-secret
custody/setup, actual Actions execution and published-feed UI upgrade. Authenticated
Cook Smarts extraction is unavailable; the documented secure browser/manual fallback
and five authorized adapters remain covered by fixtures/IPC tests. No site login,
paywall or household session was used in these release checks. Manifest size is
checked after SDK parsing; see ADR 0011 for the bounded-time/memory limitation.
Private key is ignored and currently unencrypted; its owner must secure and back
it up before publishing updates. Local evidence remains in ignored `.verification`
and the isolated acceptance app-data directory for review.

Verified implementation commit: `33dc847`. Annotated local tag `v0.1.0` points to
that commit and explicitly records the pending external acceptance checks. The
subsequent plan-only commit records the tag after its creation/verification.
