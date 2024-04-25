// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{AppHandleExt, StateFlags, WindowExt};
use tauri::{command, AppHandle, Manager, Runtime};

#[command]
pub async fn save_window_state<R: Runtime>(
    app: AppHandle<R>,
    flags: u32,
) -> std::result::Result<(), String> {
    let flags = StateFlags::from_bits(flags)
        .ok_or_else(|| format!("Invalid state flags bits: {}", flags))?;
    app.save_window_state(flags).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn restore_state<R: Runtime>(
    app: AppHandle<R>,
    label: String,
    flags: u32,
) -> std::result::Result<(), String> {
    let flags = StateFlags::from_bits(flags)
        .ok_or_else(|| format!("Invalid state flags bits: {}", flags))?;
    app.get_webview_window(&label)
        .ok_or_else(|| format!("Couldn't find window with label: {}", label))?
        .as_ref()
        .window()
        .restore_state(flags)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn filename<R: Runtime>(app: AppHandle<R>) -> String {
    app.filename()
}
