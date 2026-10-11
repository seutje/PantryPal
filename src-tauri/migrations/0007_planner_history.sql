-- Additive: keep existing plans, history and shopping adjustments intact.
ALTER TABLE plan_entry ADD COLUMN servings_text TEXT;
ALTER TABLE meal_cooked_event ADD COLUMN voided INTEGER NOT NULL DEFAULT 0 CHECK(voided IN (0,1));
ALTER TABLE meal_cooked_event ADD COLUMN revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>0);
ALTER TABLE meal_cooked_event ADD COLUMN origin_plan_entry_id TEXT;
UPDATE meal_cooked_event SET origin_plan_entry_id=plan_entry_id;
CREATE TABLE plan_week (
    week_start TEXT PRIMARY KEY NOT NULL,
    revision INTEGER NOT NULL CHECK(revision>0)
) STRICT;
INSERT INTO plan_week(week_start,revision) SELECT DISTINCT week_start,1 FROM plan_entry;
-- Durable receipts make retries safe even after an undo or removal.
CREATE TABLE planner_action (
    id TEXT PRIMARY KEY NOT NULL,
    request_json TEXT NOT NULL CHECK(json_valid(request_json)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE cooked_event_change (
    id TEXT PRIMARY KEY NOT NULL,
    event_id TEXT NOT NULL REFERENCES meal_cooked_event(id) ON DELETE RESTRICT,
    previous_date TEXT NOT NULL,
    next_date TEXT NOT NULL,
    previous_voided INTEGER NOT NULL CHECK(previous_voided IN (0,1)),
    next_voided INTEGER NOT NULL CHECK(next_voided IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
