// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/window-state/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/window-state)
//!
//! Save window positions and sizes and restore them when the app is reopened.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, RunEvent, Runtime, Window,
    WindowEvent,
};

use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, File},
    sync::{Arc, Mutex},
};

mod cmd;

/// Default filename used to store window state.
///
/// If using a custom filename, you should probably use [`AppHandleExt::filename`] instead.
pub const DEFAULT_FILENAME: &str = ".window-state.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct StateFlags: u32 {
        const SIZE        = 1 << 0;
        const POSITION    = 1 << 1;
        const MAXIMIZED   = 1 << 2;
        const VISIBLE     = 1 << 3;
        const DECORATIONS = 1 << 4;
        const FULLSCREEN  = 1 << 5;
    }
}

impl Default for StateFlags {
    fn default() -> Self {
        Self::all()
    }
}

struct PluginState {
    filename: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct WindowState {
    width: f64,
    height: f64,
    x: i32,
    y: i32,
    // prev_x and prev_y are used to store position
    // before maximization happened, because maximization
    // will set x and y to the top-left corner of the monitor
    prev_x: i32,
    prev_y: i32,
    maximized: bool,
    visible: bool,
    decorated: bool,
    fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: Default::default(),
            height: Default::default(),
            x: Default::default(),
            y: Default::default(),
            prev_x: Default::default(),
            prev_y: Default::default(),
            maximized: Default::default(),
            visible: true,
            decorated: true,
            fullscreen: Default::default(),
        }
    }
}

struct WindowStateCache(Arc<Mutex<HashMap<String, WindowState>>>);
pub trait AppHandleExt {
    /// Saves all open windows state to disk
    fn save_window_state(&self, flags: StateFlags) -> Result<()>;
    /// Get the name of the file used to store window state.
    fn filename(&self) -> String;
}

impl<R: Runtime> AppHandleExt for tauri::AppHandle<R> {
    fn save_window_state(&self, flags: StateFlags) -> Result<()> {
        if let Ok(app_dir) = self.path().app_config_dir() {
            let plugin_state = self.state::<PluginState>();
            let state_path = app_dir.join(&plugin_state.filename);
            let cache = self.state::<WindowStateCache>();
            let mut state = cache.0.lock().unwrap();
            for (label, s) in state.iter_mut() {
                if let Some(window) = self.get_webview_window(label) {
                    window.as_ref().window().update_state(s, flags)?;
                }
            }

            create_dir_all(&app_dir)
                .map_err(Error::Io)
                .and_then(|_| File::create(state_path).map_err(Into::into))
                .and_then(|mut f| serde_json::to_writer_pretty(&mut f, &*state).map_err(Into::into))
        } else {
            Ok(())
        }
    }

    fn filename(&self) -> String {
        self.state::<PluginState>().filename.clone()
    }
}

