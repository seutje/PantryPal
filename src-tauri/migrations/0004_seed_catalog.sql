-- A versioned, once-only seed. Never reseed on every launch or overwrite edits.
INSERT INTO ingredient_category(id,name,normalized_name) VALUES
('category-vegetable','Vegetable','vegetable'),('category-fruit','Fruit','fruit'),
('category-meat','Meat','meat'),('category-dairy','Dairy','dairy'),
('category-grain','Grain','grain'),('category-pantry','Pantry staple','pantry staple');
INSERT INTO shopping_section(id,name,normalized_name,sort_order) VALUES
('section-produce','Produce','produce',0),('section-meat','Meat','meat',1),
('section-dairy','Dairy & eggs','dairy & eggs',2),('section-bakery','Bakery','bakery',3),
('section-pantry','Pantry','pantry',4),('section-frozen','Frozen','frozen',5),
('section-other','Other','other',6);
INSERT INTO unit(id,label,symbol,dimension,system,factor_numerator,factor_denominator,offset_numerator,offset_denominator) VALUES
('mg','Milligram','mg','mass','metric',1,1000,0,1),
('g','Gram','g','mass','metric',1,1,0,1),
('kg','Kilogram','kg','mass','metric',1000,1,0,1),
('oz','Ounce (mass)','oz','mass','us',28349523125,1000000000,0,1),
('lb','Pound','lb','mass','us',45359237,100000,0,1),
('ml','Millilitre','mL','volume','metric',1,1,0,1),
('l','Litre','L','volume','metric',1000,1,0,1),
('cup-metric','Metric cup','metric cup','volume','metric',250,1,0,1),
('cup-us','US cup','US cup','volume','us',2365882365,10000000,0,1),
('cup-imperial','Imperial cup','Imperial cup','volume','imperial',284130625,1000000,0,1),
('tsp-metric','Metric teaspoon','metric tsp','volume','metric',5,1,0,1),
('tbsp-metric','Metric tablespoon','metric tbsp','volume','metric',15,1,0,1),
('tsp-us','US teaspoon','US tsp','volume','us',492892159375,100000000000,0,1),
('tbsp-us','US tablespoon','US tbsp','volume','us',1478676478125,100000000000,0,1),
('tsp-imperial','Imperial teaspoon','Imperial tsp','volume','imperial',284130625,48000000,0,1),
('tbsp-imperial','Imperial tablespoon','Imperial tbsp','volume','imperial',284130625,16000000,0,1),
('floz-us','US fluid ounce','US fl oz','volume','us',295735295625,10000000000,0,1),
('floz-imperial','Imperial fluid ounce','Imperial fl oz','volume','imperial',284130625,10000000,0,1),
('pint-us','US pint','US pint','volume','us',473176473,1000000,0,1),
('pint-imperial','Imperial pint','Imperial pint','volume','imperial',56826125,100000,0,1),
('quart-us','US quart','US qt','volume','us',946352946,1000000,0,1),
('quart-imperial','Imperial quart','Imperial qt','volume','imperial',11365225,10000,0,1),
('gallon-us','US gallon','US gal','volume','us',3785411784,1000000,0,1),
('gallon-imperial','Imperial gallon','Imperial gal','volume','imperial',454609,100,0,1),
('celsius','Celsius','°C','temperature','metric',1,1,0,1),
('fahrenheit','Fahrenheit','°F','temperature','us',5,9,-160,9);
INSERT INTO unit(id,label,symbol,dimension,system) VALUES
('piece','Piece','pcs','count','neutral'),('clove','Clove','cloves','count','neutral'),
('slice','Slice','slices','count','neutral'),('bunch','Bunch','bunch','count','neutral'),
('can','Can','can','count','neutral'),('package','Package','pack','count','neutral'),
('pinch','Pinch','pinch','informal','neutral'),('dash','Dash','dash','informal','neutral'),
('handful','Handful','handful','informal','neutral'),('to-taste','To taste','to taste','informal','neutral');
INSERT INTO unit_alias(unit_id,alias,normalized_alias) VALUES
('g','grams','grams'),('kg','kilograms','kilograms'),('ml','millilitres','millilitres'),
('l','litres','litres'),('lb','pounds','pounds'),('oz','ounces','ounces'),
('cup-metric','cup','cup'),('cup-us','cup','cup'),('cup-imperial','cup','cup'),
('piece','pieces','pieces'),('piece','pcs','pcs'),('to-taste','to taste','to taste');
INSERT INTO ingredient(id,name,normalized_name,category_id,section_id,preferred_unit_id) VALUES
('ingredient-onion','Onion','onion','category-vegetable','section-produce','piece'),
('ingredient-garlic','Garlic','garlic','category-vegetable','section-produce','clove'),
('ingredient-carrot','Carrot','carrot','category-vegetable','section-produce','g'),
('ingredient-broccoli','Broccoli','broccoli','category-vegetable','section-produce','g'),
('ingredient-potato','Potato','potato','category-vegetable','section-produce','g'),
('ingredient-tomato','Tomato','tomato','category-vegetable','section-produce','g'),
('ingredient-apple','Apple','apple','category-fruit','section-produce','piece'),
('ingredient-chicken','Chicken breast','chicken breast','category-meat','section-meat','g'),
('ingredient-beef','Ground beef','ground beef','category-meat','section-meat','g'),
('ingredient-milk','Milk','milk','category-dairy','section-dairy','ml'),
('ingredient-butter','Butter','butter','category-dairy','section-dairy','g'),
('ingredient-egg','Egg','egg','category-dairy','section-dairy','piece'),
('ingredient-rice','Rice','rice','category-grain','section-pantry','g'),
('ingredient-flour','All-purpose flour','all-purpose flour','category-grain','section-pantry','g'),
('ingredient-salt','Salt','salt','category-pantry','section-pantry','g'),
('ingredient-oil','Olive oil','olive oil','category-pantry','section-pantry','ml');
INSERT INTO ingredient_alias(id,ingredient_id,alias,normalized_alias) VALUES
('alias-onions','ingredient-onion','Onions','onions'),
('alias-eggs','ingredient-egg','Eggs','eggs'),
('alias-flour','ingredient-flour','Plain flour','plain flour'),
('alias-beef','ingredient-beef','Minced beef','minced beef');
INSERT INTO app_settings(key,value_json) VALUES ('measurement.mode','"mixed"');
