/**
 * Mock implementation of @tauri-apps/plugin-dialog.
 * Returns fake file paths for save/open dialogs.
 */

export async function save(options?: {
  defaultPath?: string;
  filters?: Array<{ name: string; extensions: string[] }>;
}): Promise<string | null> {
  const ext = options?.filters?.[0]?.extensions?.[0] ?? "txt";
  return `/tmp/mock-save.${ext}`;
}

export async function open(options?: {
  directory?: boolean;
  filters?: Array<{ name: string; extensions: string[] }>;
}): Promise<string | string[] | null> {
  if (options?.directory) {
    return "/tmp/mock-directory";
  }
  return "/tmp/mock-open.txt";
}
