import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import config from '../src-tauri/tauri.conf.json';
import { validateVersions } from '../scripts/check-version.mjs';
it('rejects mismatched project versions, tags and unstable first release versions', () => {
  expect(
    validateVersions(
      { version: '0.1.0' },
      { version: '0.1.0' },
      '0.1.0',
      'v0.1.0',
    ),
  ).toBe('0.1.0');
  for (const args of [
    [{ version: '0.1.0' }, { version: '0.1.1' }, '0.1.0', 'v0.1.0'],
    [{ version: '0.1.0' }, { version: '0.1.0' }, '0.1.0', 'v0.2.0'],
    [
      { version: '0.1.0-beta' },
      { version: '0.1.0-beta' },
      '0.1.0-beta',
      'v0.1.0-beta',
    ],
  ] as const)
    expect(() =>
      validateVersions(args[0], args[1], args[2], args[3]),
    ).toThrow();
});
it('uses a fixed HTTPS PantryPal feed and signing key with per-user NSIS artifacts', () => {
  expect(config.plugins.updater.endpoints).toEqual([
    'https://github.com/seutje/PantryPal/releases/latest/download/latest.json',
  ]);
  expect(
    Buffer.from(config.plugins.updater.pubkey, 'base64').toString(),
  ).toContain('minisign public key');
  expect(config.bundle.createUpdaterArtifacts).toBe(true);
  expect(config.bundle.windows.nsis.installMode).toBe('currentUser');
  expect(config.plugins.updater.windows.installMode).toBe('passive');
  const workflow = readFileSync('.github/workflows/release.yml', 'utf8');
  expect(workflow).toContain('includeUpdaterJson: true');
  expect(workflow).not.toContain('uploadUpdaterJson:');
  expect(workflow).toContain('TAURI_SIGNING_PRIVATE_KEY');
  expect(workflow).toContain('version:check');
  expect(workflow).toContain('releaseDraft: true');
});
