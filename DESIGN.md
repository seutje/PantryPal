# PantryPal — Master Design

**Status:** Initial product and technical specification  
**Target:** Windows desktop (single-user, offline-first)  
**Stack:** Rust, Tauri 2, React, TypeScript, SQLite  
**Audience:** The owner, future maintainers, and coding agents  
**Related:** [PLAN.md](PLAN.md) · [AGENTS.md](AGENTS.md)

## 1. Product vision

PantryPal is a small, personal Windows desktop app for turning a household's favorite meals into a low-effort weekly plan and a practical grocery list. It is made for one household, not designed as a commercial SaaS product. Favor speed, reliability, pleasant everyday use, and editable local data over scale, accounts, or monetization.

**Core loop:** maintain ingredients → save/import meals → choose meals for the week → calculate and adjust groceries → copy/check off the shopping list → mark meals cooked and retain useful history.

### Primary user goals

1. Quickly find or create a meal and keep its ingredients, amounts, instructions, and useful personal notes.
2. Import recipes from supported websites, possibly behind a login, with explicit user action and a review screen.
3. Select meals for a week without repeatedly doing meal math.
4. Generate a consolidated shopping list grouped into grocery-store sections, including manually added items.
5. Mix Canadian metric, US customary, and other practical recipe units without losing the recipe's original measurements.
6. Define custom meal fields (yes/no, choice, number, date, text, rating) and filter meals by them.

### Non-goals for the MVP

- Cloud accounts, sync, collaborative editing, subscription billing, social sharing, or a mobile app.
- Full pantry inventory or nutrition/calorie tracking.
- Automatic unrestricted scraping of arbitrary websites, bypassing authentication, CAPTCHAs, paywalls, or technical controls.
- AI-dependent recipe parsing or meal selection.
- Guaranteed conversion between ingredient volume, mass, pieces, packages, or informal units when no reliable conversion exists.

## 2. UX and navigation

**Navigation:** Week · Meals · Shopping · Ingredients · Settings. An Import action is prominent on Meals. Design for a Windows desktop with compact, keyboard-accessible, responsive layouts and pleasant empty states.

### Week

- Default to the current local Monday–Sunday week; navigate previous/next weeks.
- Assign one or more meals to a date, or to an unscheduled “this week” pool; assign planned servings per entry.
- Show per-day meal cards with meal name, estimated times, notable custom fields, and status (`planned`, `cooked`, `skipped`).
- “Mark cooked” records a meal-history event. **Last eaten is derived from cooked events** (with a manual backfill/override path, not a field that silently changes just by planning).
- Generate/open a shopping list for the chosen week; avoid double-counting the same planned entry.

### Meals

- Search and filter by title, tags, ingredients, custom fields, and last-cooked date.
- Create and edit title, description, base servings, ingredient lines, free-form multiline instructions, times, notes, source link, and optional tags.
- Ingredient lines support amount, unit, ingredient reference, preparation note, optional flag, ordering, and original source text.
- The ingredient field searches canonical names and aliases with typo-tolerant
  suggestions. Explicitly select uncertain matches. Enter reuses an exact match
  or creates a new catalog ingredient when there are no suggestions; with possible
  matches, choose one or explicitly add a different ingredient. Creation saves
  immediately, even if the meal is later cancelled. See ADR 0015.
- Missing quantity / “to taste” is valid. Display a warning rather than inventing a quantity for a shopping list.
- Recipe content can be edited freely after import. Keep source URL and import provenance.
- Custom field definitions are managed separately; each meal may have values for any applicable definition.

### Ingredients

- Ship a modest **seed catalog**, editable/deletable where appropriate, including common groceries. Prevent accidental duplicate display names.
- Each ingredient has a canonical name, aliases, type/category (e.g. beef, chicken, vegetable), shopping aisle/section (e.g. Meat, Produce, Dairy, Pantry), preferred shopping unit, optional seasonality and notes.
- Type/category and grocery section are independent. Both are extensible user-managed vocabularies.
- Optional seasonality: one or more calendar months (or “year-round”); don't assume regional availability or imply seasons are universally correct.
- Units belong to **recipe ingredient quantities**; ingredient default units are display/input preferences, not a rigid restriction.
- Allow aliases and custom conversion rules per ingredient (e.g. `1 medium onion ≈ 150 g` or `1 US cup flour ≈ 120 g`) with `approximate` indication.

