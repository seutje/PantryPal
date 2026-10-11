-- Additive only: retain every existing quantity, preference and rule.
ALTER TABLE ingredient_conversion ADD COLUMN original_from_text TEXT NOT NULL DEFAULT '';
ALTER TABLE ingredient_conversion ADD COLUMN original_to_text TEXT NOT NULL DEFAULT '';
ALTER TABLE ingredient_conversion ADD COLUMN retired INTEGER NOT NULL DEFAULT 0 CHECK(retired IN (0,1));
INSERT INTO app_settings(key,value_json)
SELECT 'measurement.preferences', json_object(
    'mode', coalesce((SELECT json_extract(value_json,'$') FROM app_settings WHERE key='measurement.mode'), 'mixed'),
    'cupConvention', NULL,
    'fractions', json(coalesce((SELECT value_json FROM app_settings WHERE key='display.fractions'), 'true')),
    'decimalPlaces', 3,
    'aggregateCompatible', json('true')
);
-- Deliberately ambiguous short names return every explicit regional variant.
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'tsp' FROM unit WHERE id IN ('tsp-us','tsp-metric','tsp-imperial');
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'tbsp' FROM unit WHERE id IN ('tbsp-us','tbsp-metric','tbsp-imperial');
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'fl oz' FROM unit WHERE id IN ('floz-us','floz-imperial');
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'pint' FROM unit WHERE id IN ('pint-us','pint-imperial');
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'quart' FROM unit WHERE id IN ('quart-us','quart-imperial');
INSERT OR IGNORE INTO unit_alias(unit_id,normalized_alias)
SELECT id, 'gallon' FROM unit WHERE id IN ('gallon-us','gallon-imperial');
