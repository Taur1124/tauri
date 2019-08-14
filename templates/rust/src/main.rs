#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate tauri;
extern crate tauri_ui;
extern crate serde_json;

#[cfg(not(feature = "dev"))]
extern crate tiny_http;

#[cfg(feature = "dev")]
use clap::{App, Arg};

#[cfg(not(feature = "dev"))]
#[cfg(feature = "embedded-server")]
use std::thread;

mod cmd;

fn main() {
  let debug;
  let content;
  let config = tauri::config::get();
  #[cfg(feature = "embedded-server")]
  let server_url: String;

  #[cfg(feature = "updater")]
  {
    thread::spawn(|| {
      tauri::command::spawn_relative_command(
        "updater".to_string(),
        Vec::new(),
        std::process::Stdio::inherit(),
      )
      .unwrap();
    });
  }

  #[cfg(feature = "dev")]
  {
    let app = App::new("app")
      .version("1.0.0")
      .author("Author")
      .about("About")
      .arg(
        Arg::with_name("url")
          .short("u")
          .long("url")
          .value_name("URL")
          .help("Loads the specified URL into webview")
          .required(true)
          .takes_value(true),
      );

    let matches = app.get_matches();
    content = tauri_ui::Content::Url(matches.value_of("url").unwrap().to_owned());
    debug = true;
  }

  #[cfg(not(feature = "dev"))]
  {
    debug = cfg!(debug_assertions);
    #[cfg(not(feature = "embedded-server"))]
    {
      content = tauri_ui::Content::Html(include_str!("../target/compiled-web/index.html"));
    }
    #[cfg(feature = "embedded-server")]
    {
      let port;
      let port_valid;
      if config.embedded_server.port == "random" {
        match tauri::tcp::get_available_port() {
          Some(available_port) => {
            port = available_port.to_string();
            port_valid = true;
          }
          None => {
            port = "0".to_string();
            port_valid = false;
          }
        }
      } else {
        port = config.embedded_server.port;
        port_valid = tauri::tcp::port_is_available(port.parse::<u16>().expect(&format!("Invalid port {}", port)));
      }
      if port_valid {
        server_url = format!("{}:{}", config.embedded_server.host, port);
        content = tauri_ui::Content::Url(server_url.clone());
      } else {
        panic!(format!("Port {} is not valid or not open", port));
      }
    }
  }

  let webview = tauri_ui::builder()
    .title(&config.window.title)
    .size(config.window.width, config.window.height)
    .resizable(config.window.resizable)
    .debug(debug)
    .user_data(())
    .invoke_handler(|webview, arg| {
      // leave this as is to use the tauri API from your JS code
      if !tauri::api::handler(webview, arg) {
        use cmd::Cmd::*;
        match serde_json::from_str(arg) {
          Err(_) => {}
          Ok(command) => {
            match command {
              // definitions for your custom commands from Cmd here
              MyCustomCommand { argument } => {
                //  your command code
                println!("{}", argument);
              }
            }
          }
        }
      }

      Ok(())
    })
    .content(content)
    .build()
    .unwrap();

  webview
    .handle()
    .dispatch(move |_webview| {
      _webview
        .eval(&format!(
          "window['{queue}'] = [];
          window['{fn}'] = function (payload, salt, ignoreQueue) {{
            window.tauri.promisified({{
              cmd: 'validateSalt',
              salt
            }}).then(function () {{
              const listeners = (window['{listeners}'] && window['{listeners}'][payload.type]) || []

              if (!ignoreQueue && listeners.length === 0) {{ 
                window['{queue}'].push({{ 
                  payload: payload,
                  salt: salt
                 }})
              }}

              for (let i = listeners.length - 1; i >= 0; i--) {{ 
                const listener = listeners[i]
                if (listener.once)
                  listeners.splice(i, 1)
                listener.handler(payload)
              }}
            }})
          }}", 
          fn = tauri::event::emit_function_name(),
          listeners = tauri::event::event_listeners_object_name(),
          queue = tauri::event::event_queue_object_name()
        ))
        .unwrap();

      Ok(())
    })
    .unwrap();

  #[cfg(not(feature = "dev"))]
  {
    #[cfg(feature = "embedded-server")]
    {
      thread::spawn(move || {
        let server = tiny_http::Server::http(server_url.clone()).expect(&format!("Could not start embedded server with the specified url: {}", server_url));
        for request in server.incoming_requests() {
          let mut url = request.url().to_string();
          if url == "/" {
            url = "/index.html".to_string();
          }
          request.respond(tauri::server::asset_response(&url)).unwrap();
        }
      });
    }
  }

  webview.run().unwrap();
}
