// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::WindowBuilder;

pub fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _payload| {
      let label = window.label().to_string();
      window.listen("clicked".to_string(), move |_payload| {
        println!("got 'clicked' event on window '{label}'");
      });
    })
    .setup(|app| {
      #[allow(unused_mut)]
      let mut builder = WindowBuilder::new(
        app,
        "Rust".to_string(),
        tauri::WindowUrl::App("index.html".into()),
      );
      #[cfg(target_os = "macos")]
      {
        builder = builder.tabbing_identifier("Rust");
      }
      let _window = builder.title("Tauri - Rust").build()?;
      Ok(())
    })
    .run(tauri::build_script_context!())
    .expect("failed to run tauri application");
}
