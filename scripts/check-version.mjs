import { readFileSync } from 'node:fs';
import { fileURLToPath, URL } from 'node:url';
import process from 'node:process';
import console from 'node:console';
import path from 'node:path';
export function validateVersions(npm, tauri, cargo, tag) {
  const version = npm.version;
  if (
    !/^\d+\.\d+\.\d+$/.test(version) ||
    tauri.version !== version ||
    cargo !== version
  )
    throw new Error(
      'package.json, tauri.conf.json and Cargo.toml must have the same stable version.',
    );
  if (tag && tag !== `v${version}`)
    throw new Error('Release tag must exactly match the project version.');
  return version;
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const root = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
  const read = (file) => readFileSync(path.join(root, file), 'utf8');
  const cargo = read('src-tauri/Cargo.toml').match(
    /^version\s*=\s*"([^"]+)"/m,
  )?.[1];
  const version = validateVersions(
    JSON.parse(read('package.json')),
    JSON.parse(read('src-tauri/tauri.conf.json')),
    cargo,
    process.argv[2],
  );
  console.log(
    `PantryPal ${version}: project versions${process.argv[2] ? ' and release tag' : ''} match.`,
  );
}
