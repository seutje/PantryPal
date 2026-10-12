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
  amountText?: string | null;
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
  baseServingsText?: string | null;
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
export interface IngredientDraft extends Omit<
  MealIngredient,
  'quantity' | 'amountText'
> {
  amount: string;
}
export interface MealDraft extends Omit<
  MealInput,
  'baseServings' | 'baseServingsText' | 'ingredients'
> {
  servings: string;
  ingredients: IngredientDraft[];
}
export interface MealSearch {
  text: string;
  ingredientId: string | null;
  tag: string;
  field: {
    fieldId: string;
    operator: 'equals' | 'known' | 'unknown';
    value: FieldValue | null;
  } | null;
  cooked: 'any' | 'never' | 'cooked';
  cookedFrom: string | null;
  cookedTo: string | null;
}
export interface MealRecord {
  meal: Meal;
  lastCooked: string | null;
  servingsDisplay: string;
  quantityDisplays: (string | null)[];
}
export interface MealLibrary {
  meals: MealRecord[];
  catalog: Catalog;
  fields: FieldDefinition[];
}
export type PlanStatus = 'planned' | 'cooked' | 'skipped';
export interface CookedEvent {
  id: string;
  mealId: string;
  localDate: string;
  planEntryId: string | null;
  originPlanEntryId: string | null;
  voided: boolean;
  revision: number;
}
export interface BackupInfo {
  directory: string;
}
export interface BackupResult {
  path: string;
}
export type RestoreMode = 'merge' | 'replace';
export interface RestorePreview {
  expectedState: string;
  meals: number;
  ingredients: number;
  rows: number;
}
export interface UsabilityPreferences {
  clipboardUrlPrefill: boolean;
  sectionOrder: string[];
}
export interface PlanEntry {
  id: string;
  mealId: string;
  localDate: string | null;
  servings: Amount;
  servingsText: string;
  quantityDisplays: (string | null)[];
  status: PlanStatus;
  sortOrder: number;
  cookedEvent: CookedEvent | null;
}
export interface WeekView {
  weekStart: string;
  weekEnd: string;
  days: string[];
  previousWeek: string | null;
  nextWeek: string | null;
  revision: number;
  entries: PlanEntry[];
  library: MealLibrary;
}
export type PlanOperation =
  | {
      kind: 'add';
      entryId: string;
      mealId: string;
      localDate: string | null;
      servings: string;
    }
  | {
      kind: 'edit';
      entryId: string;
      localDate: string | null;
      servings: string;
    }
  | { kind: 'reorder'; localDate: string | null; entryIds: string[] }
  | { kind: 'remove'; entryId: string; confirmed: boolean }
  | {
      kind: 'set_status';
      entryId: string;
      status: PlanStatus;
      cookedDate: string | null;
      confirmed: boolean;
    };
export interface PlanMutation {
  actionId: string;
  weekStart: string;
  expectedRevision: number;
  operation: PlanOperation;
}
export interface MealHistory {
  mealId: string;
  lastCooked: string | null;
  events: CookedEvent[];
}
export type HistoryOperation =
  | { kind: 'record'; eventId: string; localDate: string }
  | {
      kind: 'correct';
      eventId: string;
      expectedRevision: number;
      localDate: string;
      confirmed: boolean;
    }
  | {
      kind: 'undo';
      eventId: string;
      expectedRevision: number;
      confirmed: boolean;
    };
export interface HistoryMutation {
  actionId: string;
  mealId: string;
  operation: HistoryOperation;
}
export interface Meal extends MealInput {
  id: string;
  retired: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
}
export interface MeasurementPreferences {
  mode: 'mixed' | 'metric' | 'us';
  cupConvention: 'metric' | 'us' | 'imperial' | null;
  fractions: boolean;
  decimalPlaces: number;
  aggregateCompatible: boolean;
}
export interface ConversionRequest {
  amount: string;
  fromUnit: string;
  toUnit: string;
  ingredientId: string | null;
  purpose: 'quantity' | 'temperature';
}
export type ConversionResult =
  | {
      status: 'converted';
      original: ConversionRequest;
      numerator: string;
      denominator: string;
      exact: string;
      display: string;
      rounded: boolean;
      approximate: boolean;
      ruleIds: string[];
      fromUnitId: string;
      toUnitId: string;
    }
  | {
      status: 'cannot_convert';
      original: ConversionRequest;
      reason: string;
      candidates: string[];
    };
export interface RuleInput {
  ingredientId: string;
  fromUnitId: string;
  fromAmount: string;
  toUnitId: string;
  toAmount: string;
  approximate: boolean;
  notes: string;
}
export interface ConversionRule {
  id: string;
  ingredientId: string;
  fromUnitId: string;
  fromNumerator: number;
  fromDenominator: number;
  toUnitId: string;
  toNumerator: number;
  toDenominator: number;
  approximate: boolean;
  notes: string;
  originalFromText: string;
  originalToText: string;
  retired: boolean;
}

export interface ShoppingItem {
  id: string;
  key: string | null;
  ingredientId: string | null;
  label: string;
  sectionId: string | null;
  numerator: string | null;
  denominator: string | null;
  unitId: string | null;
  amountText: string;
  display: string;
  rounded: boolean;
  section: string;
  alternatives: string[];
  approximate: boolean;
  warnings: string[];
  ruleIds: string[];
  sources: {
    entryId: string;
    mealId: string;
    mealTitle: string;
    plannedServings: Amount;
    baseServings: Amount;
    line: MealIngredient;
  }[];
  adjustment: { amount: string; unitId: string | null } | null;
  checked: boolean;
  excluded: boolean;
  orphaned: boolean;
  legacy: unknown;
}
export interface ShoppingView {
  weekStart: string;
  revision: number;
  generated: boolean;
  stale: boolean;
  items: ShoppingItem[];
  sections: Vocabulary[];
  units: Unit[];
  text: string;
}
export type ShoppingOperation =
  | { kind: 'refresh' }
  | {
      kind: 'manual';
      itemId: string;
      label: string;
      amount: string;
      unitId: string | null;
      sectionId: string | null;
    }
  | { kind: 'adjust'; itemId: string; amount: string; unitId: string | null }
  | { kind: 'reset'; itemId: string; confirmed: boolean }
  | { kind: 'check'; itemId: string; checked: boolean }
  | { kind: 'exclude'; itemId: string; excluded: boolean };
export interface ShoppingMutation {
  actionId: string;
  weekStart: string;
  expectedRevision: number;
  operation: ShoppingOperation;
}
export interface ImportInput {
  sourceUrl: string;
  format: 'json_ld' | 'manual' | 'website';
  payload: string;
  permitted: boolean;
}
export interface ImportedRecipeDraft {
  meal: MealDraft;
  warnings: string[];
  originalYield: string;
  attribution: string;
  duplicateTitles: string[];
}
export interface ConfirmImport {
  actionId: string;
  source: ImportInput;
  reviewed: MealDraft;
  confirmed: boolean;
  allowDuplicate: boolean;
}
