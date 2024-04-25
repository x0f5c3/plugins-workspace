// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// Copyright 2018-2023 the Deno authors.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::{
    ipc::{CommandScope, GlobalScope},
    path::{BaseDirectory, SafePathBuf},
    utils::config::FsScope,
    Manager, Resource, ResourceId, Runtime, Webview,
};

use std::{
    fs::File,
    io::{BufReader, Lines, Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{scope::Entry, Error, FsExt};

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Plugin(#[from] Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[cfg(feature = "watch")]
    #[error(transparent)]
    Watcher(#[from] notify::Error),
}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        Self::Anyhow(anyhow::anyhow!(value))
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        Self::Anyhow(anyhow::anyhow!(value.to_string()))
    }
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Self::Anyhow(err) = self {
            serializer.serialize_str(format!("{err:#}").as_ref())
        } else {
            serializer.serialize_str(self.to_string().as_ref())
        }
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseOptions {
    base_dir: Option<BaseDirectory>,
}

#[tauri::command]
pub fn create<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<ResourceId> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.and_then(|o| o.base_dir),
    )?;
    let file = File::create(&resolved_path).map_err(|e| {
        format!(
            "failed to create file at path: {} with error: {e}",
            resolved_path.display()
        )
    })?;
    let rid = webview.resources_table().add(StdFileResource::new(file));
    Ok(rid)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptions {
    #[serde(flatten)]
    base: BaseOptions,
    #[serde(default = "default_true")]
    read: bool,
    #[serde(default)]
    write: bool,
    #[serde(default)]
    append: bool,
    #[serde(default)]
    truncate: bool,
    #[serde(default)]
    create: bool,
    #[serde(default)]
    create_new: bool,
    #[allow(unused)]
    mode: Option<u32>,
}

fn default_true() -> bool {
    true
}

#[tauri::command]
pub fn open<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<OpenOptions>,
) -> CommandResult<ResourceId> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base.base_dir),
    )?;

    let mut opts = std::fs::OpenOptions::new();
    // default to read-only
    opts.read(true);

    if let Some(options) = options {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Some(mode) = options.mode {
                opts.mode(mode);
            }
        }

        opts.read(options.read)
            .create(options.create)
            .write(options.write)
            .truncate(options.truncate)
            .append(options.append)
            .create_new(options.create_new);
    }

    let file = opts.open(&resolved_path).map_err(|e| {
        format!(
            "failed to open file at path: {} with error: {e}",
            resolved_path.display()
        )
    })?;

    let rid = webview.resources_table().add(StdFileResource::new(file));

    Ok(rid)
}

#[tauri::command]
pub fn close<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> CommandResult<()> {
    webview.resources_table().close(rid).map_err(Into::into)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyFileOptions {
    from_path_base_dir: Option<BaseDirectory>,
    to_path_base_dir: Option<BaseDirectory>,
}

#[tauri::command]
pub fn copy_file<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    from_path: SafePathBuf,
    to_path: SafePathBuf,
    options: Option<CopyFileOptions>,
) -> CommandResult<()> {
    let resolved_from_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        from_path,
        options.as_ref().and_then(|o| o.from_path_base_dir),
    )?;
    let resolved_to_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        to_path,
        options.as_ref().and_then(|o| o.to_path_base_dir),
    )?;
    std::fs::copy(&resolved_from_path, &resolved_to_path).map_err(|e| {
        format!(
            "failed to copy file from path: {}, to path: {} with error: {e}",
            resolved_from_path.display(),
            resolved_to_path.display()
        )
    })?;
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub struct MkdirOptions {
    #[serde(flatten)]
    base: BaseOptions,
    #[allow(unused)]
    mode: Option<u32>,
    recursive: Option<bool>,
}