### Shopping

- Generate grocery entries from the plan's ingredient lines, scaled to planned servings.
- Combine only compatible quantities for the same **canonical ingredient** after validated conversion; preserve separate lines with explanatory reasons if incompatible/unknown.
- Group into aisle/section, with customizable order; sort predictably within each section.
- Add manual non-recipe items, edit generated amounts, mark purchased, exclude/remove items from the shopping view, and copy plain text in a convenient predictable format.
- Track generated basis separately from user adjustments. **Regeneration must not silently erase manual items or corrections**; explicitly offer merge/recalculate behavior and communicate changes.
- Do not automatically assume what is already in the fridge; users can reduce quantities manually.
- Example output:

  ```text
  Produce
  [ ] Onions — 3 pcs
  [ ] Broccoli — 500 g

  Meat
  [ ] Chicken breast — 1.5 lb

  Other
  [ ] Toilet paper — 1 pack
  ```

### Settings

- Measurement mode: metric, US customary, or mixed/ingredient-specific (default: mixed).
- Volume conventions: **US cup ≈ 236.588 mL; metric cup = 250 mL; imperial cup ≈ 284.131 mL**. Support named, explicit variants. For unqualified imported cups and spoons, select US variants in mixed/US mode and metric variants in metric mode. Other unqualified regional units default to US. Preserve explicit source variants and raw wording; the review remains editable. See ADR 0013.
- Temperature conversion: °C ↔ °F (for display/recipe instructions; temperature never merges into shopping quantities).
- Recipe display fractions (e.g. `½`, `⅓`), automatic compatible-unit aggregation, and rounding/formatting preferences.
- Shopping section order, data export/import and backups, browser session clear/reset, and importer/site settings.
- Import settings should explain what browser site data is stored locally.

## 3. Custom meal fields
### Approved UI refinement (2026-10-13)

The owner approved the proposals and behavioral notes in `design/index.html`.
Implement a compact forest-green sidebar, task headers, primary/secondary/quiet
actions, agenda day rows, compact aisle checklists, grouped recipe editing,
contextual ingredient editing, task-based settings and guided import review.
The gallery's written notes take precedence over generated artwork errors.
All domain, persistence and remote-origin security rules below remain unchanged.
Meal picking now collects the chosen meal, exact servings and destination before
an explicit Add action; selecting a result alone does not write a plan entry.
Settings sections retain mounted drafts when switching tasks. Original quantity
wording and advanced recipe controls stay available through disclosures.
See ADR 0012 for implementation boundaries and keyboard behavior.


Field definitions have a stable ID, label, kind, optional description/default, options (for `select`), and presentation order. Field kinds:

| Kind | Example | Notes |
|---|---|---|
| Boolean | Kids enjoyed? | Unknown/null distinct from no |
| Select | Difficulty: Easy / Medium / Hard | Options have stable IDs; support safe rename |
| Number | Prep time in minutes | Optional unit/validation |
| Date | Tried on | ISO stored date; local display |
| Text | What to change? | Multiline allowed |
| Rating | Family score 1–5 | Configurable bounds if desired later |

Built-in metadata (e.g. title, servings, prep time, source, derived last eaten) should **not** be duplicated into arbitrary fields. Custom fields are data, never executable code or SQL. Field removal or type changes require confirmation and a safe migration strategy; avoid destructive implicit conversion.

## 4. Measurements and conversion model

### Principles

