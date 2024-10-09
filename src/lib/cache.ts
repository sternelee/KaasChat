/* eslint-disable no-await-in-loop */
/**
 * Util methods for file manipulations in cache folder of App Data
 */
import { BaseDirectory, join, sep } from '@tauri-apps/api/path';
import type { DirEntry } from '@tauri-apps/plugin-fs';
import { readDir, readFile, writeFile } from '@tauri-apps/plugin-fs';

const BASE_PATH = `cache${sep}`;

async function processEntriesRecursively(
  files: DirEntry[],
  parent: string,
  entries: DirEntry[]
) {
  for (const entry of entries) {
    const dir = await join(parent, entry.name);
    if (entry.isDirectory) {
      processEntriesRecursively(
        files,
        dir,
        await readDir(dir, { baseDir: BaseDirectory.AppLocalData })
      );
    } else {
      files.push(entry);
    }
  }
}

/**
 * List all files in a directory
 * @param recursive whether to list subdirectories
 * @returns file entries as DirEntry[]
 */
async function list(): Promise<DirEntry[]> {
  const entries = await readDir(BASE_PATH, {
    baseDir: BaseDirectory.AppData,
  });
  const files: DirEntry[] = [];
  processEntriesRecursively(files, BASE_PATH, entries);
  return files;
}

/**
 * Write a file, of the form of bytes array, to cache dir with given file name
 * @param name file name
 * @param data binary data as Uint8Array
 */
async function write(fileName: string, data: Uint8Array) {
  await writeFile(`${BASE_PATH}${fileName}`, data, {
    baseDir: BaseDirectory.AppData,
  });
}

/**
 * Read a file from cache as bytes array
 * @param fileName
 * @returns
 */
async function read(fileName: string): Promise<Uint8Array> {
  const data = await readFile(`${BASE_PATH}${fileName}`, {
    baseDir: BaseDirectory.AppData,
  });
  return data;
}

/**
 * Read a file from cache as object url
 * @param fileName
 * @param mimetype
 * @returns
 */
async function readObjectUrl(
  fileName: string,
  mimetype: string
): Promise<string> {
  const data = await read(fileName);
  const blob = new Blob([data], { type: mimetype });
  const url = URL.createObjectURL(blob);
  return url;
}

export default {
  list,
  write,
  read,
  readObjectUrl,
};