#[tauri::command]
pub fn mkdir<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<MkdirOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base.base_dir),
    )?;

    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(options.as_ref().and_then(|o| o.recursive).unwrap_or(false));

    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let mode = options.as_ref().and_then(|o| o.mode).unwrap_or(0o777) & 0o777;
        builder.mode(mode);
    }

    builder
        .create(&resolved_path)
        .map_err(|e| {
            format!(
                "failed to create directory at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DirEntry {
    pub name: Option<String>,
    pub is_directory: bool,
    pub is_file: bool,
    pub is_symlink: bool,
}

fn read_dir_inner<P: AsRef<Path>>(path: P) -> crate::Result<Vec<DirEntry>> {
    let mut files_and_dirs: Vec<DirEntry> = vec![];
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        let file_type = path.metadata()?.file_type();
        files_and_dirs.push(DirEntry {
            is_directory: file_type.is_dir(),
            is_file: file_type.is_file(),
            is_symlink: std::fs::symlink_metadata(&path)
                .map(|md| md.file_type().is_symlink())
                .unwrap_or(false),
            name: path
                .file_name()
                .map(|name| name.to_string_lossy())
                .map(|name| name.to_string()),
        });
    }
    Result::Ok(files_and_dirs)
}

#[tauri::command]
pub fn read_dir<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<Vec<DirEntry>> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;

    read_dir_inner(&resolved_path)
        .map_err(|e| {
            format!(
                "failed to read directory at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[tauri::command]
pub fn read<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    len: u32,
) -> CommandResult<(Vec<u8>, usize)> {
    let mut data = vec![0; len as usize];
    let file = webview.resources_table().get::<StdFileResource>(rid)?;
    let nread = StdFileResource::with_lock(&file, |mut file| file.read(&mut data))
        .map_err(|e| format!("faied to read bytes from file with error: {e}"))?;
    Ok((data, nread))
}

#[tauri::command]
pub fn read_file<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<Vec<u8>> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    std::fs::read(&resolved_path)
        .map_err(|e| {
            format!(
                "failed to read file at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[tauri::command]
pub fn read_text_file<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<String> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    std::fs::read_to_string(&resolved_path)
        .map_err(|e| {
            format!(
                "failed to read file as text at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[tauri::command]
pub fn read_text_file_lines<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<ResourceId> {
    use std::io::BufRead;

    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;

    let file = File::open(&resolved_path).map_err(|e| {
        format!(
            "failed to open file at path: {} with error: {e}",
            resolved_path.display()
        )
    })?;

    let lines = BufReader::new(file).lines();
    let rid = webview.resources_table().add(StdLinesResource::new(lines));

    Ok(rid)
}

#[tauri::command]
pub fn read_text_file_lines_next<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> CommandResult<(Option<String>, bool)> {
    let mut resource_table = webview.resources_table();
    let lines = resource_table.get::<StdLinesResource>(rid)?;

    let ret = StdLinesResource::with_lock(&lines, |lines| {
        lines.next().map(|a| (a.ok(), false)).unwrap_or_else(|| {
            let _ = resource_table.close(rid);
            (None, true)
        })
    });

    Ok(ret)
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoveOptions {
    #[serde(flatten)]
    base: BaseOptions,
    recursive: Option<bool>,
}

#[tauri::command]
pub fn remove<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<RemoveOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base.base_dir),
    )?;

    let metadata = std::fs::symlink_metadata(&resolved_path).map_err(|e| {
        format!(
            "failed to get metadata of path: {} with error: {e}",
            resolved_path.display()
        )
    })?;

    let file_type = metadata.file_type();

    // taken from deno source code: https://github.com/denoland/deno/blob/429759fe8b4207240709c240a8344d12a1e39566/runtime/ops/fs.rs#L728
    let res = if file_type.is_file() {
        std::fs::remove_file(&resolved_path)
    } else if options.as_ref().and_then(|o| o.recursive).unwrap_or(false) {
        std::fs::remove_dir_all(&resolved_path)
    } else if file_type.is_symlink() {
        #[cfg(unix)]
        {
            std::fs::remove_file(&resolved_path)
        }
        #[cfg(not(unix))]
        {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
            if metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0 {
                std::fs::remove_dir(&resolved_path)
            } else {
                std::fs::remove_file(&resolved_path)
            }
        }
    } else if file_type.is_dir() {
        std::fs::remove_dir(&resolved_path)
    } else {
        // pipes, sockets, etc...
        std::fs::remove_file(&resolved_path)
    };

    res.map_err(|e| {
        format!(
            "failed to remove path: {} with error: {e}",
            resolved_path.display()
        )
    })
    .map_err(Into::into)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
    new_path_base_dir: Option<BaseDirectory>,
    old_path_base_dir: Option<BaseDirectory>,
}

#[tauri::command]
pub fn rename<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<RenameOptions>,
) -> CommandResult<()> {
    let resolved_old_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        old_path,
        options.as_ref().and_then(|o| o.old_path_base_dir),
    )?;
    let resolved_new_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        new_path,
        options.as_ref().and_then(|o| o.new_path_base_dir),
    )?;
    std::fs::rename(&resolved_old_path, &resolved_new_path)
        .map_err(|e| {
            format!(
                "failed to rename old path: {} to new path: {} with error: {e}",
                resolved_old_path.display(),
                resolved_new_path.display()
            )
        })
        .map_err(Into::into)
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug)]
#[repr(u16)]
pub enum SeekMode {
    Start = 0,
    Current = 1,
    End = 2,
}

#[tauri::command]
pub fn seek<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    offset: i64,
    whence: SeekMode,
) -> CommandResult<u64> {
    use std::io::{Seek, SeekFrom};
    let file = webview.resources_table().get::<StdFileResource>(rid)?;
    StdFileResource::with_lock(&file, |mut file| {
        file.seek(match whence {
            SeekMode::Start => SeekFrom::Start(offset as u64),
            SeekMode::Current => SeekFrom::Current(offset),
            SeekMode::End => SeekFrom::End(offset),
        })
    })
    .map_err(|e| format!("failed to seek file with error: {e}"))
    .map_err(Into::into)
}

#[tauri::command]
pub fn stat<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<FileInfo> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    let metadata = std::fs::metadata(&resolved_path).map_err(|e| {
        format!(
            "failed to get metadata of path: {} with error: {e}",
            resolved_path.display()
        )
    })?;
    Ok(get_stat(metadata))
}

#[tauri::command]
pub fn lstat<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<FileInfo> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    let metadata = std::fs::symlink_metadata(&resolved_path).map_err(|e| {
        format!(
            "failed to get metadata of path: {} with error: {e}",
            resolved_path.display()
        )
    })?;
    Ok(get_stat(metadata))
}

