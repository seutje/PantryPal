import { invoke, isTauri } from '@tauri-apps/api/core';
import type {
  Catalog,
  Ingredient,
  IngredientInput,
  StorageStatus,
  Meal,
  MealInput,
  FieldDefinition,
  FieldInput,
  Vocabulary,
  VocabularyKind,
  MeasurementPreferences,
  ConversionRequest,
  ConversionResult,
  ConversionRule,
  RuleInput,
  MealDraft,
  MealSearch,
  MealLibrary,
  WeekView,
  PlanMutation,
  MealHistory,
  HistoryMutation,
  ShoppingView,
  ShoppingMutation,
  ImportInput,
  ImportedRecipeDraft,
  ConfirmImport,
} from './models';

export const inspectRecipeUrl = (sourceUrl: string) =>
  invokeNative<string>('inspect_recipe_url', { sourceUrl });
export const openRecipeBrowser = (sourceUrl: string) =>
  invokeNative<void>('open_recipe_browser', { sourceUrl });
export const previewRecipeImport = (input: ImportInput) =>
  invokeNative<ImportedRecipeDraft>('preview_recipe_import', { input });
export const confirmRecipeImport = (input: ConfirmImport) =>
  invokeNative<Meal>('confirm_recipe_import', { input });
export const resetRecipeBrowser = (confirmed: boolean) =>
  invokeNative<void>('reset_recipe_browser', { confirmed });

export interface AppInfo {
  name: string;
  version: string;
}

export interface AppError {
  code: string;
  message: string;
}

export class NativeError extends Error {
  readonly code: string;

  constructor(error: AppError) {
    super(error.message);
    this.name = 'NativeError';
    this.code = error.code;
  }
}

function normalizeError(error: unknown): NativeError {
  if (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    typeof error.code === 'string' &&
    typeof error.message === 'string'
  ) {
    return new NativeError({ code: error.code, message: error.message });
  }
  return new NativeError({
    code: 'native_unavailable',
    message: 'Could not connect to PantryPal. Please restart the app.',
  });
}

export const nativeAvailable = isTauri;

// Expose named, typed operations, never a generic UI-controlled command name.
export async function getAppInfo(): Promise<AppInfo> {
  return invokeNative<AppInfo>('get_app_info');
}

// Kept private: only named operations below may select a native command.
async function invokeNative<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    throw normalizeError(error);
  }
}

export const getStorageStatus = () =>
  invokeNative<StorageStatus>('get_storage_status');
export const getCatalog = () => invokeNative<Catalog>('get_catalog');
export const getMeasurementPreferences = () =>
  invokeNative<MeasurementPreferences>('get_measurement_preferences');
export const saveMeasurementPreferences = (input: MeasurementPreferences) =>
  invokeNative<MeasurementPreferences>('save_measurement_preferences', {
    input,
  });
export const convertMeasurement = (input: ConversionRequest) =>
  invokeNative<ConversionResult>('convert_measurement', { input });
export const listConversionRules = (ingredientId: string) =>
  invokeNative<ConversionRule[]>('list_conversion_rules', { ingredientId });
export const saveConversionRule = (id: string | null, input: RuleInput) =>
  invokeNative<ConversionRule>('save_conversion_rule', { id, input });
export const retireConversionRule = (id: string, confirmed: boolean) =>
  invokeNative<void>('retire_conversion_rule', { id, confirmed });
export const ingredientCandidates = (
  names: string[],
  excludeId: string | null,
) => invokeNative<Ingredient[]>('ingredient_candidates', { names, excludeId });
export const createIngredient = (input: IngredientInput) =>
  invokeNative<Ingredient>('create_ingredient', { input });
export const updateIngredient = (id: string, input: IngredientInput) =>
  invokeNative<Ingredient>('update_ingredient', { id, input });
export const retireIngredient = (id: string, confirmed: boolean) =>
  invokeNative<void>('retire_ingredient', { id, confirmed });
export const listMeals = () => invokeNative<Meal[]>('list_meals');
export const getWeek = (date: string) =>
  invokeNative<WeekView>('get_week', { date });
export const applyPlan = (input: PlanMutation) =>
  invokeNative<WeekView>('apply_plan', { input });
export const getMealHistory = (mealId: string) =>
  invokeNative<MealHistory>('get_meal_history', { mealId });
export const changeMealHistory = (input: HistoryMutation) =>
  invokeNative<MealHistory>('change_meal_history', { input });
export const getShopping = (weekStart: string) =>
  invokeNative<ShoppingView>('get_shopping', { weekStart });
export const changeShopping = (input: ShoppingMutation) =>
  invokeNative<ShoppingView>('change_shopping', { input });
export const copyShopping = (weekStart: string, expectedRevision: number) =>
  invokeNative<void>('copy_shopping', { weekStart, expectedRevision });
export const searchMeals = (input: MealSearch) =>
  invokeNative<MealLibrary>('search_meals', { input });
export const saveMealDraft = (
  id: string | null,
  expectedRevision: number | null,
  input: MealDraft,
) => invokeNative<Meal>('save_meal_draft', { id, expectedRevision, input });
export const getMeal = (id: string) => invokeNative<Meal>('get_meal', { id });
export const createMeal = (input: MealInput) =>
  invokeNative<Meal>('create_meal', { input });
export const updateMeal = (
  id: string,
  expectedRevision: number,
  input: MealInput,
) => invokeNative<Meal>('update_meal', { id, expectedRevision, input });
export const retireMeal = (id: string, confirmed: boolean) =>
  invokeNative<void>('retire_meal', { id, confirmed });
export const listFields = () => invokeNative<FieldDefinition[]>('list_fields');
export const createField = (input: FieldInput) =>
  invokeNative<FieldDefinition>('create_field', { input });
export const updateField = (id: string, input: FieldInput) =>
  invokeNative<FieldDefinition>('update_field', { id, input });
export const retireField = (id: string, confirmed: boolean) =>
  invokeNative<void>('retire_field', { id, confirmed });
export const createVocabulary = (
  kind: VocabularyKind,
  name: string,
  sortOrder: number,
) => invokeNative<Vocabulary>('create_vocabulary', { kind, name, sortOrder });
export const updateVocabulary = (
  kind: VocabularyKind,
  id: string,
  name: string,
  sortOrder: number,
) =>
  invokeNative<Vocabulary>('update_vocabulary', { kind, id, name, sortOrder });
export const retireVocabulary = (
  kind: VocabularyKind,
  id: string,
  confirmed: boolean,
) => invokeNative<void>('retire_vocabulary', { kind, id, confirmed });
