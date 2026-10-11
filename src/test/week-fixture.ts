import type { WeekView, Meal } from '../lib/models';
import demo from '../../tests/fixtures/demo-meal.json';
import { catalogFixture } from './catalog-fixture';
export const plannerMeal: Meal = {
  ...demo,
  id: 'meal-planner',
  revision: 1,
  retired: false,
  createdAt: '',
  updatedAt: '',
  baseServingsText: '4',
};
export const weekFixture: WeekView = {
  weekStart: '2026-12-28',
  weekEnd: '2027-01-03',
  days: [
    '2026-12-28',
    '2026-12-29',
    '2026-12-30',
    '2026-12-31',
    '2027-01-01',
    '2027-01-02',
    '2027-01-03',
  ],
  previousWeek: '2026-12-21',
  nextWeek: '2027-01-04',
  revision: 0,
  entries: [],
  library: {
    catalog: catalogFixture,
    fields: [],
    meals: [
      {
        meal: plannerMeal,
        lastCooked: null,
        servingsDisplay: '4',
        quantityDisplays: ['⅓', '500', null],
      },
    ],
  },
};
