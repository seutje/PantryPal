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
];
function read(index: number, transform: (html: string) => string = (s) => s) {
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
      { href: sites[index]!.url },
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
      expect(result.recipe.recipeIngredient).toEqual([
        '½ cup milk',
        'salt to taste',
      ]);
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
