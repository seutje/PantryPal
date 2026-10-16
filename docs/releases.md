# Windows installation and releases

Install the x64 setup executable on Windows 10/11 with WebView2. Installation is
per user and does not need administrator access. Microsoft Edge normally supplies
WebView2; first installation can download its runtime if absent. That prerequisite
is still pending validation on a clean Windows machine. PantryPal itself needs no
internet for ingredients, recipes, planning, shopping or backups.

Installers currently lack Authenticode publisher signatures, so Windows may show
an unrecognized publisher/SmartScreen warning. Updater signatures protect the
downloaded installer separately; they are not Windows publisher certificates.
Only run setup artifacts from the owner's reviewed release.

Release builds check once on startup. An available update shows plain-text notes,
**Not now**, and **Download and install**. Save any open recipe/ingredient draft
before installing. Not now/Escape postpones without changing data; Settings →
Application updates can check again. Offline checks never interrupt startup.
Downloading, verifying and backing up must succeed before the installer starts.
PantryPal closes at handoff; the passive Windows installer updates and restarts
it. If an error appears, continue using the app and retry later. Development
builds/browser preview do not contact the production feed.

Data lives in `%APPDATA%\ca.pantrypal.desktop`: `pantrypal.sqlite3`, backups and
the dedicated recipe-browser profile. Upgrades preserve this directory. Uninstall
from Windows Settings → Apps; leave **Delete app data** unchecked to retain
household records and sessions. Export a JSON backup and copy it outside this
directory before deliberately deleting data. Never delete a database to repair
a migration failure. See the README's backup/recovery instructions. Settings →
Recipe browser → Clear website sessions removes login state; household backups
exclude website cookies and tokens.

## Building and publishing

```powershell
npm ci
npm run version:check
npm run check
. ./scripts/Enter-DevEnvironment.ps1
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
npm run release:build
```

The installer and signature are in `target/release/bundle/nsis/`. Production
version 0.2.1 is consistent in npm, Cargo and Tauri configuration. For a later
release, update all versions (including the npm lockfile), verify, commit and
create a matching `vX.Y.Z` tag. Push that tag to trigger checks, signing, installer
build and automatic GitHub Release publication, matching WorDrop. For example,
after committing consistent versions for a new release:

```powershell
git tag -a v0.2.1 -m "PantryPal 0.2.1"
git push origin main
git push origin v0.2.1
```

The tag must point to the commit containing the new versions and release workflow.
No separate draft publication is needed; pushing the tag authorizes publication.

A dedicated PantryPal private key was created locally at the ignored
`.release-secrets/pantrypal.key`; its public key is in Tauri configuration. The
local key currently has an empty password. Back it up securely outside the
repository before relying on updates, restrict access, and retain an offline
copy. Never commit it, paste it into an issue, print it in build logs, or regenerate
it for an existing installed user base. Losing it prevents compatible updates;
changing the public key requires a deliberately planned transition.

Local builds read `TAURI_SIGNING_PRIVATE_KEY` (contents or path), falling back to
that ignored key, and optional `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Configure the
same private key contents in the PantryPal repository's GitHub Actions secret
`TAURI_SIGNING_PRIVATE_KEY`; optionally set its password secret. No repository
secrets have been uploaded by the agent. The owner's initial GitHub release run
passed the signing-secret check and signed build. Tag-triggered `.github/workflows/release.yml`
validates versions/checks and creates a **published** release containing the NSIS
executable, `.exe.sig` and `latest.json`. Manual dispatch must select the matching
release tag, not a branch. The updater manifest input is `includeUpdaterJson: true`
for tauri-action v0.6.2. Complete release acceptance before pushing the tag, then
check the Actions result and released assets/feed afterward. Clean-machine testing
remains pending by the owner's instruction; automatic publication is now explicitly
requested. No live release has been published by the agent. The initial workflow
used drafts and successfully uploaded setup, signature and `latest.json` to one.
This change affects tags containing the updated workflow; rerunning the old
`v0.1.0` tag still uses its original draft configuration. For the existing release,
the owner can publish the draft containing the files. An Actions artifact ZIP is
separate from public release assets.
The repository and release assets must be publicly readable for this feed;
private-repository authentication is not implemented. Do not embed a GitHub token
in the app or update manifest.

Do not edit `latest.json` to pretend an older installer is newer: signed versions
must match. If a release is faulty, publish a corrected higher version. To recover
locally, export current data, close PantryPal, retain the database and WAL/SHM
companions, and restore the pre-update recovery JSON in a matching-schema app.
For a schema change, use the migration recovery SQLite snapshot and compatible
app version as documented in README. An older binary may reject a newer schema;
do not force a downgrade or overwrite your only recovery copy.

## Isolated Windows acceptance

`phase8_spike` exercises bundled UI/production IPC with fixture storage, including
ingredient → custom field → meal → plan → shopping → actual Windows clipboard,
manual/override refresh, file review/restore and reopening:

```powershell
npm run build
. ./scripts/Enter-DevEnvironment.ps1
cargo run -p pantrypal --example phase8_spike --features tauri/custom-protocol --locked
cargo test -p pantrypal --test updater --locked
```

For installer testing, build two isolated identity variants using temporary JSON
configuration overrides (`productName: PantryPalAcceptance`,
`identifier: ca.pantrypal.acceptance`, versions 0.1.0 and 0.1.1). Copy each setup
and signature to `.verification/phase8-installers/`. Production versions are not
changed. Use the dedicated signing key and normal locked NSIS build arguments.
Then build `cargo build -p pantrypal --example updater_install_spike --locked`
and explicitly run `scripts/Test-Installer.ps1`.

That script refuses an existing acceptance installation/profile, installs only
the acceptance identity, launches/restarts its executable, seeds fixture records,
and uses the real SDK to download/verify a signed newer NSIS artifact from an
owned loopback server, save a recovery snapshot and perform a quiet installer
handoff. It verifies all exported tables after upgrade and after uninstall.
Loopback and quiet mode are test-only; production uses HTTPS and passive mode.
It retains evidence/profile under `.verification` and
`%APPDATA%\ca.pantrypal.acceptance` for review. This is isolated local testing,
not clean-machine acceptance or a production-feed UI update test. Clean-machine
acceptance remains pending by the owner's explicit instruction.
