-- Storage contracts only: planning, cooking and regeneration workflows follow
-- in later phases. Do not infer cooked history from scheduled entries.
CREATE TABLE plan_entry (
  id TEXT PRIMARY KEY NOT NULL,
  week_start TEXT NOT NULL CHECK(length(week_start)=10 AND date(week_start,'+0 days') IS NOT NULL AND date(week_start,'+0 days')=week_start AND strftime('%w',week_start)='1'),
  local_date TEXT CHECK(local_date IS NULL OR (length(local_date)=10 AND date(local_date,'+0 days') IS NOT NULL AND date(local_date,'+0 days')=local_date)),
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  servings_numerator INTEGER NOT NULL CHECK(servings_numerator > 0),
  servings_denominator INTEGER NOT NULL CHECK(servings_denominator > 0),
  status TEXT NOT NULL DEFAULT 'planned' CHECK(status IN ('planned','cooked','skipped')),
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  CHECK(local_date IS NULL OR (local_date >= week_start AND local_date <= date(week_start,'+6 days')))
) STRICT;
CREATE INDEX plan_entry_week_idx ON plan_entry(week_start,sort_order);
CREATE INDEX plan_entry_meal_idx ON plan_entry(meal_id);
CREATE TABLE meal_cooked_event (
  id TEXT PRIMARY KEY NOT NULL,
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  local_date TEXT NOT NULL CHECK(length(local_date)=10 AND date(local_date,'+0 days') IS NOT NULL AND date(local_date,'+0 days')=local_date),
  plan_entry_id TEXT UNIQUE REFERENCES plan_entry(id) ON DELETE RESTRICT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX cooked_event_meal_date_idx ON meal_cooked_event(meal_id,local_date);
CREATE TRIGGER cooked_event_matches_plan BEFORE INSERT ON meal_cooked_event
WHEN NEW.plan_entry_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM plan_entry WHERE id=NEW.plan_entry_id AND meal_id=NEW.meal_id)
BEGIN SELECT RAISE(ABORT,'cooked_event_meal_conflict'); END;
CREATE TRIGGER cooked_event_update_matches_plan BEFORE UPDATE ON meal_cooked_event
WHEN NEW.plan_entry_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM plan_entry WHERE id=NEW.plan_entry_id AND meal_id=NEW.meal_id)
BEGIN SELECT RAISE(ABORT,'cooked_event_meal_conflict'); END;
CREATE TRIGGER plan_update_preserves_cooked_meal BEFORE UPDATE OF meal_id ON plan_entry
WHEN EXISTS(SELECT 1 FROM meal_cooked_event WHERE plan_entry_id=OLD.id AND meal_id != NEW.meal_id)
BEGIN SELECT RAISE(ABORT,'cooked_event_meal_conflict'); END;
CREATE TABLE shopping_list (
  id TEXT PRIMARY KEY NOT NULL,
  week_start TEXT NOT NULL UNIQUE CHECK(length(week_start)=10 AND date(week_start,'+0 days') IS NOT NULL AND date(week_start,'+0 days')=week_start AND strftime('%w',week_start)='1'),
  generation_revision INTEGER NOT NULL DEFAULT 0 CHECK(generation_revision >= 0),
  generation_metadata_json TEXT NOT NULL DEFAULT '{}' CHECK(json_valid(generation_metadata_json)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE shopping_item (
  id TEXT PRIMARY KEY NOT NULL,
  list_id TEXT NOT NULL REFERENCES shopping_list(id) ON DELETE RESTRICT,
  ingredient_id TEXT REFERENCES ingredient(id) ON DELETE RESTRICT,
  manual_label TEXT,
  origin TEXT NOT NULL CHECK(origin IN ('generated','manual')),
  generation_key TEXT,
  generated_basis_json TEXT CHECK(generated_basis_json IS NULL OR json_valid(generated_basis_json)),
  -- An explicit flag distinguishes no override from an unknown amount override.
  has_override INTEGER NOT NULL DEFAULT 0 CHECK(has_override IN (0,1)),
  override_json TEXT CHECK(override_json IS NULL OR json_valid(override_json)),
  unit_id TEXT REFERENCES unit(id) ON DELETE RESTRICT,
  checked INTEGER NOT NULL DEFAULT 0 CHECK(checked IN (0,1)),
  excluded INTEGER NOT NULL DEFAULT 0 CHECK(excluded IN (0,1)),
  section_id TEXT REFERENCES shopping_section(id) ON DELETE RESTRICT,
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
  UNIQUE(list_id,generation_key),
  CHECK((origin='manual' AND manual_label IS NOT NULL AND length(trim(manual_label)) > 0 AND generation_key IS NULL)
    OR (origin='generated' AND ingredient_id IS NOT NULL AND generation_key IS NOT NULL AND generated_basis_json IS NOT NULL))
) STRICT;
CREATE INDEX shopping_item_ingredient_idx ON shopping_item(ingredient_id);
CREATE INDEX shopping_item_section_idx ON shopping_item(section_id);
CREATE TABLE import_record (
  id TEXT PRIMARY KEY NOT NULL,
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  source_url TEXT NOT NULL,
  source_site TEXT,
  imported_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  provenance_json TEXT NOT NULL DEFAULT '{}' CHECK(json_valid(provenance_json))
) STRICT;
CREATE INDEX import_record_meal_idx ON import_record(meal_id);
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY NOT NULL CHECK(length(trim(key)) > 0),
  value_json TEXT NOT NULL CHECK(json_valid(value_json)),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
