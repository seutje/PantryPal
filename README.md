# PantryPal

An offline-first Windows desktop meal planner and grocery-list generator for a
Canadian household. Keep recipes, plan the week and shop with practical quantities
while preserving original recipe wording. Household data stays on your computer;
no account is required.

## Install on Windows

PantryPal requires Windows 10/11 x64 and the Microsoft Edge WebView2 Runtime.
Download the setup executable from [PantryPal releases](https://github.com/seutje/PantryPal/releases)
and run it. Installation is per user and does not require administrator access.
The installer can download WebView2 if it is missing.

Windows publisher signing is currently absent, so setup may show an unrecognized
publisher or SmartScreen warning. Clean-machine installation and the published
update feed remain pending acceptance checks. See
[Windows installation and releases](docs/releases.md) for installation, recovery
and known limitations.

For source setup, builds and tests, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Everyday use

- **Meals:** create or import recipes, retain original amounts and unknown quantities,
  and add custom yes/no, choice, number, date, text or rating fields for filtering.
- **Week:** plan a Monday-start week with multiple or repeated meals and independent
  fractional servings. Record cooked meals explicitly; scheduling alone never
  changes last eaten. Cooking history supports backfill, date correction and undo.
- **Shopping:** generate a grouped list for the selected week. Refresh after recipe
  or plan changes; manual items, overrides, checkmarks and exclusions are preserved.
  A stale list requires refresh before copying plain text to the Windows clipboard.
- **Ingredients:** manage names, aliases, categories, shopping sections, season
  months, notes, preferred units and ingredient-specific conversion rules.
- **Settings:** choose measurement preferences, convert temperatures, order shopping
  aisles, manage backups, clear website sessions and check for application updates.

Recipes, planning, shopping and backups work offline. Website browsing and update
checks need an internet connection.

## Adding meals to a day

In Week, choose **Add meal** on a day, or **Add meal to week** for an unscheduled
meal. Search and filter the meal library, then select a meal card with the mouse
or keyboard. Review its planned servings and destination, then choose **Add planned
meal**. Selecting a card alone does not save. The picker closes after saving;
each addition creates a separate occurrence, including repeated meals.

Servings start at the recipe's base servings. **Edit servings** changes only that
occurrence, and **Edit occurrence** also allows changing the day. Expand
**Ingredients for … servings** to see scaled amounts; the saved recipe and other
occurrences retain their servings. Refresh Shopping to use the new totals.
Escape or Cancel closes the picker without adding a meal; failed saves retain
the search and offer a retry.

## Shopping quantity displays

Shopping quantities round upward to readable measures while keeping their exact
calculated totals and overrides. Each row uses just one metric scale: g/mL below
one kilogram/litre, kg/L at or above it. Volume rows also
show clearly labelled US, metric and Imperial cup/fluid-ounce alternatives.
Copied grocery text includes the same alternatives. Unknown amounts remain
unknown, and alternatives never assume an ingredient density. For example, an
exact `59 1176473/8000000 mL` requirement displays as **60 mL**, with alternatives
including **¼ US cup** and **2 US fl oz**. See
[ADR 0009](docs/adr/0009-practical-shopping-measures.md) for the rounding policy.

## Recipe importing and website sessions

On Meals, choose **Import recipe**, paste a public HTTPS recipe URL and choose
**Check website**. **Open recipe website** opens a separate WebView2 window;
sign in on the website itself. Its local profile retains sessions best-effort.
Cross-origin redirects, popups, downloads and device permissions are blocked. If
a login, MFA or page needs another origin, use your regular browser instead.
Close the previous recipe window before opening a different recipe.

For Budget Bytes, Allrecipes, Spend With Pennies, RecipeTin Eats, NYT Cooking and
Cook Smarts (`mealplans.cooksmarts.com`),
return to PantryPal once the recipe is visible, confirm permission and choose
**Read recipe from browser**. The authorized adapter reads visible recipe text
into a preview. Alternatively enter permitted recipe text, or paste an authorized
schema.org Recipe JSON-LD export/object (without script tags).
Unqualified cups and spoons default to US variants in mixed/US measurement mode
and metric variants in metric mode. Other unqualified regional units default to
US; abbreviations and plurals such as `lb`, `lbs`, `pound`, `tbsp` and `teaspoons`
are recognized. Unresolved or missing units default to **Piece** for review.
Recognized source units and original amounts are preserved.
Check servings and resolved regional units,
review canonical ingredient suggestions and confirm saving. Unknown quantities,
ranges and package sizes retain their original wording. Imported HTML markup is
inert text. The saved meal remains editable; its original import payload,
attribution, source and timestamp remain in a durable import record.

For a missing ingredient, choose **Add ingredient** beside its ingredient
selector in recipe review. The modal saves the ingredient to your catalog and
selects it on that recipe line, retaining quantities and original wording.
Existing name/alias matches require an explicit **Use existing** choice.
Ingredients saved this way remain in the catalog even if you later cancel the
recipe. The Ingredients page's **Add ingredient** uses the same modal; Cancel
or Escape closes it and restores focus when no save is in progress.

Duplicate source URLs require an explicit **Save another meal** choice and never
overwrite an existing recipe. Failed saves retain the draft; identical retries
reuse the original action ID. Cancel/discard does not create a meal.

**Clear website sessions** is available in Import and Settings. Confirming it
closes recipe browsing and clears cookies, cache, local storage and saved browser
data in `%APPDATA%\ca.pantrypal.desktop\recipe-browser-profile`. Recipes and other
household data are unchanged. Clearing waits for WebView2 completion; an error
does not claim success. Website passwords/tokens are never collected by the app.

Hidden/gated content cannot be imported, even if recipe metadata exists. Site
layout changes or cross-origin login prompts may require manual entry. Subscriber
login/MFA has not been verified. Cook Smarts supports the supplied recipe layout,
including `?version=standard`, `gf`, `paleo` or `vegetarian` on recipe URLs.
Keep the URL's version aligned with the displayed recipe, and expand Prep and
Make before reading. Ingredient sections, optional items and source wording
remain in review. Divided amounts such as `1 Tbsp + 2 Tbsp` and ranges remain
unknown until reviewed; enter the intended total explicitly. Check sections
marked for multiple nights before planning. Active/Total times and selected diet
remain source notes rather than guessed prep/cook durations.

## Backups and everyday settings

In **Settings → Backups and restore**, choose **Export JSON backup**. The
completed path appears on screen. Files are stored under
`%APPDATA%\ca.pantrypal.desktop\backups`; copy them elsewhere with Windows Explorer
for off-device protection. They contain personal household data in plain JSON,
including retired recipes, original quantities, custom fields, plans, cooking
history, shopping adjustments/manual items, import provenance and preferences.
Website profiles and login sessions are excluded. Files remain until you remove
them; there is no automatic retention cleanup.

To restore, choose a JSON file, select **Merge** or **Replace**, then **Review
backup**. Merge adds new IDs and keeps identical records; conflicts in recipes,
catalogs, settings or references stop the entire import. It does not overwrite
or remap IDs. Replace restores the entire snapshot, removing current records
absent from it. Review the counts and confirmation, then choose **Confirm merge**
or **Confirm replacement**. Cancel leaves household data unchanged. A recovery
JSON backup is saved before either mode writes; backup failure stops the import.
If household data changes after review, review again before retrying.

Only format 1 / schema 8 JSON backups are currently supported, up to 50 MiB and
100,000 rows. Old/future or damaged files are rejected without resetting data.
Use a matching application version for other schemas. To undo a successful
restore, review and replace with its displayed recovery JSON file.

Pending database upgrades automatically create a consistent
`backups/before-migration-*.sqlite3` copy before migrating. These are native SQLite
files, not JSON imports. For manual migration recovery: close every PantryPal
process, preserve the current database and any WAL/SHM files in a separate safety
folder, then put a copy of the chosen recovery file at `pantrypal.sqlite3` in the
app-data directory with no old WAL/SHM companions. Open it with the matching
application version. Never overwrite your only recovery copy. Ask for help if
the compatible version or file is uncertain.

Settings also saves measurement conventions/formatting, shopping aisle order
(keyboard-accessible Up/Down controls), and website session clearing. Clipboard
URL prefill is off by default. Enabling it reads a bounded public HTTPS URL only
when **Import recipe** is clicked; it never monitors the clipboard or opens a
website automatically. Invalid clipboard contents leave manual entry available.

## Application updates

Installed release builds check for updates once at startup. Review the notes,
save open drafts, then choose **Download and install** or **Not now**.
**Settings → Application updates** offers a manual check. Offline use remains
available. Updates verify signatures and signed versions, and save a household
recovery backup before launching the installer. PantryPal closes for installation
and restarts afterward. If an error appears, continue using the app and retry later.

To uninstall, use Windows Settings → Apps. Leave **Delete app data** unchecked to
retain household records and website sessions. Export a JSON backup and copy it
elsewhere before deliberately deleting app data.

## Local data

PantryPal stores household data in
`%APPDATA%\ca.pantrypal.desktop\pantrypal.sqlite3`. Backups and the separate
recipe-browser profile also live in this app-data directory. SQLite's `-wal`
and `-shm` companion files may be present during use. Never delete the database
to repair a migration error.

If storage is unavailable or incompatible, the app displays an error and never
silently resets data. Use Settings to export and restore household JSON backups
as described above. For a manual database safety copy, close all PantryPal
processes and copy the database and any remaining WAL/SHM companions; do not copy
only the main file while the app is writing. Keep website profiles separate from
household backups.
