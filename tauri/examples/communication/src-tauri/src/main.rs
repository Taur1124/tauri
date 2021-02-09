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

#[derive(tauri::FromTauriContext)]
#[config_path = "examples/communication/src-tauri/tauri.conf.json"]
struct Context;

fn main() {
  tauri::AppBuilder::<tauri::flavors::Wry, Context>::new()
    .setup(|dispatcher, _source| async move {
      let mut dispatcher = dispatcher.clone();
      tauri::event::listen(String::from("js-event"), move |msg| {
        println!("got js-event with message '{:?}'", msg);
        let reply = Reply {
          data: "something else".to_string(),
        };

        tauri::event::emit(&mut dispatcher, String::from("rust-event"), Some(reply))
          .expect("failed to emit");
      });
    })
    .invoke_handler(|mut dispatcher, arg| async move {
      use cmd::Cmd::*;
      match serde_json::from_str(&arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            LogOperation { event, payload } => {
              println!("{} {:?}", event, payload);
            }
            PerformRequest {
              endpoint,
              body,
              callback,
              error,
            } => {
              // tauri::execute_promise is a helper for APIs that uses the tauri.promisified JS function
              // so you can easily communicate between JS and Rust with promises
              tauri::execute_promise(
                &mut dispatcher,
                async move {
                  println!("{} {:?}", endpoint, body);
                  // perform an async operation here
                  // if the returned value is Ok, the promise will be resolved with its value
                  // if the returned value is Err, the promise will be rejected with its value
                  // the value is a string that will be eval'd
                  Ok("{ key: 'response', value: [{ id: 3 }] }".to_string())
                },
                callback,
                error,
              )
              .await
            }
          }
          Ok(())
        }
      }
    })
    .build()
    .unwrap()
    .run();
}