#[tauri::command]
pub fn fstat<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> CommandResult<FileInfo> {
    let file = webview.resources_table().get::<StdFileResource>(rid)?;
    let metadata = StdFileResource::with_lock(&file, |file| file.metadata())
        .map_err(|e| format!("failed to get metadata of file with error: {e}"))?;
    Ok(get_stat(metadata))
}

#[tauri::command]
pub fn truncate<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    len: Option<u64>,
    options: Option<BaseOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    let f = std::fs::OpenOptions::new()
        .write(true)
        .open(&resolved_path)
        .map_err(|e| {
            format!(
                "failed to open file at path: {} with error: {e}",
                resolved_path.display()
            )
        })?;
    f.set_len(len.unwrap_or(0))
        .map_err(|e| {
            format!(
                "failed to truncate file at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[tauri::command]
pub fn ftruncate<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    len: Option<u64>,
) -> CommandResult<()> {
    let file = webview.resources_table().get::<StdFileResource>(rid)?;
    StdFileResource::with_lock(&file, |file| file.set_len(len.unwrap_or(0)))
        .map_err(|e| format!("failed to truncate file with error: {e}"))
        .map_err(Into::into)
}

#[tauri::command]
pub fn write<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
    data: Vec<u8>,
) -> CommandResult<usize> {
    let file = webview.resources_table().get::<StdFileResource>(rid)?;
    StdFileResource::with_lock(&file, |mut file| file.write(&data))
        .map_err(|e| format!("failed to write bytes to file with error: {e}"))
        .map_err(Into::into)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileOptions {
    #[serde(flatten)]
    base: BaseOptions,
    #[serde(default)]
    append: bool,
    #[serde(default = "default_create_value")]
    create: bool,
    #[serde(default)]
    create_new: bool,
    #[allow(unused)]
    mode: Option<u32>,
}

fn default_create_value() -> bool {
    true
}

fn write_file_inner<R: Runtime>(
    webview: Webview<R>,
    global_scope: &GlobalScope<Entry>,
    command_scope: &CommandScope<Entry>,
    path: SafePathBuf,
    data: &[u8],
    options: Option<WriteFileOptions>,
) -> CommandResult<()> {
    let resolved_path = resolve_path(
        &webview,
        global_scope,
        command_scope,
        path,
        options.as_ref().and_then(|o| o.base.base_dir),
    )?;

    let mut opts = std::fs::OpenOptions::new();
    // defaults
    opts.read(false).write(true).truncate(true).create(true);

    if let Some(options) = options {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Some(mode) = options.mode {
                opts.mode(mode);
            }
        }

        opts.create(options.create)
            .append(options.append)
            .truncate(!options.append)
            .create_new(options.create_new);
    }

    let mut file = opts.open(&resolved_path).map_err(|e| {
        format!(
            "failed to open file at path: {} with error: {e}",
            resolved_path.display()
        )
    })?;

    file.write_all(data)
        .map_err(|e| {
            format!(
                "failed to write bytes to file at path: {} with error: {e}",
                resolved_path.display()
            )
        })
        .map_err(Into::into)
}

#[tauri::command]
pub fn write_file<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    data: Vec<u8>,
    options: Option<WriteFileOptions>,
) -> CommandResult<()> {
    write_file_inner(webview, &global_scope, &command_scope, path, &data, options)
}

