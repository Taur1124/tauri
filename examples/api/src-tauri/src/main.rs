// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
mod menu;

use serde::Serialize;
use tauri::{
  CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowBuilder, WindowUrl,
};

#[derive(Serialize)]
struct Reply {
  data: String,
}

fn main() {
  tauri::Builder::default()
    .on_page_load(|window, _| {
      let window_ = window.clone();
      window.listen("js-event", move |event| {
        println!("got js-event with message '{:?}'", event.payload());
        let reply = Reply {
          data: "something else".to_string(),
        };

        window_
          .emit("rust-event", Some(reply))
          .expect("failed to emit");
      });
    })
    .menu(menu::get_menu())
    .on_menu_event(|event| {
      println!("{:?}", event.menu_item_id());
    })
    .system_tray(
      SystemTray::new().with_menu(
        SystemTrayMenu::new()
          .add_item(CustomMenuItem::new("toggle".into(), "Toggle"))
          .add_item(CustomMenuItem::new("new".into(), "New window")),
      ),
    )
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::LeftClick {
        position: _,
        size: _,
        ..
      } => {
        let window = app.get_window("main").unwrap();
        window.show().unwrap();
        window.set_focus().unwrap();
      }
      SystemTrayEvent::MenuItemClick { id, .. } => {
        let item_handle = app.tray_handle().get_item(&id);
        match id.as_str() {
          "toggle" => {
            let window = app.get_window("main").unwrap();
            let new_title = if window.is_visible().unwrap() {
              window.hide().unwrap();
              "Show"
            } else {
              window.show().unwrap();
              "Hide"
            };
            item_handle.set_title(new_title).unwrap();
          }
          "new" => app
            .create_window(
              "new".into(),
              WindowUrl::App("index.html".into()),
              |window_builder, webview_attributes| {
                (window_builder.title("Tauri"), webview_attributes)
              },
            )
            .unwrap(),
          _ => {}
        }
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
