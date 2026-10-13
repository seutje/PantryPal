export function validateVersions(
  npm: { version: string },
  tauri: { version: string },
  cargo: string | undefined,
  tag?: string,
): string;