- **Preserve what the source said.** Save entered numeric value, exact unit ID, optional fraction/raw text, and source convention. Display converted amounts as views, not replacements.
- Unit catalog supports mass (`mg`, `g`, `kg`, `oz`, `lb`), volume (`mL`, `L`, US/metric/imperial `cup`, `tsp`, `tbsp`, `fl oz`, `pint`, `quart`, `gallon` with explicit regional variants), count (`piece`, `clove`, `slice`, `bunch`, `can`, `package`), informal (`pinch`, `dash`, `handful`, `to taste`), and temperature (`°C`, `°F`). Extend registry without schema changes.
- Distinguish US and Imperial fluid ounces, pints, quarts and gallons. Ambiguous imported units must be resolved or retained without unsafe conversion.
- **Universal conversions** only within a compatible dimensional family (mass ↔ mass, volume ↔ volume). Temperature is affine, not multiplicative.
- Count-to-count, count-to-mass, package-to-volume, and mass-to-volume require ingredient-specific conversion rules and may be approximate. A `can` without package size is not a standard volume.
- Never convert 1 cup of flour to grams using water density. Never convert pieces to mass without ingredient metadata. Never sum mass and volume as though they match.
- Support decimal and rational recipe amounts (`1/3`, `1 1/2`) without cumulative floating-point artifacts: parse fractions and store scaled decimal or rational values (e.g. decimal arithmetic in Rust). Use explicit display rounding only at the boundary.
- The shopping generator should prefer each ingredient's preferred shopping unit, then the global preference, then a reasonable compatible source unit; keep unconvertible amounts distinct.
- Preserve an indication if the aggregate uses an approximate conversion.

### Examples

- `500 g chicken` + `1 lb chicken` → one chicken line, rendered using the selected preferred unit.
- `1 metric cup milk` + `250 mL milk` → `500 mL`.
- `2 onions` + `300 g onions` → separate entries unless onion count-to-mass conversion is configured.
- `1 can tomatoes` + `400 g tomatoes` → separate unless the can's net mass is known for that ingredient.
- `salt to taste` → separate nonnumeric line / clear shopping-note behavior.

## 5. Importing recipes

Import is **opt-in, user-initiated**: paste URL → inspect domain → open a browser window → sign in on the source website if required → inspect page data using a permitted integration → preview and normalize → confirm save. Authentication is never handled by an app-owned password form.

### Authenticated browsing and safety

- Tauri 2 on Windows uses WebView2; use a **separate browser window/profile** with a stable user-data directory where supported, so cookies and site storage survive app restarts. Session persistence is **best-effort**; websites can expire sessions, require MFA, or block embedded WebViews.
- Treat arbitrary remote sites as **untrusted**. Never expose Tauri privileged IPC, filesystem commands, database, secrets, or local development endpoints to remote content. Use allowlisted navigation/window permissions and isolate web content from the app origin.
- **Do not assume** that executing scripts against a third-party WebView or harvesting its DOM is exposed as a safe, supported Tauri API. Spike and validate the actual Tauri 2 / WebView2 capabilities before committing to the architecture. Alternatives may include: an explicit browser-assisted copy/import flow, extraction from HTML supplied via a sanctioned mechanism, or an official export/API. Do not sacrifice the security boundary merely to automate importing.
- Limit import URLs to `https://` (with explicit exceptions only if justified); validate hostname and redirects, reject local/private network destinations for any backend fetch; impose timeouts, content-size limits, and sanitization. Never log cookies, tokens, passwords, or raw credential-bearing headers.
- Comply with each site's terms, access controls, and permissions. Do **not** bypass paywalls, CAPTCHAs, or anti-bot measures. Cook Smarts is a **target to investigate**, not a guaranteed supported integration. Seek authorized export/API or permission where necessary.

  Owner-requested follow-up (2026-10-17): enable a Cook Smarts visible-page adapter
  for the supplied authenticated recipe layout on `mealplans.cooksmarts.com`,
  using the existing isolated browser and explicit permission/review flow.
  Only its four known diet-version query values are accepted on recipe paths;
  arbitrary query strings remain rejected. Keep section/preparation notes and
  optional markers, leave divided/range amounts unknown for review, and retain
  Active/Total timing as source notes. See ADR 0014. Live authenticated access
  remains unverified; this does not guarantee site permission or login support.
- Provide a visible clear-session control, per-site failure messages, and a manual recipe-entry fallback.

### Importer pipeline

