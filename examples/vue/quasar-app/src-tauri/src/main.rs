#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
  tauri::AppBuilder::new()
    .setup(|_webview| {
      let handle = _webview.handle();
      tauri::event::listen(String::from("hello"), move |msg| {
        #[derive(Serialize)]
        pub struct Reply {
          pub msg: String,
          pub rep: String,
        }

        let reply = Reply {
          msg: format!("{}", msg).to_string(),
          rep: "something else".to_string(),
        };

        tauri::event::emit(
          &handle,
          String::from("reply"),
          serde_json::to_string(&reply).unwrap(),
        );

        println!("Message from emit:hello => {}", msg);
      });
    })
    .build()
    .run();
}
