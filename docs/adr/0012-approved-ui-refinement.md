# ADR 0012: Approved desktop UI refinement

Date: 2026-10-13
Status: Accepted by the owner

The owner approved `design/index.html` and requested implementation. The written
behavior notes resolve known image-generation artifacts, including incorrect
filter counts, restricted numeric inputs, unsupported controls and branding.

Keep the existing typed IPC, Rust services, exact quantities, schema and
capability boundary. Implement the presentation in React/CSS with local fonts
and code-native icons; generated PNGs remain proposal artifacts only.

Use native disclosures with labelled buttons for secondary actions, rather than
pretending an incomplete disclosure is an ARIA menu. Escape closes disclosures
and restores their trigger focus. Native modal dialogs retain focus containment,
Escape handling and opener restoration. Confirmation copy describes the action's
actual data effect; keep the safe choice prominent.

Meal selection in the planner becomes explicit selection followed by destination
and text servings input and Add. Preserve independent occurrences and backend
validation/idempotent retry. Settings task panels remain mounted while hidden,
retaining unsaved drafts; saving still uses each existing validated operation.
Recipe and ingredient forms group optional metadata behind disclosures without
dropping raw wording, aliases, conversion rules, null values or historical data.

The stale measurement aggregation copy conflicts with Phase 5 completion.
Replace its future-tense wording while preserving the existing preference.
Do not infer new functionality from generated icons, slogans, spinners, sorting
arrows, illustrative release availability or restart buttons.