1. Identify URL/site and check whether integration is supported.
2. Prefer authorized machine-readable recipe data such as `schema.org/Recipe` / JSON-LD when available.
3. Apply specific extractor modules only for approved supported sites; use stable fixtures and versioned tests.
4. Produce `ImportedRecipeDraft`: title, source URL/site, attribution, servings, timing, instruction blocks, ingredient lines (original raw text + parsed amount/unit/name), optional notes.
5. Display a **review screen**: default unresolved or missing units to `piece` (owner-approved import fallback), preserve their original wording for review, resolve aliases and ingredients, choose how to handle duplicates, confirm servings and instructions. Do not invent missing or to-taste amounts.
6. Save normalized local recipe and source provenance. Import failure must leave current data intact.

### Ingredient matching

Normalize case, whitespace, plurals where safe, and common aliases; search canonical ingredients and aliases first. Suggest fuzzy matches, but **require explicit confirmation** for uncertain matches. Never silently merge two differently named ingredients based only on approximate text similarity. Store original ingredient text for traceability.

## 6. Proposed architecture

```text
pantrypal/
  AGENTS.md
  DESIGN.md
  PLAN.md
  src/                         # React + TypeScript
    app/                       # routes, shell, providers
    features/
      week/ meals/ shopping/ ingredients/ import/ settings/
    components/                # shared UI
    lib/                       # typed Tauri invoke wrappers, formatting
  src-tauri/
    src/
      commands/                # narrow Tauri IPC commands
      domain/                  # core entities, validators, measurement engine
      services/                # shopping generation, planning, matching
      importers/               # importer contracts, allowed implementations
      persistence/             # SQLite repositories / migrations
      lib.rs
    migrations/
    capabilities/              # least-privilege Tauri permissions
  tests/                       # app-level fixtures/e2e as useful
```

**Frontend:** React + TypeScript, Tailwind CSS and shadcn/ui (or equivalent accessible components), TanStack Query for async server-like local queries. Use forms with validation; keep frontend business logic thin.  
**Backend:** Rust domain/services with typed Tauri commands and structured error results. Put conversion, aggregation, and validation rules in Rust where unit tests can exercise them.  
**Persistence:** SQLite + SQLx migrations; local app-data location, WAL mode as appropriate, foreign keys enabled, transactional writes and backups. No network service required.

### Conceptual data model

The following are conceptual entities, **not a fixed migration**; codify in Phase 1.

| Entity | Essential attributes/relations |
|---|---|
| `ingredient` | ID, name, category/type ID, section ID, preferred unit ID, season months, notes, timestamps |
| `ingredient_alias` | ID, ingredient ID, alias, unique normalized lookup |
| `ingredient_conversion` | ID, ingredient ID, from unit+amount, to unit+amount, approximation flag, notes |
| `unit` | ID, symbol, dimension, system/variant, conversion definition, aliases |
| `meal` | ID, title, description, base servings, prep/cook times, instructions, source URL/site, timestamps |
| `meal_ingredient` | ID, meal ID, ingredient ID (nullable until resolved), quantity, unit ID (nullable), raw text, optional, note, order |
| `custom_field_definition` | ID, label, type, options/validation, order, active flag |
| `meal_custom_field_value` | meal ID, field ID, typed/validated payload |
| `plan_entry` | ID, local date or unscheduled week, meal ID, servings, status, timestamps |
| `meal_cooked_event` | ID, meal ID, date/time, optional associated plan entry; source for last eaten |
| `shopping_list` | ID, week start, generation revision/metadata, timestamps |
| `shopping_item` | ID, list ID, ingredient ID or manual label, generated basis, override, unit, checked, excluded, section, sort order, origin |
| `import_record` | ID, meal ID, source URL/site, imported at, original information where useful |
| `app_settings` | Namespaced validated preferences |

Prefer durable IDs, unique constraints, referential integrity, timestamps, and migration tests. Use safe deletion/retirement semantics where an ingredient, field, or meal is referenced. Any denormalized/generated output must be regenerable or explicitly versioned.

## 7. Business rules and edge cases

