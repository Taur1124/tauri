// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Open path or URL with `with`, or system default
pub fn open(path: String, with: Option<String>) -> crate::api::Result<()> {
  {
    let exit_status = if let Some(with) = with {
      open::with(&path, &with)
    } else {
      open::that(&path)
    };
    match exit_status {
      Ok(status) => {
        if status.success() {
          Ok(())
        } else {
          Err(crate::api::Error::Shell("open command failed".into()))
        }
      }
      Err(err) => Err(crate::api::Error::Shell(format!(
        "failed to open: {}",
        err.to_string()
      ))),
    }
  }
}
