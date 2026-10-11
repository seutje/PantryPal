function captureRecipe(spec) {
  // Fixed native-host script, never exposed as an arbitrary-JavaScript IPC API.
  // Only visible recipe text and a small projection of recipe metadata leave
  // the browser. No HTML, cookies, credentials, hidden app state or requests.
  const fail = () => ({ error: 'unavailable' });
  try {
    const expected = new URL(spec.url);
    const current = new URL(location.href);
    if (
      current.origin !== expected.origin ||
      current.pathname.replace(/\/$/, '') !==
        expected.pathname.replace(/\/$/, '') ||
      current.search ||
      current.hash
    )
      return fail();
    const visible = (e) => {
      for (let n = e; n && n instanceof Element; n = n.parentElement) {
        const s = getComputedStyle(n);
        if (
          n.hidden ||
          s.display === 'none' ||
          s.visibility === 'hidden' ||
          s.opacity === '0'
        )
          return false;
      }
      return !!e?.getClientRects().length;
    };
    const nodes = (root, selector) =>
      [...root.querySelectorAll(selector)].filter(visible);
    const completeNodes = (root, selector) => {
      const all = [...root.querySelectorAll(selector)];
      if (all.some((e) => !visible(e))) throw new Error('Gated recipe');
      return all;
    };
    const text = (e) => (e?.innerText || '').trim();
    if (
      nodes(
        document,
        '#gateway-content, [data-testid="paywall"], [data-testid="gateway"], [class*="paywall"], [class*="Paywall"]',
      ).length
    )
      return fail();
    let title,
      ingredients,
      steps,
      yieldText = '';
    if (
      ['budget_bytes', 'spend_with_pennies', 'recipe_tin_eats'].includes(
        spec.site,
      )
    ) {
      const cards = nodes(document, '.wprm-recipe-container');
      if (cards.length !== 1) return fail();
      const card = cards[0];
      title = text(nodes(card, '.wprm-recipe-name')[0]);
      // innerText excludes hidden alternate units and checkbox screen-reader
      // labels; use structured spans to avoid repeated accessible labels.
      ingredients = completeNodes(card, '.wprm-recipe-ingredient').map((e) => {
        const parts = nodes(
          e,
          '.wprm-recipe-ingredient-amount, .wprm-recipe-ingredient-unit, .wprm-recipe-ingredient-name, .wprm-recipe-ingredient-notes',
        );
        return parts.map(text).filter(Boolean).join(' ');
      });
      steps = completeNodes(card, '.wprm-recipe-instruction-text').map(text);
      yieldText = text(nodes(card, '.wprm-recipe-servings')[0]);
    } else if (spec.site === 'allrecipes') {
      title = text(nodes(document, 'h1.article-heading')[0]);
      ingredients = completeNodes(
        document,
        '.mm-recipes-structured-ingredients__list-item',
      ).map(text);
      steps = completeNodes(
        document,
        '.mm-recipes-steps__content > ol > li > p',
      ).map(text);
      const servings = nodes(document, '.mm-recipes-details__item').find((e) =>
        /^Servings:?$/i.test(
          text(e.querySelector('.mm-recipes-details__label')),
        ),
      );
      yieldText = text(servings?.querySelector('.mm-recipes-details__value'));
    } else if (spec.site === 'nyt_cooking') {
      const bodies = nodes(document, '[data-testid="recipe-body"]');
      if (bodies.length !== 1) return fail();
      const body = bodies[0];
      title = text(nodes(document, 'h1')[0]);
      ingredients = completeNodes(
        body,
        '[class*="ingredients_ingredients__"] > ul > li',
      ).map(text);
      steps = completeNodes(body, '[class*="preparation_stepContent__"]').map(
        text,
      );
      yieldText = text(nodes(body, '[class*="ingredients_recipeYield__"]')[0]);
    } else return fail();
    if (
      !title ||
      !ingredients?.length ||
      !steps?.length ||
      ingredients.length > 300 ||
      steps.length > 300 ||
      ingredients.some((s) => !s || s.length > 2000) ||
      steps.some((s) => !s) ||
      title.length > 200 ||
      yieldText.length > 200
    )
      return fail();
    // Prefer authorized JSON-LD for attribution/times only. Ingredients and
    // instructions always come from visible DOM, including adjusted servings.
    const recipes = [];
    const walk = (v, depth) => {
      if (depth > 24) return;
      if (Array.isArray(v)) v.forEach((x) => walk(x, depth + 1));
      else if (v && typeof v === 'object') {
        const types = [].concat(v['@type'] || []);
        if (
          types.includes('Recipe') ||
          types.includes('https://schema.org/Recipe')
        )
          recipes.push(v);
        if (v['@graph']) walk(v['@graph'], depth + 1);
      }
    };
    const scripts = document.querySelectorAll(
      'script[type="application/ld+json"]',
    );
    if (scripts.length > 32) return fail();
    let bytes = 0;
    for (const s of scripts) {
      bytes += s.textContent.length;
      if (bytes > 262144) return fail();
      try {
        walk(JSON.parse(s.textContent), 0);
      } catch {
        /* DOM fallback */
      }
    }
    const metadata = recipes.length === 1 ? recipes[0] : {};
    const authors = []
      .concat(metadata.author || [])
      .map((a) => (typeof a === 'string' ? a : a?.name))
      .filter((a) => typeof a === 'string')
      .join(', ');
    const recipe = {
      '@type': 'Recipe',
      name: title,
      recipeYield: yieldText,
      recipeIngredient: ingredients,
      recipeInstructions: steps,
      author: authors,
    };
    for (const key of ['prepTime', 'cookTime']) {
      if (typeof metadata[key] === 'string' && metadata[key].length <= 100)
        recipe[key] = metadata[key];
    }
    const result = { version: 1, site: spec.site, recipe };
    return new TextEncoder().encode(JSON.stringify(result)).length <= 262144
      ? result
      : fail();
  } catch {
    return fail();
  }
}
