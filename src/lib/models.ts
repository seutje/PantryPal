export interface Amount {
  numerator: number;
  denominator: number;
}
export interface IngredientInput {
  name: string;
  categoryId: string | null;
  sectionId: string | null;
  preferredUnitId: string | null;
  aliases: string[];
  seasonMonths: number[];
  notes: string;
}
export interface Ingredient extends IngredientInput {
  id: string;
  retired: boolean;
  createdAt: string;
  updatedAt: string;
}
export type VocabularyKind = 'category' | 'section';
export interface Vocabulary {
  id: string;
  name: string;
  sortOrder: number;
  retired: boolean;
}
export interface Unit {
  id: string;
  label: string;
  symbol: string;
  dimension: 'mass' | 'volume' | 'count' | 'informal' | 'temperature';
  system: 'metric' | 'us' | 'imperial' | 'neutral';
  factorNumerator: number | null;
  factorDenominator: number | null;
  offsetNumerator: number | null;
  offsetDenominator: number | null;
}
export interface Catalog {
  ingredients: Ingredient[];
  categories: Vocabulary[];
  sections: Vocabulary[];
  units: Unit[];
}
export interface StorageStatus {
  schemaVersion: number;
  ingredientCount: number;
  mealCount: number;
}
export type FieldKind =
  'boolean' | 'select' | 'number' | 'date' | 'text' | 'rating';
export type FieldValue =
  | { kind: 'boolean'; value: boolean }
  | { kind: 'select' | 'date' | 'text'; value: string }
  | { kind: 'number' | 'rating'; value: number };
export interface FieldInput {
  label: string;
  kind: FieldKind;
  description: string;
  options: { id: string; label: string }[];
  minimum: number | null;
  maximum: number | null;
  unitLabel: string | null;
  sortOrder: number;
}
export interface FieldDefinition extends FieldInput {
  id: string;
  retired: boolean;
}
export interface MealIngredient {
  id: string;
  ingredientId: string | null;
  quantity: Amount | null;
  unitId: string | null;
  rawText: string;
  sourceConvention: string | null;
  note: string;
  optional: boolean;
}
export interface MealInput {
  title: string;
  description: string;
  baseServings: Amount;
  prepMinutes: number | null;
  cookMinutes: number | null;
  instructions: string;
  notes: string;
  sourceUrl: string | null;
  sourceSite: string | null;
  tags: string[];
  ingredients: MealIngredient[];
  customFields: { fieldId: string; value: FieldValue | null }[];
}
export interface Meal extends MealInput {
  id: string;
  retired: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
}
