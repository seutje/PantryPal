import type {
  Catalog,
  Ingredient,
  MeasurementPreferences,
  Unit,
} from '../lib/models';
export const preferences: MeasurementPreferences = {
  mode: 'mixed',
  cupConvention: null,
  fractions: true,
  decimalPlaces: 3,
  aggregateCompatible: true,
};
export const onion: Ingredient = {
  id: 'ingredient-onion',
  name: 'Onion',
  aliases: ['Onions'],
  categoryId: 'category-vegetable',
  sectionId: 'section-produce',
  preferredUnitId: 'g',
  seasonMonths: [6, 7],
  notes: 'Keep cool',
  retired: false,
  createdAt: '',
  updatedAt: '',
};
const unit = (
  id: string,
  label: string,
  symbol: string,
  dimension: Unit['dimension'],
  system: Unit['system'],
  factorNumerator: number | null,
  factorDenominator: number | null,
): Unit => ({
  id,
  label,
  symbol,
  dimension,
  system,
  factorNumerator,
  factorDenominator,
  offsetNumerator: factorNumerator === null ? null : 0,
  offsetDenominator: factorNumerator === null ? null : 1,
});
export const catalogFixture: Catalog = {
  ingredients: [onion],
  categories: [
    {
      id: 'category-vegetable',
      name: 'Vegetable',
      sortOrder: 0,
      retired: false,
    },
  ],
  sections: [
    { id: 'section-produce', name: 'Produce', sortOrder: 0, retired: false },
  ],
  units: [
    unit('g', 'Gram', 'g', 'mass', 'metric', 1, 1),
    unit('kg', 'Kilogram', 'kg', 'mass', 'metric', 1000, 1),
    unit('ml', 'Millilitre', 'mL', 'volume', 'metric', 1, 1),
    unit('cup-metric', 'Metric cup', 'metric cup', 'volume', 'metric', 250, 1),
    unit('cup-us', 'US cup', 'US cup', 'volume', 'us', 2365882365, 10000000),
    unit(
      'cup-imperial',
      'Imperial cup',
      'Imperial cup',
      'volume',
      'imperial',
      284130625,
      1000000,
    ),
    unit('piece', 'Piece', 'pc', 'count', 'neutral', null, null),
    unit('celsius', 'Celsius', '°C', 'temperature', 'metric', 1, 1),
    {
      ...unit('fahrenheit', 'Fahrenheit', '°F', 'temperature', 'us', 5, 9),
      offsetNumerator: -160,
      offsetDenominator: 9,
    },
  ],
};
