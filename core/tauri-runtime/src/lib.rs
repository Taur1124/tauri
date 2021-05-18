// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Internal runtime between Tauri and the underlying webview runtime.

#![cfg_attr(doc_cfg, feature(doc_cfg))]

use std::{fmt::Debug, hash::Hash, path::PathBuf};

use serde::Serialize;
use tauri_utils::assets::Assets;
use uuid::Uuid;

/// Create window and system tray menus.
#[cfg(any(feature = "menu", feature = "system-tray"))]
#[cfg_attr(doc_cfg, doc(cfg(any(feature = "menu", feature = "system-tray"))))]
pub mod menu;
/// Types useful for interacting with a user's monitors.
pub mod monitor;
pub mod tag;
pub mod webview;
pub mod window;

use monitor::Monitor;
use tag::Tag;
use webview::WindowBuilder;
use window::{
  dpi::{PhysicalPosition, PhysicalSize, Position, Size},
  DetachedWindow, PendingWindow, WindowEvent,
};

/// A type that can be derived into a menu id.
pub trait MenuId: Serialize + Hash + Eq + Debug + Clone + Send + Sync + 'static {}

impl<T> MenuId for T where T: Serialize + Hash + Eq + Debug + Clone + Send + Sync + 'static {}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Failed to create webview.
  #[error("failed to create webview: {0}")]
  CreateWebview(Box<dyn std::error::Error + Send>),
  /// Failed to create window.
  #[error("failed to create window")]
  CreateWindow,
  /// Failed to send message to webview.
  #[error("failed to send message to the webview")]
  FailedToSendMessage,
  /// Failed to serialize/deserialize.
  #[error("JSON error: {0}")]
  Json(#[from] serde_json::Error),
  /// Encountered an error creating the app system tray.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  #[error("error encountered during tray setup: {0}")]
  SystemTray(Box<dyn std::error::Error + Send>),
  /// Failed to load window icon.
  #[error("invalid icon: {0}")]
  InvalidIcon(Box<dyn std::error::Error + Send>),
}

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

#[doc(hidden)]
pub mod private {
  pub trait ParamsBase {}
}

/// Types associated with the running Tauri application.
pub trait Params: private::ParamsBase + 'static {
  /// The event type used to create and listen to events.
  type Event: Tag;

  /// The type used to determine the name of windows.
  type Label: Tag;

  /// The type used to determine window menu ids.
  type MenuId: MenuId;

  /// The type used to determine system tray menu ids.
  type SystemTrayMenuId: MenuId;

  /// Assets that Tauri should serve from itself.
  type Assets: Assets;

  /// The underlying webview runtime used by the Tauri application.
  type Runtime: Runtime;
}

/// A icon definition.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
  /// Icon from file path.
  File(PathBuf),
  /// Icon from raw bytes.
  Raw(Vec<u8>),
}

/// A system tray event.
pub struct SystemTrayEvent {
  pub menu_item_id: u32,
}

/// A [`Send`] handle to the runtime.
pub trait RuntimeHandle: Send + Sized + Clone + 'static {
  type Runtime: Runtime<Handle = Self>;
  /// Create a new webview window.
  fn create_window<P: Params<Runtime = Self::Runtime>>(
    &self,
    pending: PendingWindow<P>,
  ) -> crate::Result<DetachedWindow<P>>;
}

/// The webview runtime interface.
pub trait Runtime: Sized + 'static {
  /// The message dispatcher.
  type Dispatcher: Dispatch<Runtime = Self>;
  /// The runtime handle type.
  type Handle: RuntimeHandle<Runtime = Self>;

  /// Creates a new webview runtime.
  fn new() -> crate::Result<Self>;

  /// Gets a runtime handle.
  fn handle(&self) -> Self::Handle;

  /// Create a new webview window.
  fn create_window<P: Params<Runtime = Self>>(
    &self,
    pending: PendingWindow<P>,
  ) -> crate::Result<DetachedWindow<P>>;

  /// Adds the icon to the system tray with the specified menu items.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn system_tray<I: MenuId>(
    &self,
    icon: Icon,
    menu: Vec<menu::SystemTrayMenuItem<I>>,
  ) -> crate::Result<()>;

  /// Registers a system tray event handler.
  #[cfg(feature = "system-tray")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "system-tray")))]
  fn on_system_tray_event<F: Fn(&SystemTrayEvent) + Send + 'static>(&mut self, f: F) -> Uuid;

  /// Run the webview runtime.
  fn run(self);
}

