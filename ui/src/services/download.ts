import { save } from '@tauri-apps/plugin-dialog';
import { writeFile, writeTextFile } from '@tauri-apps/plugin-fs';

export interface SaveFilter {
  name: string;
  extensions: string[];
}

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

function blobDownload(filename: string, blob: Blob) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export async function promptAndSaveBytes(
  defaultName: string,
  bytes: Uint8Array,
  filters: SaveFilter[],
): Promise<boolean> {
  if (!isTauri()) {
    blobDownload(defaultName, new Blob([bytes]));
    return true;
  }
  const path = await save({ defaultPath: defaultName, filters });
  if (!path) return false;
  await writeFile(path, bytes);
  return true;
}

export async function promptAndSaveText(
  defaultName: string,
  text: string,
  filters: SaveFilter[],
): Promise<boolean> {
  if (!isTauri()) {
    blobDownload(defaultName, new Blob([text], { type: 'text/plain;charset=utf-8' }));
    return true;
  }
  const path = await save({ defaultPath: defaultName, filters });
  if (!path) return false;
  await writeTextFile(path, text);
  return true;
}
