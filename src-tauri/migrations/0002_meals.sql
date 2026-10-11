CREATE TABLE meal (
  id TEXT PRIMARY KEY NOT NULL,
  title TEXT NOT NULL CHECK(length(trim(title)) BETWEEN 1 AND 200),
  normalized_title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  servings_numerator INTEGER NOT NULL CHECK(servings_numerator > 0),
  servings_denominator INTEGER NOT NULL CHECK(servings_denominator > 0),
  prep_minutes INTEGER CHECK(prep_minutes >= 0),
  cook_minutes INTEGER CHECK(cook_minutes >= 0),
  instructions TEXT NOT NULL DEFAULT '',
  notes TEXT NOT NULL DEFAULT '',
  source_url TEXT,
  source_site TEXT,
  retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1)),
  revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX meal_title_lookup ON meal(normalized_title);
CREATE TABLE meal_tag (
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  tag TEXT NOT NULL,
  PRIMARY KEY(meal_id,tag)
) STRICT;
CREATE TABLE meal_ingredient (
  id TEXT PRIMARY KEY NOT NULL,
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  ingredient_id TEXT REFERENCES ingredient(id) ON DELETE RESTRICT,
  quantity_numerator INTEGER,
  quantity_denominator INTEGER,
  unit_id TEXT REFERENCES unit(id) ON DELETE RESTRICT,
  raw_text TEXT NOT NULL DEFAULT '',
  source_convention TEXT,
  note TEXT NOT NULL DEFAULT '',
  optional INTEGER NOT NULL DEFAULT 0 CHECK(optional IN (0,1)),
  sort_order INTEGER NOT NULL CHECK(sort_order >= 0),
  UNIQUE(meal_id,sort_order),
  CHECK((quantity_numerator IS NULL AND quantity_denominator IS NULL)
    OR (quantity_numerator IS NOT NULL AND quantity_denominator IS NOT NULL AND quantity_numerator >= 0 AND quantity_denominator > 0))
) STRICT;
CREATE INDEX meal_ingredient_ingredient_idx ON meal_ingredient(ingredient_id);
CREATE TABLE custom_field_definition (
  id TEXT PRIMARY KEY NOT NULL,
  label TEXT NOT NULL CHECK(length(trim(label)) BETWEEN 1 AND 120),
  normalized_label TEXT NOT NULL UNIQUE,
  kind TEXT NOT NULL CHECK(kind IN ('boolean','select','number','date','text','rating')),
  description TEXT NOT NULL DEFAULT '',
  options_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(options_json) AND json_type(options_json)='array'),
  minimum REAL,
  maximum REAL,
  unit_label TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
  retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  CHECK(minimum IS NULL OR maximum IS NULL OR minimum <= maximum)
) STRICT;
CREATE TABLE meal_custom_field_value (
  meal_id TEXT NOT NULL REFERENCES meal(id) ON DELETE RESTRICT,
  field_id TEXT NOT NULL REFERENCES custom_field_definition(id) ON DELETE RESTRICT,
  -- Tagged typed payload: NULL means unknown; false remains a boolean value.
  value_json TEXT CHECK(value_json IS NULL OR json_valid(value_json)),
  PRIMARY KEY(meal_id,field_id)
) STRICT;
CREATE INDEX meal_custom_field_field_idx ON meal_custom_field_value(field_id);
