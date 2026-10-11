CREATE TABLE ingredient_category (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
  normalized_name TEXT NOT NULL UNIQUE,
  retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE shopping_section (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
  normalized_name TEXT NOT NULL UNIQUE,
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
  retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE TABLE unit (
  id TEXT PRIMARY KEY NOT NULL,
  label TEXT NOT NULL,
  symbol TEXT NOT NULL,
  dimension TEXT NOT NULL CHECK(dimension IN ('mass','volume','count','informal','temperature')),
  system TEXT NOT NULL CHECK(system IN ('metric','us','imperial','neutral')),
  -- Exact affine definition relative to g, mL, or Celsius. NULL means no
  -- universal conversion (counts, packages, informal quantities).
  factor_numerator INTEGER,
  factor_denominator INTEGER,
  offset_numerator INTEGER,
  offset_denominator INTEGER,
  CHECK((factor_numerator IS NULL AND factor_denominator IS NULL AND offset_numerator IS NULL AND offset_denominator IS NULL)
    OR (factor_numerator IS NOT NULL AND factor_denominator IS NOT NULL AND offset_numerator IS NOT NULL AND offset_denominator IS NOT NULL
      AND factor_numerator > 0 AND factor_denominator > 0 AND offset_denominator > 0))
) STRICT;
CREATE TABLE unit_alias (
  unit_id TEXT NOT NULL REFERENCES unit(id) ON DELETE RESTRICT,
  alias TEXT NOT NULL,
  normalized_alias TEXT NOT NULL,
  PRIMARY KEY(unit_id,normalized_alias)
) STRICT;
CREATE INDEX unit_alias_lookup ON unit_alias(normalized_alias);
CREATE TABLE ingredient (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 120),
  normalized_name TEXT NOT NULL UNIQUE,
  category_id TEXT REFERENCES ingredient_category(id) ON DELETE RESTRICT,
  section_id TEXT REFERENCES shopping_section(id) ON DELETE RESTRICT,
  preferred_unit_id TEXT REFERENCES unit(id) ON DELETE RESTRICT,
  notes TEXT NOT NULL DEFAULT '',
  retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;
CREATE INDEX ingredient_category_idx ON ingredient(category_id);
CREATE INDEX ingredient_section_idx ON ingredient(section_id);
CREATE TABLE ingredient_alias (
  id TEXT PRIMARY KEY NOT NULL,
  ingredient_id TEXT NOT NULL REFERENCES ingredient(id) ON DELETE RESTRICT,
  alias TEXT NOT NULL CHECK(length(trim(alias)) BETWEEN 1 AND 120),
  normalized_alias TEXT NOT NULL UNIQUE
) STRICT;
CREATE INDEX ingredient_alias_ingredient_idx ON ingredient_alias(ingredient_id);
-- Canonical names and aliases share one identity namespace, including retired
-- records. A collision never silently chooses a different ingredient.
CREATE TRIGGER ingredient_name_insert BEFORE INSERT ON ingredient
WHEN EXISTS(SELECT 1 FROM ingredient_alias WHERE normalized_alias=NEW.normalized_name)
BEGIN SELECT RAISE(ABORT,'ingredient_name_conflict'); END;
CREATE TRIGGER ingredient_name_update BEFORE UPDATE OF normalized_name ON ingredient
WHEN EXISTS(SELECT 1 FROM ingredient_alias WHERE normalized_alias=NEW.normalized_name)
BEGIN SELECT RAISE(ABORT,'ingredient_name_conflict'); END;
CREATE TRIGGER ingredient_alias_insert BEFORE INSERT ON ingredient_alias
WHEN EXISTS(SELECT 1 FROM ingredient WHERE normalized_name=NEW.normalized_alias)
BEGIN SELECT RAISE(ABORT,'ingredient_name_conflict'); END;
CREATE TRIGGER ingredient_alias_update BEFORE UPDATE OF normalized_alias ON ingredient_alias
WHEN EXISTS(SELECT 1 FROM ingredient WHERE normalized_name=NEW.normalized_alias)
BEGIN SELECT RAISE(ABORT,'ingredient_name_conflict'); END;
CREATE TABLE ingredient_season_month (
  ingredient_id TEXT NOT NULL REFERENCES ingredient(id) ON DELETE RESTRICT,
  month INTEGER NOT NULL CHECK(month BETWEEN 1 AND 12),
  PRIMARY KEY(ingredient_id,month)
) STRICT;
CREATE TABLE ingredient_conversion (
  id TEXT PRIMARY KEY NOT NULL,
  ingredient_id TEXT NOT NULL REFERENCES ingredient(id) ON DELETE RESTRICT,
  from_unit_id TEXT NOT NULL REFERENCES unit(id) ON DELETE RESTRICT,
  from_numerator INTEGER NOT NULL CHECK(from_numerator > 0),
  from_denominator INTEGER NOT NULL CHECK(from_denominator > 0),
  to_unit_id TEXT NOT NULL REFERENCES unit(id) ON DELETE RESTRICT,
  to_numerator INTEGER NOT NULL CHECK(to_numerator > 0),
  to_denominator INTEGER NOT NULL CHECK(to_denominator > 0),
  approximate INTEGER NOT NULL CHECK(approximate IN (0,1)),
  notes TEXT NOT NULL DEFAULT '',
  CHECK(from_unit_id != to_unit_id)
) STRICT;
CREATE INDEX ingredient_conversion_ingredient_idx ON ingredient_conversion(ingredient_id);
