// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This is an example of a tauri app built into a dll
// Calling lib_test1 within the dll will launch the webview

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[no_mangle]
pub extern "C" fn run_tauri() {
  tauri::Builder::default()
    .run(tauri::build_script_context!())
    .expect("error while running tauri application");
}
