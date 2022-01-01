// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to display dialog.

#[cfg(any(dialog_open, dialog_save))]
use std::path::{Path, PathBuf};

use crate::{Runtime, Window};

#[cfg(not(target_os = "linux"))]
macro_rules! run_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let response = $e;
      $h(response);
    });
  }};
}

#[cfg(target_os = "linux")]
macro_rules! run_dialog {
  ($e:expr, $h: ident) => {{
    std::thread::spawn(move || {
      let context = glib::MainContext::default();
      context.invoke_with_priority(glib::PRIORITY_HIGH, move || {
        let response = $e;
        $h(response);
      });
    });
  }};
}

/// The file dialog builder.
///
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[cfg(any(dialog_open, dialog_save))]
#[derive(Debug, Default)]
pub struct FileDialogBuilder(rfd::FileDialog);

#[cfg(any(dialog_open, dialog_save))]
impl FileDialogBuilder {
  /// Gets the default file dialog builder.
  pub fn new() -> Self {
    Default::default()
  }

  /// Add file extension filter. Takes in the name of the filter, and list of extensions
  pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[&str]) -> Self {
    self.0 = self.0.add_filter(name.as_ref(), extensions);
    self
  }

  /// Set starting directory of the dialog.
  pub fn set_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
    self.0 = self.0.set_directory(directory);
    self
  }

  /// Set starting file name of the dialog.
  pub fn set_file_name(mut self, file_name: &str) -> Self {
    self.0 = self.0.set_file_name(file_name);
    self
  }

  /// Sets the parent window of the dialog.
  pub fn set_parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
    self.0 = self.0.set_parent(parent);
    self
  }

  /// Pick one file.
  pub fn pick_file<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_file(), f)
  }

  /// Pick multiple files.
  pub fn pick_files<F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_files(), f)
  }

  /// Pick one folder.
  pub fn pick_folder<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.pick_folder(), f)
  }

  /// Opens save file dialog.
  pub fn save_file<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
    run_dialog!(self.0.save_file(), f)
  }
}

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button.
#[allow(unused_variables)]
pub fn ask<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  f: F,
) {
  run_message_dialog(parent_window, title, message, rfd::MessageButtons::YesNo, f)
}

/// Displays a dialog with a message and an optional title with an "ok" and a "cancel" button.
#[allow(unused_variables)]
pub fn confirm<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  f: F,
) {
  run_message_dialog(
    parent_window,
    title,
    message,
    rfd::MessageButtons::OkCancel,
    f,
  )
}

/// Displays a message dialog.
#[allow(unused_variables)]
pub fn message<R: Runtime>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
) {
  run_message_dialog(
    parent_window,
    title,
    message,
    rfd::MessageButtons::Ok,
    |_| {},
  )
}

#[allow(unused_variables)]
fn run_message_dialog<R: Runtime, F: FnOnce(bool) + Send + 'static>(
  parent_window: Option<&Window<R>>,
  title: impl AsRef<str>,
  message: impl AsRef<str>,
  buttons: rfd::MessageButtons,
  f: F,
) {
  let title = title.as_ref().to_string();
  let message = message.as_ref().to_string();
  #[allow(unused_mut)]
  let mut builder = rfd::MessageDialog::new()
    .set_title(&title)
    .set_description(&message)
    .set_buttons(buttons)
    .set_level(rfd::MessageLevel::Info);

  #[cfg(any(windows, target_os = "macos"))]
  {
    if let Some(window) = parent_window {
      builder = builder.set_parent(window);
    }
  }

  run_dialog!(builder.show(), f)
}
