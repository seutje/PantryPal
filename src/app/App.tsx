import { useQuery } from '@tanstack/react-query';
import { useEffect, useRef, useState } from 'react';
import { WeekPage } from '../features/week/WeekPage';
import { MealsPage } from '../features/meals/MealsPage';
import { ShoppingPage } from '../features/shopping/ShoppingPage';
import { IngredientsPage } from '../features/ingredients/IngredientsPage';
import { SettingsPage } from '../features/settings/SettingsPage';
import { getAppInfo, nativeAvailable } from '../lib/ipc';

const sections = [
  'Week',
  'Meals',
  'Shopping',
  'Ingredients',
  'Settings',
] as const;
type Section = (typeof sections)[number];
const pages = {
  Week: WeekPage,
  Meals: MealsPage,
  Shopping: ShoppingPage,
  Ingredients: IngredientsPage,
  Settings: SettingsPage,
};

export function App() {
  const [section, setSection] = useState<Section>('Week');
  const heading = useRef<HTMLHeadingElement>(null);
  const previousSection = useRef(section);
  const info = useQuery({
    queryKey: ['app-info'],
    queryFn: getAppInfo,
    enabled: nativeAvailable(),
  });
  const Page = pages[section];

  useEffect(() => {
    if (previousSection.current !== section) {
      previousSection.current = section;
      heading.current?.focus();
    }
  }, [section]);

  return (
    <div className="min-h-screen bg-stone-50 text-stone-900">
      <a href="#main-content" className="skip-link">
        Skip to content
      </a>
      <header className="border-b border-stone-200 bg-white px-6 py-5">
        <div className="mx-auto flex max-w-6xl flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-2xl font-bold tracking-tight text-green-800">
              PantryPal
            </p>
            <p className="text-sm text-stone-600">Good meals. Easier weeks.</p>
          </div>
          <p className="text-sm text-stone-600" role="status">
            {!nativeAvailable()
              ? 'Browser preview'
              : info.isError
                ? 'App connection unavailable'
                : info.data
                  ? `Desktop · v${info.data.version}`
                  : 'Connecting to app…'}
          </p>
        </div>
      </header>
      <div className="mx-auto grid max-w-6xl gap-6 p-6 md:grid-cols-[180px_1fr]">
        <nav
          aria-label="Main navigation"
          className="flex flex-wrap gap-1 self-start md:flex-col"
        >
          {sections.map((name) => (
            <button
              key={name}
              type="button"
              aria-current={name === section ? 'page' : undefined}
              onClick={() => setSection(name)}
              className={`rounded-lg px-4 py-3 text-left font-medium ${name === section ? 'bg-green-100 text-green-900' : 'text-stone-600 hover:bg-stone-200'}`}
            >
              {name}
            </button>
          ))}
        </nav>
        <main id="main-content" tabIndex={-1} className="min-w-0">
          <h1
            ref={heading}
            tabIndex={-1}
            className="mb-6 text-3xl font-semibold tracking-tight"
          >
            {section}
          </h1>
          <Page />
        </main>
      </div>
    </div>
  );
}
