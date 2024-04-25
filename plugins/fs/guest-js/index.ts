// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the file system.
 *
 * ## Security
 *
 * This module prevents path traversal, not allowing absolute paths or parent dir components
 * (i.e. "/usr/path/to/file" or "../path/to/file" paths are not allowed).
 * Paths accessed with this API must be relative to one of the {@link BaseDirectory | base directories}
 * so if you need access to arbitrary filesystem paths, you must write such logic on the core layer instead.
 *
 * The API has a scope configuration that forces you to restrict the paths that can be accessed using glob patterns.
 *
 * The scope configuration is an array of glob patterns describing folder paths that are allowed.
 * For instance, this scope configuration only allows accessing files on the
 * *databases* folder of the {@link https://beta.tauri.app/2/reference/js/core/namespacepath/#appdatadir | `$APPDATA` directory}:
 * ```json
 * {
 *   "plugins": {
 *     "fs": {
 *       "scope": ["$APPDATA/databases/*"]
 *     }
 *   }
 * }
 * ```
 *
 * Notice the use of the `$APPDATA` variable. The value is injected at runtime, resolving to the {@link https://beta.tauri.app/2/reference/js/core/namespacepath/#appdatadir | app data directory}.
 *
 * The available variables are:
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#appconfigdir | $APPCONFIG},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#appdatadir | $APPDATA},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#appLocaldatadir | $APPLOCALDATA},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#appcachedir | $APPCACHE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#applogdir | $APPLOG},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#audiodir | $AUDIO},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#cachedir | $CACHE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#configdir | $CONFIG},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#datadir | $DATA},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#localdatadir | $LOCALDATA},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#desktopdir | $DESKTOP},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#documentdir | $DOCUMENT},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#downloaddir | $DOWNLOAD},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#executabledir | $EXE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#fontdir | $FONT},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#homedir | $HOME},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#picturedir | $PICTURE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#publicdir | $PUBLIC},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#runtimedir | $RUNTIME},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#templatedir | $TEMPLATE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#videodir | $VIDEO},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#resourcedir | $RESOURCE},
 * {@linkcode https://beta.tauri.app/2/reference/js/core/namespacepath/#tempdir | $TEMP}.
 *
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * Note that this scope applies to **all** APIs on this module.
 *
 * @module
 */

import { BaseDirectory } from "@tauri-apps/api/path";
import { Channel, invoke, Resource } from "@tauri-apps/api/core";

enum SeekMode {
  Start = 0,
  Current = 1,
  End = 2,
}

/**
 * A FileInfo describes a file and is returned by `stat`, `lstat` or `fstat`.
 *
 * @since 2.0.0
 */
interface FileInfo {
  /**
   * True if this is info for a regular file. Mutually exclusive to
   * `FileInfo.isDirectory` and `FileInfo.isSymlink`.
   */
  isFile: boolean;
  /**
   * True if this is info for a regular directory. Mutually exclusive to
   * `FileInfo.isFile` and `FileInfo.isSymlink`.
   */
  isDirectory: boolean;
  /**
   * True if this is info for a symlink. Mutually exclusive to
   * `FileInfo.isFile` and `FileInfo.isDirectory`.
   */
  isSymlink: boolean;
  /**
   * The size of the file, in bytes.
   */
  size: number;
  /**
   * The last modification time of the file. This corresponds to the `mtime`
   * field from `stat` on Linux/Mac OS and `ftLastWriteTime` on Windows. This
   * may not be available on all platforms.
   */
  mtime: Date | null;
  /**
   * The last access time of the file. This corresponds to the `atime`
   * field from `stat` on Unix and `ftLastAccessTime` on Windows. This may not
   * be available on all platforms.
   */
  atime: Date | null;
  /**
   * The creation time of the file. This corresponds to the `birthtime`
   * field from `stat` on Mac/BSD and `ftCreationTime` on Windows. This may
   * not be available on all platforms.
   */
  birthtime: Date | null;
  /** Whether this is a readonly (unwritable) file. */
  readonly: boolean;
  /**
   * This field contains the file system attribute information for a file
   * or directory. For possible values and their descriptions, see
   * {@link https://docs.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants | File Attribute Constants} in the Windows Dev Center
   *
   * #### Platform-specific
   *
   * - **macOS / Linux / Android / iOS:** Unsupported.
   */
  fileAttributes: number | null;
  /**
   * ID of the device containing the file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  dev: number | null;
  /**
   * Inode number.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  ino: number | null;
  /**
   * The underlying raw `st_mode` bits that contain the standard Unix
   * permissions for this file/directory.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  mode: number | null;
  /**
   * Number of hard links pointing to this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  nlink: number | null;
  /**
   * User ID of the owner of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  uid: number | null;
  /**
   * Group ID of the owner of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  gid: number | null;
  /**
   * Device ID of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  rdev: number | null;
  /**
   * Blocksize for filesystem I/O.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  blksize: number | null;
  /**
   * Number of blocks allocated to the file, in 512-byte units.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  blocks: number | null;
}

interface UnparsedFileInfo {
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
  size: number;
  mtime: number | null;
  atime: number | null;
  birthtime: number | null;
  readonly: boolean;
  fileAttributes: number;
  dev: number | null;
  ino: number | null;
  mode: number | null;
  nlink: number | null;
  uid: number | null;
  gid: number | null;
  rdev: number | null;
  blksize: number | null;
  blocks: number | null;
}
function parseFileInfo(r: UnparsedFileInfo): FileInfo {
  return {
    isFile: r.isFile,
    isDirectory: r.isDirectory,
    isSymlink: r.isSymlink,
    size: r.size,
    mtime: r.mtime != null ? new Date(r.mtime) : null,
    atime: r.atime != null ? new Date(r.atime) : null,
    birthtime: r.birthtime != null ? new Date(r.birthtime) : null,
    readonly: r.readonly,
    fileAttributes: r.fileAttributes,
    dev: r.dev,
    ino: r.ino,
    mode: r.mode,
    nlink: r.nlink,
    uid: r.uid,
    gid: r.gid,
    rdev: r.rdev,
    blksize: r.blksize,
    blocks: r.blocks,
  };
}

/**
 *  The Tauri abstraction for reading and writing files.
 *
 * @since 2.0.0
 */
class FileHandle extends Resource {
  /**
   * Reads up to `p.byteLength` bytes into `p`. It resolves to the number of
   * bytes read (`0` < `n` <= `p.byteLength`) and rejects if any error
   * encountered. Even if `read()` resolves to `n` < `p.byteLength`, it may
   * use all of `p` as scratch space during the call. If some data is
   * available but not `p.byteLength` bytes, `read()` conventionally resolves
   * to what is available instead of waiting for more.
   *
   * When `read()` encounters end-of-file condition, it resolves to EOF
   * (`null`).
   *
   * When `read()` encounters an error, it rejects with an error.
   *
   * Callers should always process the `n` > `0` bytes returned before
   * considering the EOF (`null`). Doing so correctly handles I/O errors that
   * happen after reading some bytes and also both of the allowed EOF
   * behaviors.
   *
   * @example
   * ```typescript
   * import { open, read, close, BaseDirectory } from "@tauri-apps/plugin-fs"
   * // if "$APP/foo/bar.txt" contains the text "hello world":
   * const file = await open("foo/bar.txt", { baseDir: BaseDirectory.App });
   * const buf = new Uint8Array(100);
   * const numberOfBytesRead = await file.read(buf); // 11 bytes
   * const text = new TextDecoder().decode(buf);  // "hello world"
   * await close(file.rid);
   * ```
   *
   * @since 2.0.0
   */
  async read(buffer: Uint8Array): Promise<number | null> {
    if (buffer.byteLength === 0) {
      return 0;
    }

    const [data, nread] = await invoke<[number[], number]>("plugin:fs|read", {
      rid: this.rid,
      len: buffer.byteLength,
    });

    buffer.set(data);

    return nread === 0 ? null : nread;
  }

  /**
   * Seek sets the offset for the next `read()` or `write()` to offset,
   * interpreted according to `whence`: `Start` means relative to the
   * start of the file, `Current` means relative to the current offset,
   * and `End` means relative to the end. Seek resolves to the new offset
   * relative to the start of the file.
   *
   * Seeking to an offset before the start of the file is an error. Seeking to
   * any positive offset is legal, but the behavior of subsequent I/O
   * operations on the underlying object is implementation-dependent.
   * It returns the number of cursor position.
   *
   * @example
   * ```typescript
   * import { open, seek, write, SeekMode, BaseDirectory } from '@tauri-apps/plugin-fs';
   *
   * // Given hello.txt pointing to file with "Hello world", which is 11 bytes long:
   * const file = await open('hello.txt', { read: true, write: true, truncate: true, create: true, baseDir: BaseDirectory.App });
   * await file.write(new TextEncoder().encode("Hello world"), { baseDir: BaseDirectory.App });
   *
   * // Seek 6 bytes from the start of the file
   * console.log(await file.seek(6, SeekMode.Start)); // "6"
   * // Seek 2 more bytes from the current position
   * console.log(await file.seek(2, SeekMode.Current)); // "8"
   * // Seek backwards 2 bytes from the end of the file
   * console.log(await file.seek(-2, SeekMode.End)); // "9" (e.g. 11-2)
   * ```
   *
   * @since 2.0.0
   */
  async seek(offset: number, whence: SeekMode): Promise<number> {
    return await invoke("plugin:fs|seek", {
      rid: this.rid,
      offset,
      whence,
    });
  }

  /**
   * Returns a {@linkcode FileInfo } for this file.
   *
   * @example
   * ```typescript
   * import { open, fstat, BaseDirectory } from '@tauri-apps/plugin-fs';
   * const file = await open("file.txt", { read: true, baseDir: BaseDirectory.App });
   * const fileInfo = await fstat(file.rid);
   * console.log(fileInfo.isFile); // true
   * ```
   *
   * @since 2.0.0
   */
  async stat(): Promise<FileInfo> {
    const res = await invoke<UnparsedFileInfo>("plugin:fs|fstat", {
      rid: this.rid,
    });

    return parseFileInfo(res);
  }

  /**
   * Truncates or extends this file, to reach the specified `len`.
   * If `len` is not specified then the entire file contents are truncated.
   *
   * @example
   * ```typescript
   * import { ftruncate, open, write, read, BaseDirectory } from '@tauri-apps/plugin-fs';
   *
   * // truncate the entire file
   * const file = await open("my_file.txt", { read: true, write: true, create: true, baseDir: BaseDirectory.App });
   * await ftruncate(file.rid);
   *
   * // truncate part of the file
   * const file = await open("my_file.txt", { read: true, write: true, create: true, baseDir: BaseDirectory.App });
   * await write(file.rid, new TextEncoder().encode("Hello World"));
   * await ftruncate(file.rid, 7);
   * const data = new Uint8Array(32);
   * await read(file.rid, data);
   * console.log(new TextDecoder().decode(data)); // Hello W
   * ```
   *
   * @since 2.0.0
   */
  async truncate(len?: number): Promise<void> {
    await invoke("plugin:fs|ftruncate", {
      rid: this.rid,
      len,
    });
  }

  /**
   * Writes `p.byteLength` bytes from `p` to the underlying data stream. It
   * resolves to the number of bytes written from `p` (`0` <= `n` <=
   * `p.byteLength`) or reject with the error encountered that caused the
   * write to stop early. `write()` must reject with a non-null error if
   * would resolve to `n` < `p.byteLength`. `write()` must not modify the
   * slice data, even temporarily.
   *
   * @example
   * ```typescript
   * import { open, write, close, BaseDirectory } from '@tauri-apps/plugin-fs';
   * const encoder = new TextEncoder();
   * const data = encoder.encode("Hello world");
   * const file = await open("bar.txt", { write: true, baseDir: BaseDirectory.App });
   * const bytesWritten = await write(file.rid, data); // 11
   * await close(file.rid);
   * ```
   *
   * @since 2.0.0
   */
  async write(data: Uint8Array): Promise<number> {
    return await invoke("plugin:fs|write", {
      rid: this.rid,
      data: Array.from(data),
    });
  }
}

/**
 * @since 2.0.0
 */
interface CreateOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Creates a file if none exists or truncates an existing file and resolves to
 *  an instance of {@linkcode FileHandle }.
 *
 * @example
 * ```typescript
 * import { create, BaseDirectory } from "@tauri-apps/plugin-fs"
 * const file = await create("foo/bar.txt", { baseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function create(
  path: string | URL,
  options?: CreateOptions,
): Promise<FileHandle> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  const rid = await invoke<number>("plugin:fs|create", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });

  return new FileHandle(rid);
}

/**
 * @since 2.0.0
 */
interface OpenOptions {
  /**
   * Sets the option for read access. This option, when `true`, means that the
   * file should be read-able if opened.
   */
  read?: boolean;
  /**
   * Sets the option for write access. This option, when `true`, means that
   * the file should be write-able if opened. If the file already exists,
   * any write calls on it will overwrite its contents, by default without
   * truncating it.
   */
  write?: boolean;
  /**
   * Sets the option for the append mode. This option, when `true`, means that
   * writes will append to a file instead of overwriting previous contents.
   * Note that setting `{ write: true, append: true }` has the same effect as
   * setting only `{ append: true }`.
   */
  append?: boolean;
  /**
   * Sets the option for truncating a previous file. If a file is
   * successfully opened with this option set it will truncate the file to `0`
   * size if it already exists. The file must be opened with write access
   * for truncate to work.
   */
  truncate?: boolean;
  /**
   * Sets the option to allow creating a new file, if one doesn't already
   * exist at the specified path. Requires write or append access to be
   * used.
   */
  create?: boolean;
  /**
   * Defaults to `false`. If set to `true`, no file, directory, or symlink is
   * allowed to exist at the target location. Requires write or append
   * access to be used. When createNew is set to `true`, create and truncate
   * are ignored.
   */
  createNew?: boolean;
  /**
   * Permissions to use if creating the file (defaults to `0o666`, before
   * the process's umask).
   * Ignored on Windows.
   */
  mode?: number;
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Open a file and resolve to an instance of {@linkcode FileHandle}. The
 * file does not need to previously exist if using the `create` or `createNew`
 * open options. It is the callers responsibility to close the file when finished
 * with it.
 *
 * @example
 * ```typescript
 * import { open, BaseDirectory } from "@tauri-apps/plugin-fs"
 * const file = await open("foo/bar.txt", { read: true, write: true, baseDir: BaseDirectory.App });
 * // Do work with file
 * await close(file.rid);
 * ```
 *
 * @since 2.0.0
 */
async function open(
  path: string | URL,
  options?: OpenOptions,
): Promise<FileHandle> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  const rid = await invoke<number>("plugin:fs|open", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });

  return new FileHandle(rid);
}

/**
 * @since 2.0.0
 */
interface CopyFileOptions {
  /** Base directory for `fromPath`. */
  fromPathBaseDir?: BaseDirectory;
  /** Base directory for `toPath`. */
  toPathBaseDir?: BaseDirectory;
}

/**
 * Copies the contents and permissions of one file to another specified path, by default creating a new file if needed, else overwriting.
 * @example
 * ```typescript
 * import { copyFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * await copyFile('app.conf', 'app.conf.bk', { fromPathBaseDir: BaseDirectory.App, toPathBaseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function copyFile(
  fromPath: string | URL,
  toPath: string | URL,
  options?: CopyFileOptions,
): Promise<void> {
  if (
    (fromPath instanceof URL && fromPath.protocol !== "file:") ||
    (toPath instanceof URL && toPath.protocol !== "file:")
  ) {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|copy_file", {
    fromPath: fromPath instanceof URL ? fromPath.toString() : fromPath,
    toPath: toPath instanceof URL ? toPath.toString() : toPath,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface MkdirOptions {
  /** Permissions to use when creating the directory (defaults to `0o777`, before the process's umask). Ignored on Windows. */
  mode?: number;
  /**
   * Defaults to `false`. If set to `true`, means that any intermediate directories will also be created (as with the shell command `mkdir -p`).
   * */
  recursive?: boolean;
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Creates a new directory with the specified path.
 * @example
 * ```typescript
 * import { mkdir, BaseDirectory } from '@tauri-apps/plugin-fs';
 * await mkdir('users', { baseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function mkdir(
  path: string | URL,
  options?: MkdirOptions,
): Promise<void> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|mkdir", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface ReadDirOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * A disk entry which is either a file, a directory or a symlink.
 *
 * This is the result of the {@linkcode readDir}.
 *
 * @since 2.0.0
 */
interface DirEntry {
  /** The name of the entry (file name with extension or directory name). */
  name: string;
  /** Specifies whether this entry is a directory or not. */
  isDirectory: boolean;
  /** Specifies whether this entry is a file or not. */
  isFile: boolean;
  /** Specifies whether this entry is a symlink or not. */
  isSymlink: boolean;
}

/**
 * Reads the directory given by path and returns an array of `DirEntry`.
 * @example
 * ```typescript
 * import { readDir, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const dir = "users"
 * const entries = await readDir('users', { baseDir: BaseDirectory.App });
 * processEntriesRecursive(dir, entries);
 * async function processEntriesRecursive(parent, entries) {
 *   for (const entry of entries) {
 *     console.log(`Entry: ${entry.name}`);
 *     if (entry.isDirectory) {
 *        const dir = parent + entry.name;
 *       processEntriesRecursive(dir, await readDir(dir, { baseDir: BaseDirectory.App }))
 *     }
 *   }
 * }
 * ```
 *
 * @since 2.0.0
 */
async function readDir(
  path: string | URL,
  options?: ReadDirOptions,
): Promise<DirEntry[]> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  return await invoke("plugin:fs|read_dir", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface ReadFileOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Reads and resolves to the entire contents of a file as an array of bytes.
 * TextDecoder can be used to transform the bytes to string if required.
 * @example
 * ```typescript
 * import { readFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const contents = await readFile('avatar.png', { baseDir: BaseDirectory.Resource });
 * ```
 *
 * @since 2.0.0
 */
async function readFile(
  path: string | URL,
  options?: ReadFileOptions,
): Promise<Uint8Array> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  const arr = await invoke<number[]>("plugin:fs|read_file", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });

  return Uint8Array.from(arr);
}

/**
 * Reads and returns the entire contents of a file as UTF-8 string.
 * @example
 * ```typescript
 * import { readTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const contents = await readTextFile('app.conf', { baseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function readTextFile(
  path: string | URL,
  options?: ReadFileOptions,
): Promise<string> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  return await invoke<string>("plugin:fs|read_text_file", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });
}

/**
 * Returns an async {@linkcode AsyncIterableIterator} over the lines of a file as UTF-8 string.
 * @example
 * ```typescript
 * import { readTextFileLines, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const lines = await readTextFileLines('app.conf', { baseDir: BaseDirectory.App });
 * for await (const line of lines) {
 *   console.log(line);
 * }
 * ```
 * You could also call {@linkcode AsyncIterableIterator.next} to advance the
 * iterator so you can lazily read the next line whenever you want.
 *
 * @since 2.0.0
 */
async function readTextFileLines(
  path: string | URL,
  options?: ReadFileOptions,
): Promise<AsyncIterableIterator<string>> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  const pathStr = path instanceof URL ? path.toString() : path;

  return await Promise.resolve({
    path: pathStr,
    rid: null as number | null,
    async next(): Promise<IteratorResult<string>> {
      if (this.rid == null) {
        this.rid = await invoke<number>("plugin:fs|read_text_file_lines", {
          path: pathStr,
          options,
        });
      }

      const [line, done] = await invoke<[string | null, boolean]>(
        "plugin:fs|read_text_file_lines_next",
        { rid: this.rid },
      );

      // an iteration is over, reset rid for next iteration
      if (done) this.rid = null;

      return {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        value: done ? "" : line!,
        done,
      };
    },
    [Symbol.asyncIterator](): AsyncIterableIterator<string> {
      return this;
    },
  });
}

/**
 * @since 2.0.0
 */
interface RemoveOptions {
  /** Defaults to `false`. If set to `true`, path will be removed even if it's a non-empty directory. */
  recursive?: boolean;
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Removes the named file or directory.
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { remove, BaseDirectory } from '@tauri-apps/plugin-fs';
 * await remove('users/file.txt', { baseDir: BaseDirectory.App });
 * await remove('users', { baseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function remove(
  path: string | URL,
  options?: RemoveOptions,
): Promise<void> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|remove", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface RenameOptions {
  /** Base directory for `oldPath`. */
  oldPathBaseDir?: BaseDirectory;
  /** Base directory for `newPath`. */
  newPathBaseDir?: BaseDirectory;
}

/**
 * Renames (moves) oldpath to newpath. Paths may be files or directories.
 * If newpath already exists and is not a directory, rename() replaces it.
 * OS-specific restrictions may apply when oldpath and newpath are in different directories.
 *
 * On Unix, this operation does not follow symlinks at either path.
 *
 * @example
 * ```typescript
 * import { rename, BaseDirectory } from '@tauri-apps/plugin-fs';
 * await rename('avatar.png', 'deleted.png', { oldPathBaseDir: BaseDirectory.App, newPathBaseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function rename(
  oldPath: string | URL,
  newPath: string | URL,
  options?: RenameOptions,
): Promise<void> {
  if (
    (oldPath instanceof URL && oldPath.protocol !== "file:") ||
    (newPath instanceof URL && newPath.protocol !== "file:")
  ) {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|rename", {
    oldPath: oldPath instanceof URL ? oldPath.toString() : oldPath,
    newPath: newPath instanceof URL ? newPath.toString() : newPath,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface StatOptions {
  /** Base directory for `path`. */
  baseDir?: BaseDirectory;
}

/**
 * Resolves to a {@linkcode FileInfo} for the specified `path`. Will always
 * follow symlinks but will reject if the symlink points to a path outside of the scope.
 *
 * @example
 * ```typescript
 * import { stat, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const fileInfo = await stat("hello.txt", { baseDir: BaseDirectory.App });
 * console.log(fileInfo.isFile); // true
 * ```
 *
 * @since 2.0.0
 */
async function stat(
  path: string | URL,
  options?: StatOptions,
): Promise<FileInfo> {
  const res = await invoke<UnparsedFileInfo>("plugin:fs|stat", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });

  return parseFileInfo(res);
}

/**
 * Resolves to a {@linkcode FileInfo} for the specified `path`. If `path` is a
 * symlink, information for the symlink will be returned instead of what it
 * points to.
 *
 * @example
 * ```typescript
 * import { lstat, BaseDirectory } from '@tauri-apps/plugin-fs';
 * const fileInfo = await lstat("hello.txt", { baseDir: BaseDirectory.App });
 * console.log(fileInfo.isFile); // true
 * ```
 *
 * @since 2.0.0
 */
async function lstat(
  path: string | URL,
  options?: StatOptions,
): Promise<FileInfo> {
  const res = await invoke<UnparsedFileInfo>("plugin:fs|lstat", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });

  return parseFileInfo(res);
}

/**
 * @since 2.0.0
 */
interface TruncateOptions {
  /** Base directory for `path`. */
  baseDir?: BaseDirectory;
}

/**
 * Truncates or extends the specified file, to reach the specified `len`.
 * If `len` is `0` or not specified, then the entire file contents are truncated.
 *
 * @example
 * ```typescript
 * import { truncate, readFile, writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // truncate the entire file
 * await truncate("my_file.txt", 0, { baseDir: BaseDirectory.App });
 *
 * // truncate part of the file
 * let file = "file.txt";
 * await writeFile(file, new TextEncoder().encode("Hello World"), { baseDir: BaseDirectory.App });
 * await truncate(file, 7);
 * const data = await readFile(file, { baseDir: BaseDirectory.App });
 * console.log(new TextDecoder().decode(data));  // "Hello W"
 * ```
 *
 * @since 2.0.0
 */
async function truncate(
  path: string | URL,
  len?: number,
  options?: TruncateOptions,
): Promise<void> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|truncate", {
    path: path instanceof URL ? path.toString() : path,
    len,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface WriteFileOptions {
  /** Defaults to `false`. If set to `true`, will append to a file instead of overwriting previous contents. */
  append?: boolean;
  /** Sets the option to allow creating a new file, if one doesn't already exist at the specified path (defaults to `true`). */
  create?: boolean;
  /** Sets the option to create a new file, failing if it already exists. */
  createNew?: boolean;
  /** File permissions. Ignored on Windows. */
  mode?: number;
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * Write `data` to the given `path`, by default creating a new file if needed, else overwriting.
 * @example
 * ```typescript
 * import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
 *
 * let encoder = new TextEncoder();
 * let data = encoder.encode("Hello World");
 * await writeFile('file.txt', data, { baseDir: BaseDirectory.App });
 * ```
 *
 * @since 2.0.0
 */
async function writeFile(
  path: string | URL,
  data: Uint8Array,
  options?: WriteFileOptions,
): Promise<void> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|write_file", {
    path: path instanceof URL ? path.toString() : path,
    data: Array.from(data),
    options,
  });
}

/**
  * Writes UTF-8 string `data` to the given `path`, by default creating a new file if needed, else overwriting.
    @example
  * ```typescript
  * import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs';
  *
  * await writeTextFile('file.txt', "Hello world", { baseDir: BaseDirectory.App });
  * ```
  *
  * @since 2.0.0
  */
async function writeTextFile(
  path: string | URL,
  data: string,
  options?: WriteFileOptions,
): Promise<void> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  await invoke("plugin:fs|write_text_file", {
    path: path instanceof URL ? path.toString() : path,
    data,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface ExistsOptions {
  /** Base directory for `path`. */
  baseDir?: BaseDirectory;
}

/**
 * Check if a path exists.
 * @example
 * ```typescript
 * import { exists, BaseDirectory } from '@tauri-apps/plugin-fs';
 * // Check if the `$APPDATA/avatar.png` file exists
 * await exists('avatar.png', { baseDir: BaseDirectory.AppData });
 * ```
 *
 * @since 2.0.0
 */
async function exists(
  path: string | URL,
  options?: ExistsOptions,
): Promise<boolean> {
  if (path instanceof URL && path.protocol !== "file:") {
    throw new TypeError("Must be a file URL.");
  }

  return await invoke("plugin:fs|exists", {
    path: path instanceof URL ? path.toString() : path,
    options,
  });
}

/**
 * @since 2.0.0
 */
interface WatchOptions {
  /** Watch a directory recursively */
  recursive?: boolean;
  /** Base directory for `path` */
  baseDir?: BaseDirectory;
}

/**
 * @since 2.0.0
 */
interface DebouncedWatchOptions extends WatchOptions {
  /** Debounce delay */
  delayMs?: number;
}

/**
 * @since 2.0.0
 */
interface WatchEvent {
  type: WatchEventKind;
  paths: string[];
  attrs: unknown;
}

/**
 * @since 2.0.0
 */
type WatchEventKind =
  | "any"
  | { access: WatchEventKindAccess }
  | { create: WatchEventKindCreate }
  | { modify: WatchEventKindModify }
  | { remove: WatchEventKindRemove }
  | "other";

/**
 * @since 2.0.0
 */
type WatchEventKindAccess =
  | { kind: "any" }
  | { kind: "close"; mode: "any" | "execute" | "read" | "write" | "other" }
  | { kind: "open"; mode: "any" | "execute" | "read" | "write" | "other" }
  | { kind: "other" };

/**
 * @since 2.0.0
 */
type WatchEventKindCreate =
  | { kind: "any" }
  | { kind: "file" }
  | { kind: "folder" }
  | { kind: "other" };

/**
 * @since 2.0.0
 */
type WatchEventKindModify =
  | { kind: "any" }
  | { kind: "data"; mode: "any" | "size" | "content" | "other" }
  | {
      kind: "metadata";
      mode:
        | "any"
        | "access-time"
        | "write-time"
        | "permissions"
        | "ownership"
        | "extended"
        | "other";
    }
  | { kind: "rename"; mode: "any" | "to" | "from" | "both" | "other" }
  | { kind: "other" };

/**
 * @since 2.0.0
 */
type WatchEventKindRemove =
  | { kind: "any" }
  | { kind: "file" }
  | { kind: "folder" }
  | { kind: "other" };

/**
 * @since 2.0.0
 */
type UnwatchFn = () => void;

async function unwatch(rid: number): Promise<void> {
  await invoke("plugin:fs|unwatch", { rid });
}

/**
 * Watch changes (after a delay) on files or directories.
 *
 * @since 2.0.0
 */
async function watch(
  paths: string | string[] | URL | URL[],
  cb: (event: WatchEvent) => void,
  options?: DebouncedWatchOptions,
): Promise<UnwatchFn> {
  const opts = {
    recursive: false,
    delayMs: 2000,
    ...options,
  };

  const watchPaths = Array.isArray(paths) ? paths : [paths];

  for (const path of watchPaths) {
    if (path instanceof URL && path.protocol !== "file:") {
      throw new TypeError("Must be a file URL.");
    }
  }

  const onEvent = new Channel<WatchEvent>();
  onEvent.onmessage = cb;

  const rid: number = await invoke("plugin:fs|watch", {
    paths: watchPaths.map((p) => (p instanceof URL ? p.toString() : p)),
    options: opts,
    onEvent,
  });

  return () => {
    void unwatch(rid);
  };
}

/**
 * Watch changes on files or directories.
 *
 * @since 2.0.0
 */
async function watchImmediate(
  paths: string | string[] | URL | URL[],
  cb: (event: WatchEvent) => void,
  options?: WatchOptions,
): Promise<UnwatchFn> {
  const opts = {
    recursive: false,
    ...options,
    delayMs: null,
  };

  const watchPaths = Array.isArray(paths) ? paths : [paths];

  for (const path of watchPaths) {
    if (path instanceof URL && path.protocol !== "file:") {
      throw new TypeError("Must be a file URL.");
    }
  }

  const onEvent = new Channel<WatchEvent>();
  onEvent.onmessage = cb;

  const rid: number = await invoke("plugin:fs|watch", {
    paths: watchPaths.map((p) => (p instanceof URL ? p.toString() : p)),
    options: opts,
    onEvent,
  });

  return () => {
    void unwatch(rid);
  };
}

export type {
  CreateOptions,
  OpenOptions,
  CopyFileOptions,
  MkdirOptions,
  DirEntry,
  ReadDirOptions,
  ReadFileOptions,
  RemoveOptions,
  RenameOptions,
  StatOptions,
  TruncateOptions,
  WriteFileOptions,
  ExistsOptions,
  FileInfo,
  WatchOptions,
  DebouncedWatchOptions,
  WatchEvent,
  WatchEventKind,
  WatchEventKindAccess,
  WatchEventKindCreate,
  WatchEventKindModify,
  WatchEventKindRemove,
  UnwatchFn,
};

export {
  BaseDirectory,
  FileHandle,
  create,
  open,
  copyFile,
  mkdir,
  readDir,
  readFile,
  readTextFile,
  readTextFileLines,
  remove,
  rename,
  SeekMode,
  stat,
  lstat,
  truncate,
  writeFile,
  writeTextFile,
  exists,
  watch,
  watchImmediate,
};