#[tauri::command]
pub fn write_text_file<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    data: String,
    options: Option<WriteFileOptions>,
) -> CommandResult<()> {
    write_file_inner(
        webview,
        &global_scope,
        &command_scope,
        path,
        data.as_bytes(),
        options,
    )
}

#[tauri::command]
pub fn exists<R: Runtime>(
    webview: Webview<R>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
    path: SafePathBuf,
    options: Option<BaseOptions>,
) -> CommandResult<bool> {
    let resolved_path = resolve_path(
        &webview,
        &global_scope,
        &command_scope,
        path,
        options.as_ref().and_then(|o| o.base_dir),
    )?;
    Ok(resolved_path.exists())
}

pub fn resolve_path<R: Runtime>(
    app: &Webview<R>,
    global_scope: &GlobalScope<Entry>,
    command_scope: &CommandScope<Entry>,
    path: SafePathBuf,
    base_dir: Option<BaseDirectory>,
) -> CommandResult<PathBuf> {
    let path = file_url_to_safe_pathbuf(path)?;
    let path = if let Some(base_dir) = base_dir {
        app.path().resolve(&path, base_dir)?
    } else {
        path.as_ref().to_path_buf()
    };

    let scope = tauri::scope::fs::Scope::new(
        app,
        &FsScope::Scope {
            allow: app
                .fs_scope()
                .allowed
                .lock()
                .unwrap()
                .clone()
                .into_iter()
                .chain(global_scope.allows().iter().map(|e| e.path.clone()))
                .chain(command_scope.allows().iter().map(|e| e.path.clone()))
                .collect(),
            deny: app
                .fs_scope()
                .denied
                .lock()
                .unwrap()
                .clone()
                .into_iter()
                .chain(global_scope.denies().iter().map(|e| e.path.clone()))
                .chain(command_scope.denies().iter().map(|e| e.path.clone()))
                .collect(),
            require_literal_leading_dot: None,
        },
    )?;

    if scope.is_allowed(&path) {
        Ok(path)
    } else {
        Err(CommandError::Plugin(Error::PathForbidden(path)))
    }
}

