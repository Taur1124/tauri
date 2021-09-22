// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
#[cfg(any(dialog_open, dialog_save))]
use crate::api::dialog::FileDialogBuilder;
use crate::{
  api::dialog::{ask as ask_dialog, message as message_dialog},
  runtime::Runtime,
  Window,
};
use serde::Deserialize;

use std::{path::PathBuf, sync::mpsc::channel};

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFilter {
  name: String,
  extensions: Vec<String>,
}

/// The options for the open dialog API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
  /// The filters of the dialog.
  #[serde(default)]
  pub filters: Vec<DialogFilter>,
  /// Whether the dialog allows multiple selection or not.
  #[serde(default)]
  pub multiple: bool,
  /// Whether the dialog is a directory selection (`true` value) or file selection (`false` value).
  #[serde(default)]
  pub directory: bool,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
}

/// The options for the save dialog API.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogOptions {
  /// The filters of the dialog.
  #[serde(default)]
  pub filters: Vec<DialogFilter>,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum Cmd {
  /// The open dialog API.
  OpenDialog {
    options: OpenDialogOptions,
  },
  /// The save dialog API.
  SaveDialog {
    options: SaveDialogOptions,
  },
  MessageDialog {
    message: String,
  },
  AskDialog {
    title: Option<String>,
    message: String,
  },
}

impl Cmd {
  #[allow(unused_variables)]
  pub fn run<R: Runtime>(self, window: Window<R>) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(dialog_open)]
      Self::OpenDialog { options } => open(&window, options),
      #[cfg(not(dialog_open))]
      Self::OpenDialog { .. } => Err(crate::Error::ApiNotAllowlisted("dialog > open".to_string())),

      #[cfg(dialog_save)]
      Self::SaveDialog { options } => save(window, options),
      #[cfg(not(dialog_save))]
      Self::SaveDialog { .. } => Err(crate::Error::ApiNotAllowlisted("dialog > save".to_string())),

      Self::MessageDialog { message } => {
        let exe = std::env::current_exe()?;
        let app_name = exe
          .file_stem()
          .expect("failed to get binary filename")
          .to_string_lossy()
          .to_string();
        message_dialog(Some(&window), app_name, message);
        Ok(().into())
      }
      Self::AskDialog { title, message } => {
        let exe = std::env::current_exe()?;
        let answer = ask(
          &window,
          title.unwrap_or_else(|| {
            exe
              .file_stem()
              .expect("failed to get binary filename")
              .to_string_lossy()
              .to_string()
          }),
          message,
        )?;
        Ok(answer)
      }
    }
  }
}

#[cfg(any(dialog_open, dialog_save))]
fn set_default_path(
  mut dialog_builder: FileDialogBuilder,
  default_path: PathBuf,
) -> FileDialogBuilder {
  if default_path.is_file() || !default_path.exists() {
    if let (Some(parent), Some(file_name)) = (default_path.parent(), default_path.file_name()) {
      dialog_builder = dialog_builder.set_directory(parent);
      dialog_builder = dialog_builder.set_file_name(&file_name.to_string_lossy().to_string());
    } else {
      dialog_builder = dialog_builder.set_directory(default_path);
    }
    dialog_builder
  } else {
    dialog_builder.set_directory(default_path)
  }
}

/// Shows an open dialog.
#[cfg(dialog_open)]
#[allow(unused_variables)]
pub fn open<R: Runtime>(
  window: &Window<R>,
  options: OpenDialogOptions,
) -> crate::Result<InvokeResponse> {
  let mut dialog_builder = FileDialogBuilder::new();
  #[cfg(any(windows, target_os = "macos"))]
  {
    dialog_builder = dialog_builder.set_parent(&crate::api::dialog::window_parent(window)?);
  }
  if let Some(default_path) = options.default_path {
    if !default_path.exists() {
      return Err(crate::Error::DialogDefaultPathNotExists(default_path));
    }
    dialog_builder = set_default_path(dialog_builder, default_path);
  }
  for filter in options.filters {
    let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
    dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
  }

  let (tx, rx) = channel();

  if options.directory {
    dialog_builder.pick_folder(move |p| tx.send(p.into()).unwrap());
  } else if options.multiple {
    dialog_builder.pick_files(move |p| tx.send(p.into()).unwrap());
  } else {
    dialog_builder.pick_file(move |p| tx.send(p.into()).unwrap());
  }

  Ok(rx.recv().unwrap())
}

/// Shows a save dialog.
#[cfg(dialog_save)]
#[allow(unused_variables)]
pub fn save<R: Runtime>(
  window: Window<R>,
  options: SaveDialogOptions,
) -> crate::Result<InvokeResponse> {
  let mut dialog_builder = FileDialogBuilder::new();
  #[cfg(any(windows, target_os = "macos"))]
  {
    dialog_builder = dialog_builder.set_parent(&crate::api::dialog::window_parent(&window)?);
  }
  if let Some(default_path) = options.default_path {
    dialog_builder = set_default_path(dialog_builder, default_path);
  }
  for filter in options.filters {
    let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
    dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
  }
  let (tx, rx) = channel();
  dialog_builder.save_file(move |p| tx.send(p).unwrap());
  Ok(rx.recv().unwrap().into())
}

/// Shows a dialog with a yes/no question.
pub fn ask<R: Runtime>(
  window: &Window<R>,
  title: String,
  message: String,
) -> crate::Result<InvokeResponse> {
  let (tx, rx) = channel();
  ask_dialog(Some(window), title, message, move |m| tx.send(m).unwrap());
  Ok(rx.recv().unwrap().into())
}