pub trait WindowExt {
    /// Restores this window state from disk
    fn restore_state(&self, flags: StateFlags) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExt for Window<R> {
    fn restore_state(&self, flags: StateFlags) -> tauri::Result<()> {
        let cache = self.state::<WindowStateCache>();
        let mut c = cache.0.lock().unwrap();

        let mut should_show = true;

        if let Some(state) = c.get(self.label()) {
            // avoid restoring the default zeroed state
            if *state == WindowState::default() {
                return Ok(());
            }

            if flags.contains(StateFlags::DECORATIONS) {
                self.set_decorations(state.decorated)?;
            }

            if flags.contains(StateFlags::SIZE) {
                self.set_size(LogicalSize {
                    width: state.width,
                    height: state.height,
                })?;
            }

            if flags.contains(StateFlags::POSITION) {
                let position = (state.x, state.y).into();
                let size = (state.width, state.height).into();
                // restore position to saved value if saved monitor exists
                // otherwise, let the OS decide where to place the window
                for m in self.available_monitors()? {
                    if m.intersects(position, size) {
                        self.set_position(PhysicalPosition {
                            x: if state.maximized {
                                state.prev_x
                            } else {
                                state.x
                            },
                            y: if state.maximized {
                                state.prev_y
                            } else {
                                state.y
                            },
                        })?;
                    }
                }
            }

            if flags.contains(StateFlags::MAXIMIZED) && state.maximized {
                self.maximize()?;
            }

            if flags.contains(StateFlags::FULLSCREEN) {
                self.set_fullscreen(state.fullscreen)?;
            }

            should_show = state.visible;
        } else {
            let mut metadata = WindowState::default();

            if flags.contains(StateFlags::SIZE) {
                let scale_factor = self
                    .current_monitor()?
                    .map(|m| m.scale_factor())
                    .unwrap_or(1.);
                let size = self.inner_size()?.to_logical(scale_factor);
                metadata.width = size.width;
                metadata.height = size.height;
            }

            if flags.contains(StateFlags::POSITION) {
                let pos = self.outer_position()?;
                metadata.x = pos.x;
                metadata.y = pos.y;
            }

            if flags.contains(StateFlags::MAXIMIZED) {
                metadata.maximized = self.is_maximized()?;
            }

            if flags.contains(StateFlags::VISIBLE) {
                metadata.visible = self.is_visible()?;
            }

            if flags.contains(StateFlags::DECORATIONS) {
                metadata.decorated = self.is_decorated()?;
            }

            if flags.contains(StateFlags::FULLSCREEN) {
                metadata.fullscreen = self.is_fullscreen()?;
            }

            c.insert(self.label().into(), metadata);
        }

        if flags.contains(StateFlags::VISIBLE) && should_show {
            self.show()?;
            self.set_focus()?;
        }

        Ok(())
    }
}

trait WindowExtInternal {
    fn update_state(&self, state: &mut WindowState, flags: StateFlags) -> tauri::Result<()>;
}

impl<R: Runtime> WindowExtInternal for Window<R> {
    fn update_state(&self, state: &mut WindowState, flags: StateFlags) -> tauri::Result<()> {
        let is_maximized = match flags.intersects(StateFlags::MAXIMIZED | StateFlags::SIZE) {
            true => self.is_maximized()?,
            false => false,
        };

        if flags.contains(StateFlags::MAXIMIZED) {
            state.maximized = is_maximized;
        }

        if flags.contains(StateFlags::FULLSCREEN) {
            state.fullscreen = self.is_fullscreen()?;
        }

        if flags.contains(StateFlags::DECORATIONS) {
            state.decorated = self.is_decorated()?;
        }

        if flags.contains(StateFlags::VISIBLE) {
            state.visible = self.is_visible()?;
        }

        if flags.contains(StateFlags::SIZE) {
            let scale_factor = self
                .current_monitor()?
                .map(|m| m.scale_factor())
                .unwrap_or(1.);
            let size = self.inner_size()?.to_logical(scale_factor);

            // It doesn't make sense to save a window with 0 height or width
            if size.width > 0. && size.height > 0. && !is_maximized {
                state.width = size.width;
                state.height = size.height;
            }
        }

        if flags.contains(StateFlags::POSITION) && !is_maximized {
            let position = self.outer_position()?;
            state.x = position.x;
            state.y = position.y;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    denylist: HashSet<String>,
    skip_initial_state: HashSet<String>,
    state_flags: StateFlags,
    filename: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the state flags to control what state gets restored and saved.
    pub fn with_state_flags(mut self, flags: StateFlags) -> Self {
        self.state_flags = flags;
        self
    }

    /// Sets a custom filename to use when saving and restoring window states from disk.
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename.replace(filename.into());
        self
    }

    /// Sets a list of windows that shouldn't be tracked and managed by this plugin
    /// for example splash screen windows.
    pub fn with_denylist(mut self, denylist: &[&str]) -> Self {
        self.denylist = denylist.iter().map(|l| l.to_string()).collect();
        self
    }

    /// Adds the given window label to a list of windows to skip initial state restore.
    pub fn skip_initial_state(mut self, label: &str) -> Self {
        self.skip_initial_state.insert(label.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let flags = self.state_flags;
        let filename = self.filename.unwrap_or_else(|| DEFAULT_FILENAME.into());

        PluginBuilder::new("window-state")
            .invoke_handler(tauri::generate_handler![
                cmd::save_window_state,
                cmd::restore_state,
                cmd::filename
            ])
            .setup(|app, _api| {
                let cache: Arc<Mutex<HashMap<String, WindowState>>> =
                    if let Ok(app_dir) = app.path().app_config_dir() {
                        let state_path = app_dir.join(&filename);
                        if state_path.exists() {
                            Arc::new(Mutex::new(
                                std::fs::read(state_path)
                                    .map_err(Error::from)
                                    .and_then(|state| {
                                        serde_json::from_slice(&state).map_err(Into::into)
                                    })
                                    .unwrap_or_default(),
                            ))
                        } else {
                            Default::default()
                        }
                    } else {
                        Default::default()
                    };
                app.manage(WindowStateCache(cache));
                app.manage(PluginState { filename });
                Ok(())
            })
            .on_window_ready(move |window| {
                if self.denylist.contains(window.label()) {
                    return;
                }

                if !self.skip_initial_state.contains(window.label()) {
                    let _ = window.restore_state(self.state_flags);
                }

                let cache = window.state::<WindowStateCache>();
                let cache = cache.0.clone();
                let label = window.label().to_string();
                let window_clone = window.clone();
                let flags = self.state_flags;

                // insert a default state if this window should be tracked and
                // the disk cache doesn't have a state for it
                {
                    cache
                        .lock()
                        .unwrap()
                        .entry(label.clone())
                        .or_insert_with(WindowState::default);
                }

                window.on_window_event(move |e| match e {
                    WindowEvent::CloseRequested { .. } => {
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            let _ = window_clone.update_state(state, flags);
                        }
                    }

                    WindowEvent::Moved(position) if flags.contains(StateFlags::POSITION) => {
                        let mut c = cache.lock().unwrap();
                        if let Some(state) = c.get_mut(&label) {
                            state.prev_x = state.x;
                            state.prev_y = state.y;

                            state.x = position.x;
                            state.y = position.y;
                        }
                    }
                    _ => {}
                });
            })
            .on_event(move |app, event| {
                if let RunEvent::Exit = event {
                    let _ = app.save_window_state(flags);
                }
            })
            .build()
    }
}

trait MonitorExt {
    fn intersects(&self, position: PhysicalPosition<i32>, size: LogicalSize<u32>) -> bool;
}

impl MonitorExt for Monitor {
    fn intersects(&self, position: PhysicalPosition<i32>, size: LogicalSize<u32>) -> bool {
        let size = size.to_physical::<u32>(self.scale_factor());

        let PhysicalPosition { x, y } = *self.position();
        let PhysicalSize { width, height } = *self.size();

        let left = x;
        let right = x + width as i32;
        let top = y;
        let bottom = y + height as i32;

        [
            (position.x, position.y),
            (position.x + size.width as i32, position.y),
            (position.x, position.y + size.height as i32),
            (
                position.x + size.width as i32,
                position.y + size.height as i32,
            ),
        ]
        .into_iter()
        .any(|(x, y)| x >= left && x < right && y >= top && y < bottom)
    }
}
