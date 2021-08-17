// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[cfg(not(any(feature = "api-all", feature = "shell-all", feature = "shell-execute")))]
fn main() {
  eprintln!("Not supported without `api-all`, `shell-all` or `shell-execute`")
}

#[cfg(any(feature = "api-all", feature = "shell-all", feature = "shell-execute"))]
fn main() {
  use tauri::{
    api::{
      path::{resolve_path, BaseDirectory},
      process::{Command, CommandEvent},
    },
    Manager,
  };
  let context = tauri::generate_context!("../../examples/resources/src-tauri/tauri.conf.json");
  let script_path = resolve_path(
    context.config(),
    context.package_info(),
    "assets/index.js",
    Some(BaseDirectory::Resource),
  )
  .unwrap();
  tauri::Builder::default()
    .setup(move |app| {
      let window = app.get_window("main").unwrap();
      let script_path = script_path.to_string_lossy().to_string();
      tauri::async_runtime::spawn(async move {
        let (mut rx, _child) = Command::new("node")
          .args(&[script_path])
          .spawn()
          .expect("Failed to spawn node");

        #[allow(clippy::collapsible_match)]
        while let Some(event) = rx.recv().await {
          if let CommandEvent::Stdout(line) = event {
            window
              .emit("message", Some(format!("'{}'", line)))
              .expect("failed to emit event");
          }
        }
      });

      Ok(())
    })
    .run(context)
    .expect("error while running tauri application");
}
