use crate::api::dialog::{
  ask as ask_dialog, message as message_dialog, AskResponse, FileDialogBuilder,
};
use serde::Deserialize;
use serde_json::Value as JsonValue;

use std::path::PathBuf;

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
  pub async fn run(self) -> crate::Result<JsonValue> {
    match self {
      Self::OpenDialog { options } => {
        #[cfg(open_dialog)]
        return open(options);
        #[cfg(not(open_dialog))]
        Err(crate::Error::ApiNotAllowlisted("title".to_string()));
      }
      Self::SaveDialog { options } => {
        #[cfg(save_dialog)]
        return save(options);
        #[cfg(not(save_dialog))]
        Err(crate::Error::ApiNotAllowlisted("saveDialog".to_string()));
      }
      Self::MessageDialog { message } => {
        let exe = std::env::current_exe()?;
        let app_name = exe
          .file_stem()
          .expect("failed to get binary filename")
          .to_string_lossy()
          .to_string();
        message_dialog(app_name, message);
        Ok(JsonValue::Null)
      }
      Self::AskDialog { title, message } => {
        let exe = std::env::current_exe()?;
        let answer = ask(
          title.unwrap_or_else(|| {
            exe
              .file_stem()
              .expect("failed to get binary filename")
              .to_string_lossy()
              .to_string()
          }),
          message,
        )?;
        Ok(JsonValue::Bool(answer))
      }
    }
  }
}

/// Shows an open dialog.
#[cfg(open_dialog)]
pub fn open(options: OpenDialogOptions) -> crate::Result<JsonValue> {
  let mut dialog_builder = FileDialogBuilder::new();
  if let Some(default_path) = options.default_path {
    dialog_builder = dialog_builder.set_directory(default_path);
  }
  for filter in options.filters {
    let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
    dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
  }
  let response = if options.directory {
    serde_json::to_value(dialog_builder.pick_folder())?
  } else if options.multiple {
    serde_json::to_value(dialog_builder.pick_files())?
  } else {
    serde_json::to_value(dialog_builder.pick_file())?
  };
  Ok(response)
}

/// Shows a save dialog.
#[cfg(save_dialog)]
pub fn save(options: SaveDialogOptions) -> crate::Result<JsonValue> {
  let mut dialog_builder = FileDialogBuilder::new();
  if let Some(default_path) = options.default_path {
    dialog_builder = dialog_builder.set_directory(default_path);
  }
  for filter in options.filters {
    let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
    dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
  }
  let response = dialog_builder.save_file();
  Ok(serde_json::to_value(response)?)
}

/// Shows a dialog with a yes/no question.
pub fn ask(title: String, message: String) -> crate::Result<bool> {
  match ask_dialog(title, message) {
    AskResponse::Yes => Ok(true),
    _ => Ok(false),
  }
}
