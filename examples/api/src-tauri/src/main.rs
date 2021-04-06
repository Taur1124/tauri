#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

use serde::Serialize;

#[derive(Serialize)]
struct Reply {
  data: String,
}

fn main() {
  tauri::AppBuilder::default()
    .on_page_load(|window, _| {
      let window_ = window.clone();
      window.listen("js-event".into(), move |event| {
        println!("got js-event with message '{:?}'", event.payload());
        let reply = Reply {
          data: "something else".to_string(),
        };

        window_
          .emit(&"rust-event".into(), Some(reply))
          .expect("failed to emit");
      });
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request
    ])
    .build(tauri::generate_context!())
    .run()
    .expect("error while running tauri application");
}