#[inline]
fn file_url_to_safe_pathbuf(path: SafePathBuf) -> CommandResult<SafePathBuf> {
    if path.as_ref().starts_with("file:") {
        let url = url::Url::parse(&path.display().to_string())?
            .to_file_path()
            .map_err(|_| "failed to get path from `file:` url")?;
        SafePathBuf::new(url).map_err(Into::into)
    } else {
        Ok(path)
    }
}

struct StdFileResource(Mutex<File>);

impl StdFileResource {
    fn new(file: File) -> Self {
        Self(Mutex::new(file))
    }

    fn with_lock<R, F: FnMut(&File) -> R>(&self, mut f: F) -> R {
        let file = self.0.lock().unwrap();
        f(&file)
    }
}

impl Resource for StdFileResource {}

struct StdLinesResource(Mutex<Lines<BufReader<File>>>);

impl StdLinesResource {
    fn new(lines: Lines<BufReader<File>>) -> Self {
        Self(Mutex::new(lines))
    }

    fn with_lock<R, F: FnMut(&mut Lines<BufReader<File>>) -> R>(&self, mut f: F) -> R {
        let mut lines = self.0.lock().unwrap();
        f(&mut lines)
    }
}

impl Resource for StdLinesResource {}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L913
#[inline]
fn to_msec(maybe_time: std::result::Result<SystemTime, std::io::Error>) -> Option<u64> {
    match maybe_time {
        Ok(time) => {
            let msec = time
                .duration_since(UNIX_EPOCH)
                .map(|t| t.as_millis() as u64)
                .unwrap_or_else(|err| err.duration().as_millis() as u64);
            Some(msec)
        }
        Err(_) => None,
    }
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L926
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    is_file: bool,
    is_directory: bool,
    is_symlink: bool,
    size: u64,
    // In milliseconds, like JavaScript. Available on both Unix or Windows.
    mtime: Option<u64>,
    atime: Option<u64>,
    birthtime: Option<u64>,
    readonly: bool,
    // Following are only valid under Windows.
    file_attribues: Option<u32>,
    // Following are only valid under Unix.
    dev: Option<u64>,
    ino: Option<u64>,
    mode: Option<u32>,
    nlink: Option<u64>,
    uid: Option<u32>,
    gid: Option<u32>,
    rdev: Option<u64>,
    blksize: Option<u64>,
    blocks: Option<u64>,
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L950
#[inline(always)]
fn get_stat(metadata: std::fs::Metadata) -> FileInfo {
    // Unix stat member (number types only). 0 if not on unix.
    macro_rules! usm {
        ($member:ident) => {{
            #[cfg(unix)]
            {
                Some(metadata.$member())
            }
            #[cfg(not(unix))]
            {
                None
            }
        }};
    }

    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    #[cfg(windows)]
    use std::os::windows::fs::MetadataExt;
    FileInfo {
        is_file: metadata.is_file(),
        is_directory: metadata.is_dir(),
        is_symlink: metadata.file_type().is_symlink(),
        size: metadata.len(),
        // In milliseconds, like JavaScript. Available on both Unix or Windows.
        mtime: to_msec(metadata.modified()),
        atime: to_msec(metadata.accessed()),
        birthtime: to_msec(metadata.created()),
        readonly: metadata.permissions().readonly(),
        // Following are only valid under Windows.
        #[cfg(windows)]
        file_attribues: Some(metadata.file_attributes()),
        #[cfg(not(windows))]
        file_attribues: None,
        // Following are only valid under Unix.
        dev: usm!(dev),
        ino: usm!(ino),
        mode: usm!(mode),
        nlink: usm!(nlink),
        uid: usm!(uid),
        gid: usm!(gid),
        rdev: usm!(rdev),
        blksize: usm!(blksize),
        blocks: usm!(blocks),
    }
}