/// Webview dispatcher. A thread-safe handle to the webview API.
pub trait Dispatch: Clone + Send + Sized + 'static {
  /// The runtime this [`Dispatch`] runs under.
  type Runtime: Runtime;

  /// The winoow builder type.
  type WindowBuilder: WindowBuilder + Clone;

  /// Run a task on the main thread.
  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> crate::Result<()>;

  /// Registers a window event handler.
  fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) -> Uuid;

  /// Registers a window event handler.
  #[cfg(feature = "menu")]
  #[cfg_attr(doc_cfg, doc(cfg(feature = "menu")))]
  fn on_menu_event<F: Fn(&window::MenuEvent) + Send + 'static>(&self, f: F) -> Uuid;

  // GETTERS

  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  fn scale_factor(&self) -> crate::Result<f64>;

  /// Returns the position of the top-left hand corner of the window's client area relative to the top-left hand corner of the desktop.
  fn inner_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the position of the top-left hand corner of the window relative to the top-left hand corner of the desktop.
  fn outer_position(&self) -> crate::Result<PhysicalPosition<i32>>;

  /// Returns the physical size of the window's client area.
  ///
  /// The client area is the content of the window, excluding the title bar and borders.
  fn inner_size(&self) -> crate::Result<PhysicalSize<u32>>;

  /// Returns the physical size of the entire window.
  ///
  /// These dimensions include the title bar and borders. If you don't want that (and you usually don't), use inner_size instead.
  fn outer_size(&self) -> crate::Result<PhysicalSize<u32>>;

  /// Gets the window's current fullscreen state.
  fn is_fullscreen(&self) -> crate::Result<bool>;

  /// Gets the window's current maximized state.
  fn is_maximized(&self) -> crate::Result<bool>;

  /// Returns the monitor on which the window currently resides.
  ///
  /// Returns None if current monitor can't be detected.
  fn current_monitor(&self) -> crate::Result<Option<Monitor>>;

  /// Returns the primary monitor of the system.
  ///
  /// Returns None if it can't identify any monitor as a primary one.
  fn primary_monitor(&self) -> crate::Result<Option<Monitor>>;

  /// Returns the list of all the monitors available on the system.
  fn available_monitors(&self) -> crate::Result<Vec<Monitor>>;

  // SETTERS

  /// Opens the dialog to prints the contents of the webview.
  fn print(&self) -> crate::Result<()>;

  /// Create a new webview window.
  fn create_window<P: Params<Runtime = Self::Runtime>>(
    &mut self,
    pending: PendingWindow<P>,
  ) -> crate::Result<DetachedWindow<P>>;

  /// Updates the window resizable flag.
  fn set_resizable(&self, resizable: bool) -> crate::Result<()>;

  /// Updates the window title.
  fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()>;

  /// Maximizes the window.
  fn maximize(&self) -> crate::Result<()>;

  /// Unmaximizes the window.
  fn unmaximize(&self) -> crate::Result<()>;

  /// Minimizes the window.
  fn minimize(&self) -> crate::Result<()>;

  /// Unminimizes the window.
  fn unminimize(&self) -> crate::Result<()>;

  /// Shows the window.
  fn show(&self) -> crate::Result<()>;

  /// Hides the window.
  fn hide(&self) -> crate::Result<()>;

  /// Closes the window.
  fn close(&self) -> crate::Result<()>;

  /// Updates the hasDecorations flag.
  fn set_decorations(&self, decorations: bool) -> crate::Result<()>;

  /// Updates the window alwaysOnTop flag.
  fn set_always_on_top(&self, always_on_top: bool) -> crate::Result<()>;

  /// Resizes the window.
  fn set_size(&self, size: Size) -> crate::Result<()>;

  /// Updates the window min size.
  fn set_min_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window max size.
  fn set_max_size(&self, size: Option<Size>) -> crate::Result<()>;

  /// Updates the window position.
  fn set_position(&self, position: Position) -> crate::Result<()>;

  /// Updates the window fullscreen state.
  fn set_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Updates the window icon.
  fn set_icon(&self, icon: Icon) -> crate::Result<()>;

  /// Starts dragging the window.
  fn start_dragging(&self) -> crate::Result<()>;

  /// Executes javascript on the window this [`Dispatch`] represents.
  fn eval_script<S: Into<String>>(&self, script: S) -> crate::Result<()>;
}
