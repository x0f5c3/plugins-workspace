// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Use native message and file open/save dialogs.
//!
//! This module exposes non-blocking APIs on its root, relying on callback closures
//! to give results back. This is particularly useful when running dialogs from the main thread.
//! When using on asynchronous contexts such as async commands, the [`blocking`] APIs are recommended.

use std::path::PathBuf;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use rfd::{AsyncFileDialog, AsyncMessageDialog};
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{models::*, FileDialogBuilder, MessageDialogBuilder};

const OK: &str = "Ok";

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Dialog<R>> {
    Ok(Dialog(app.clone()))
}

/// Access to the dialog APIs.
#[derive(Debug)]
pub struct Dialog<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Clone for Dialog<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> Dialog<R> {
    pub(crate) fn app_handle(&self) -> &AppHandle<R> {
        &self.0
    }
}

impl From<MessageDialogKind> for rfd::MessageLevel {
    fn from(kind: MessageDialogKind) -> Self {
        match kind {
            MessageDialogKind::Info => Self::Info,
            MessageDialogKind::Warning => Self::Warning,
            MessageDialogKind::Error => Self::Error,
        }
    }
}

struct WindowHandle(RawWindowHandle);

impl HasWindowHandle for WindowHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.0) })
    }
}

impl<R: Runtime> From<FileDialogBuilder<R>> for AsyncFileDialog {
    fn from(d: FileDialogBuilder<R>) -> Self {
        let mut builder = AsyncFileDialog::new();

        if let Some(title) = d.title {
            builder = builder.set_title(title);
        }
        if let Some(starting_directory) = d.starting_directory {
            builder = builder.set_directory(starting_directory);
        }
        if let Some(file_name) = d.file_name {
            builder = builder.set_file_name(file_name);
        }
        for filter in d.filters {
            let v: Vec<&str> = filter.extensions.iter().map(|x| &**x).collect();
            builder = builder.add_filter(&filter.name, &v);
        }
        #[cfg(desktop)]
        if let Some(parent) = d.parent {
            builder = builder.set_parent(&WindowHandle(parent));
        }

        builder = builder.set_can_create_directories(d.can_create_directories.unwrap_or(true));

        builder
    }
}

impl<R: Runtime> From<MessageDialogBuilder<R>> for AsyncMessageDialog {
    fn from(d: MessageDialogBuilder<R>) -> Self {
        let mut dialog = AsyncMessageDialog::new()
            .set_title(&d.title)
            .set_description(&d.message)
            .set_level(d.kind.into());

        let buttons = match (d.ok_button_label, d.cancel_button_label) {
            (Some(ok), Some(cancel)) => Some(rfd::MessageButtons::OkCancelCustom(ok, cancel)),
            (Some(ok), None) => Some(rfd::MessageButtons::OkCustom(ok)),
            (None, Some(cancel)) => Some(rfd::MessageButtons::OkCancelCustom(OK.into(), cancel)),
            (None, None) => None,
        };
        if let Some(buttons) = buttons {
            dialog = dialog.set_buttons(buttons);
        }

        if let Some(parent) = d.parent {
            dialog = dialog.set_parent(&WindowHandle(parent));
        }

        dialog
    }
}

pub fn pick_file<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncFileDialog::from(dialog).pick_file();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}

pub fn pick_files<R: Runtime, F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let f = |paths: Option<Vec<rfd::FileHandle>>| {
        f(paths.map(|list| list.into_iter().map(|p| p.path().to_path_buf()).collect()))
    };
    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncFileDialog::from(dialog).pick_files();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}

pub fn pick_folder<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncFileDialog::from(dialog).pick_folder();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}

pub fn pick_folders<R: Runtime, F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let f = |paths: Option<Vec<rfd::FileHandle>>| {
        f(paths.map(|list| list.into_iter().map(|p| p.path().to_path_buf()).collect()))
    };
    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncFileDialog::from(dialog).pick_folders();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}

pub fn save_file<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let f = |path: Option<rfd::FileHandle>| f(path.map(|p| p.path().to_path_buf()));
    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncFileDialog::from(dialog).save_file();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}

/// Shows a message dialog
pub fn show_message_dialog<R: Runtime, F: FnOnce(bool) + Send + 'static>(
    dialog: MessageDialogBuilder<R>,
    f: F,
) {
    use rfd::MessageDialogResult;

    let ok_label = dialog.ok_button_label.clone();
    let f = move |res| {
        f(match res {
            MessageDialogResult::Ok | MessageDialogResult::Yes => true,
            MessageDialogResult::Custom(s) => ok_label.map_or(s == OK, |ok_label| ok_label == s),
            _ => false,
        });
    };

    let handle = dialog.dialog.app_handle().to_owned();
    let _ = handle.run_on_main_thread(move || {
        let dialog = AsyncMessageDialog::from(dialog).show();
        std::thread::spawn(move || f(tauri::async_runtime::block_on(dialog)));
    });
}
