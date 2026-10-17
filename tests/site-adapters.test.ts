import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import sites from './fixtures/import-sites.json';

const script = readFileSync('src-tauri/src/importers/capture.js', 'utf8');
const ids = [
  'budget_bytes',
  'allrecipes',
  'spend_with_pennies',
  'recipe_tin_eats',
  'nyt_cooking',
  'cook_smarts',
];
function read(
  index: number,
  transform: (html: string) => string = (s) => s,
  actualUrl = sites[index]!.url,
) {
  document.body.innerHTML = transform(
    readFileSync(`tests/fixtures/site-adapters/${ids[index]}.html`, 'utf8'),
  );
  // JSDOM has no layout/innerText; emulate these browser primitives only in
  // tests. Actual WebView2 visibility is covered by the native live harness.
  Object.defineProperty(HTMLElement.prototype, 'innerText', {
    configurable: true,
    get() {
      return this.textContent;
    },
  });
  Element.prototype.getClientRects = () =>
    [{ width: 10, height: 10 }] as unknown as DOMRectList;
  try {
    const capture = new Function(
      'document',
      'location',
      'Element',
      'getComputedStyle',
      'TextEncoder',
      `return (${script})(${JSON.stringify({ site: ids[index], url: sites[index]!.url })})`,
    );
    return capture(
      document,
      { href: actualUrl },
      Element,
      getComputedStyle,
      TextEncoder,
    );
  } finally {
    document.body.innerHTML = '';
  }
}
describe('authorized website adapters, synthetic fixtures based on inspected publisher structures', () => {
  for (let index = 0; index < sites.length; index++) {
    it(`reads ${ids[index]} visible content and metadata without hidden recipe material`, () => {
      const result = read(index);
      expect(result).toEqual(
        JSON.parse(
          readFileSync(
            `tests/fixtures/site-adapters/${ids[index]}.json`,
            'utf8',
          ),
        ),
      );
      if (index !== 5)
        expect(result.recipe.recipeIngredient).toEqual([
          '½ cup milk',
          'salt to taste',
        ]);
      if (index !== 5)
        expect(result.recipe.recipeInstructions).toEqual([
          'Original fixture preparation.',
          'Original fixture cooking.',
        ]);
      expect(JSON.stringify(result)).not.toContain('Hidden secret instruction');
    });
    it(`rejects ${ids[index]} gated content even when JSON-LD contains a recipe`, () => {
      expect(
        read(index, (html) => `<div id="gateway-content">Sign in</div>${html}`),
      ).toEqual({ error: 'unavailable' });
      expect(read(index, (html) => `<div hidden>${html}</div>`)).toEqual({
        error: 'unavailable',
      });
    });
  }
  it('keeps Cook Smarts sections and excludes account data and alternate methods', () => {
    const result = read(5);
    expect(result.sections[0]).toBe('Roasted Squash (for 2 nights):');
    expect(result.recipe.recipeIngredient[6]).toBe(
      'Oil, olive - 1 Tbsp + 2 Tbsp',
    );
    expect(result.recipe.recipeYield).toBe('4');
    expect(result.recipe.recipeInstructions[0]).toBe('Prep');
    expect(result.recipe.recipeInstructions[2]).toBe('Make');
    expect(JSON.stringify(result)).not.toMatch(/private-|Private review|999 g/);
    expect(result.recipe.prepTime).toBeUndefined();
    expect(result.recipe.cookTime).toBeUndefined();
  });
  it('captures the supplied Budget Bytes olive oil row once, including its price note', () => {
    const result = read(0, (html) =>
      html.replace(
        /<li class="wprm-recipe-ingredient">[\s\S]*?<\/li>/,
        `<li class="wprm-recipe-ingredient" data-uid="4">
           <span class="wprm-checkbox-container"><input type="checkbox" aria-label=" 2 Tbsp olive oil ($0.32)"><label><span class="sr-only">▢ </span></label></span>
           <span class="wprm-recipe-ingredient-unit-system wprm-recipe-ingredient-unit-system-1"><span class="wprm-recipe-ingredient-amount">2</span> <span class="wprm-recipe-ingredient-unit">Tbsp</span></span>
           <span class="wprm-recipe-ingredient-unit-system wprm-recipe-ingredient-unit-system-2"></span>
           <span class="wprm-recipe-ingredient-name">olive oil </span>
           <span class="wprm-recipe-ingredient-notes wprm-recipe-ingredient-notes-normal">($0.32)</span>
         </li>`,
      ),
    );
    expect(result.recipe.recipeIngredient[0]).toBe('2 Tbsp olive oil ($0.32)');
  });
  it('rejects incomplete, collapsed or ambiguous Cook Smarts recipes and changed variants', () => {
    for (const transform of [
      (s: string) => s.replace('id="prepBlock"', 'hidden id="prepBlock"'),
      (s: string) => s.replace('<li>Kale', '<li hidden>Kale'),
      (s: string) => s.replace('id="makeBlock"', 'id="missing"'),
      (s: string) => s.replace('id="ingredients"', 'id="missing"'),
      (s: string) => s + s,
      (s: string) => s.replace('Kale - 5 oz', 'k'.repeat(2001)),
    ])
      expect(read(5, transform)).toEqual({ error: 'unavailable' });
    expect(read(5, undefined, sites[5]!.url.replace('standard', 'gf'))).toEqual(
      { error: 'unavailable' },
    );
    expect(read(5, undefined, sites[5]!.url + '&token=secret')).toEqual({
      error: 'unavailable',
    });
  });
  it('rejects multiple cards, missing steps and oversized ingredients', () => {
    expect(read(0, (s) => s + s)).toEqual({ error: 'unavailable' });
    expect(
      read(0, (s) =>
        s.replaceAll('wprm-recipe-instruction-text', 'not-a-step'),
      ),
    ).toEqual({ error: 'unavailable' });
    expect(read(0, (s) => s.replace('milk', 'm'.repeat(2001)))).toEqual({
      error: 'unavailable',
    });
    expect(
      read(0, (s) =>
        s.replace(
          'class="wprm-recipe-instruction-text"',
          'hidden class="wprm-recipe-instruction-text"',
        ),
      ),
    ).toEqual({ error: 'unavailable' });
  });
  it('omits hidden alternate WPRM units and does not default to schema ingredient amounts', () => {
    const result = read(0, (s) =>
      s.replace(
        '<span class="wprm-recipe-ingredient-name">milk</span>',
        '<span hidden class="wprm-recipe-ingredient-amount">999</span><span class="wprm-recipe-ingredient-name">milk</span>',
      ),
    );
    expect(result.recipe.recipeIngredient[0]).toBe('½ cup milk');
  });
});
