# ADR 0005: Local weeks, plan actions and actual cooking

Status: Accepted

Date: 2026-10-11

## Decision

Rust resolves the selected local ISO calendar date to a Monday–Sunday week, including month/year and leap-day boundaries. The frontend supplies this computer's current local date for the initial view and renders date labels without a UTC round-trip. Scheduling uses calendar dates, not elapsed 24-hour intervals, so DST does not move a meal to another day. Supported dates are years 0001–9999 with a complete seven-day week. Previous/next links are absent at the supported range boundaries.

Each occurrence has a stable ID, independent positive exact servings, explicit status and order within a day or the unscheduled “This week” pool. Moving an occurrence appends it to its new group. Reordering is a keyboard-accessible operation that must name every entry in that group exactly once. Duplicate meals are separate occurrences. Historical retired recipes remain readable/editable in existing plans; new occurrences cannot use a retired recipe. Changing a scheduled date or servings never changes actual cooking dates. Unchanged legacy serving pairs remain intact.

Migration 7 is additive. A per-week revision guards all plan writes and ordering against stale views. Every plan/history mutation has a stable user-action ID and durable request receipt, committed together with its changes. An exact retry returns current data without repeating the operation; reuse with a different request is rejected. Receipts survive removal and undo, so retrying an old cook or add action cannot reinstate it. The UI retains failed inputs and retries the same payload; reloading is an explicit alternative for conflicts. The ledger is local household data, never telemetry.

Only an explicit cooking action creates history. The date actually cooked is independent of the scheduled date and may be backfilled. Rust validates dates and rejects new future cooking against SQLite's `date('now','localtime')`, which follows the computer's local calendar. Historical dates remain calendar dates when timezone settings change; audit timestamps remain UTC. Correcting a date or undoing a record requires confirmation and its saved revision. Corrections update a linked plan atomically; undo returns that plan to `planned` and increments its week revision.

Undo voids the event instead of deleting it. Last eaten is the maximum date of non-voided events. Re-recording the same planned occurrence reactivates its single linked event; date/void changes retain an audit trail. Removing a plan requires confirmation and detaches its history, retaining the origin occurrence ID; it never erases an actual cooking record or recipe. Standalone backfills are separately identified events. Legacy status/event inconsistencies are not silently repaired or used to invent cooked events.

The planner opens recipe detail in Meals and passes its canonical Monday to Shopping. The generation action's week selection and navigation are implemented here; grocery calculations and generated items belong to Phase 5. The Shopping screen explicitly explains their current availability. No Phase 5 calculation or regeneration behavior is claimed.

## Occurrence ingredient displays

Follow-up (2026-10-12): Week cards expose **Edit servings** and an expandable
ingredient view for the occurrence. Rust derives each numeric amount using
exact `quantity × planned servings ÷ recipe base servings`, preserving the
source unit and unknown amounts; temperatures remain unscaled. Decimal views
identify rounding and retain the exact fraction. These view-only strings are
returned alongside plan entries, never written to recipes or the database.
Changing servings uses the existing plan edit action and Shopping's existing
explicit refresh policy. Recipe base servings, ingredient quantities and other
occurrences remain unchanged; no migration is needed.

## Verification

Rust tests cover repeated occurrences, exact/unknown/invalid servings, order and day membership, stale writes, action retries after undo/removal and restart, status changes, actual last-eaten dates, corrections, void audit and preserved detached history. Migration coverage upgrades an existing schema 6 plan/history/manual shopping adjustment without loss. Command integration uses actual Tauri deserialization, capabilities and temporary SQLite with the OS runtime mocked; remote-origin planner/history calls are rejected. A separate-process restart test retains repeated plans and cooking and safely retries the original action. React tests cover forms, keyboard ordering, statuses/confirmation, backfill/correction/undo, failed-input retention and the same-action retry, recipe navigation and week-aware Shopping handoff. Windows debug build/native startup and calendar navigation are verified separately; release installation and external-WebView isolation remain in their assigned phases.
