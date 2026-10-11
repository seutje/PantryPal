import { readFileSync, readdirSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import config from '../src-tauri/tauri.conf.json';
import main from '../src-tauri/capabilities/main.json';

describe('native permission boundary', () => {
  const commands = [
    'get_app_info',
    'get_storage_status',
    'get_catalog',
    'create_ingredient',
    'update_ingredient',
    'retire_ingredient',
    'list_meals',
    'get_meal',
    'create_meal',
    'update_meal',
    'retire_meal',
    'list_fields',
    'create_field',
    'update_field',
    'retire_field',
    'create_vocabulary',
    'update_vocabulary',
    'retire_vocabulary',
    'get_measurement_preferences',
    'save_measurement_preferences',
    'convert_measurement',
    'list_conversion_rules',
    'save_conversion_rule',
    'retire_conversion_rule',
    'ingredient_candidates',
    'search_meals',
    'save_meal_draft',
    'get_week',
    'apply_plan',
    'get_meal_history',
    'change_meal_history',
    'get_shopping',
    'change_shopping',
    'copy_shopping',
    'inspect_recipe_url',
    'preview_recipe_import',
    'confirm_recipe_import',
    'open_recipe_browser',
    'reset_recipe_browser',
  ];
  it('grants only named repository operations to the local main window', () => {
    expect(config.app.security.capabilities).toEqual(['main']);
    expect(readdirSync('src-tauri/capabilities')).toEqual(['main.json']);
    expect(main).toMatchObject({
      local: true,
      windows: ['main'],
      permissions: commands.map(
        (command) => `allow-${command.replaceAll('_', '-')}`,
      ),
    });
    expect(main).not.toHaveProperty('remote');
    expect(main).not.toHaveProperty('webviews');
    expect(config.app.withGlobalTauri).toBe(false);
    expect(config.app.windows.map((window) => window.label)).toEqual(['main']);
  });

  it('subjects every registered application command to ACL', () => {
    const build = readFileSync('src-tauri/build.rs', 'utf8');
    const lib = readFileSync('src-tauri/src/lib.rs', 'utf8');
    const registered = lib
      .match(/generate_handler!\[([^\]]+)\]/)?.[1]
      ?.split(',')
      .filter((name) => name.trim().length > 0)
      .map((name) => name.trim().replace('commands::', ''));
    expect(registered).toEqual(commands);
    const manifest = build
      .match(/\.commands\(&\[([\s\S]*?)\]\)/)?.[1]
      ?.match(/"([a-z_]+)"/g)
      ?.map((command) => command.replaceAll('"', ''));
    expect(manifest).toEqual(commands);
  });

  it('blocks remote scripts, frames and network in the production shell', () => {
    expect(config.app.security.csp).toContain("script-src 'self'");
    expect(config.app.security.csp).toContain("frame-src 'none'");
    expect(config.app.security.csp).toContain("object-src 'none'");
    expect(config.app.security.csp).toContain(
      'connect-src ipc: http://ipc.localhost;',
    );
    expect(config.app.security.csp).not.toMatch(/https:|unsafe-eval|\*/);
  });
});