- Scaling: `planned quantity = recipe quantity × planned servings ÷ base servings`; reject or flag zero/invalid servings, and preserve fractional servings where useful.
- Shopping aggregation keys are canonical ingredient and convertible unit group, not raw import strings.
- If one ingredient appears with incompatible dimensions, show separate shopping lines and the reason; never discard a line.
- Shopping overrides, manual additions, checkmarks, and excluded rows persist across recalculation by an explicit documented merge policy.
- Editing a meal after planning should trigger a clear “shopping list needs refresh” indication; do not silently mutate a checked-off list.
- Duplicate meal planning is allowed (e.g. leftovers or repeated meals), each plan entry counts separately.
- A cooked event is created once per deliberate action; undoing/reversing a status must not leave accidental duplicate history.
- `Unknown` is distinct from `false` for boolean custom fields; date/time behavior uses local dates with a documented timezone policy.
- Import and paste inputs must be treated as untrusted text and rendered safely; no untrusted HTML execution in the app UI.

## 8. Nonfunctional requirements

- **Offline first:** meals, planning, list editing and copying work without network. Recipe importing may require network and a live subscription.
- **Privacy:** default local-only. Don't send recipe data to analytics or third parties other than websites intentionally visited.
- **Data longevity:** automatic or user-triggered backup, export/import in a documented format (e.g. JSON with schema version); recover gracefully after interruption.
- **Security:** least-privilege Tauri capabilities, dependency updates, input validation, safe migration/backups, isolated external websites.
- **Accessibility:** keyboard navigation, visible focus states, legible contrast, proper labels, no drag-only essential actions.
- **Performance:** instant-feeling ordinary list/search and shopping generation on a modest household dataset; no unnecessary background network traffic.
- **Distribution:** reproducible signed or unsigned Windows installer as appropriate, documented install/update and data-location instructions; treat signing as optional for personal use.
- **Application updates (owner-approved Phase 8 extension):** installed release
  builds check the fixed PantryPal GitHub HTTPS feed once at startup and on a
  deliberate Settings check. Show release notes and allow postponement; require
  explicit download/install consent. Verify installer signatures and their signed
  versions, then save a household recovery backup before Windows installer
  handoff. Network failures never prevent core offline use. No household data,
  website credentials or cookies are sent with update checks. Updater signatures
  are mandatory; Windows Authenticode publisher signing remains optional.
- **Tests:** prioritize Rust unit/property tests for measurements and list generation; integration tests for SQLite/migrations/import drafts; UI tests for key user journeys.

## 9. Decisions and open questions

### Agreed

- Name: **PantryPal**.
- Windows desktop, built with **Rust + Tauri 2 + React + TypeScript**.
- Local-first SQLite; no cloud requirement.
- Seed and custom ingredients, customizable categories/sections, recipe library, configurable meal fields, free-form instructions.
- Weekly planning, shopping consolidation and convenient copy workflow.
- Mixed Canadian measurement conventions with explicit, safe conversions.
- Authenticated recipe site browser and session retention **as a target**, subject to feasibility and site permissions.

### To validate during implementation (do not quietly guess)

1. Tauri/WebView2 secure mechanism for recipe-page extraction without exposing native APIs to remote pages.
2. Cook Smarts permitted import/export method and login behavior; support is conditional.
3. Default seed ingredient scope and whether cuisine-specific categories are useful.
4. Preferred UI treatment for unscheduled weekly meals vs. exact dates.
5. Exact shopping-list regeneration/override UX and backup retention policy.

These should be captured as short architectural decisions (`docs/adr/`) as they are resolved, with a safe fallback when blocked.

## 10. MVP definition of done

A fresh Windows install can seed ingredients; create/edit a meal with ingredients, instructions and custom fields; plan multiple meals with servings; generate a correct, editable, copyable grouped shopping list; preserve data across restart; export a backup; and safely handle mixed units and unknown conversions. Import workflow must at least offer a working manual fallback and a tested secure browser/session feasibility path; an unpermitted or infeasible Cook Smarts extractor is **not** a release blocker for the local core app.
