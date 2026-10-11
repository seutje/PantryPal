-- Retain entered numeric wording independently of normalized rational values.
ALTER TABLE meal ADD COLUMN base_servings_text TEXT;
ALTER TABLE meal_ingredient ADD COLUMN amount_text TEXT;
