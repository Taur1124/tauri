#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;

#[macro_use]
extern crate lazy_static;

extern crate includedir;
extern crate phf;

pub mod api;
pub mod command;
pub mod config;
pub mod dir;
pub mod event;
pub mod file;
pub mod file_system;
pub mod http;
pub mod platform;
pub mod process;
pub mod rpc;
pub mod salt;
pub mod tcp;
pub mod updater;
pub mod version;
#[cfg(feature = "embedded-server")]
pub mod server;
mod app;
pub use app::*;

use tauri_ui::WebView;

use threadpool::ThreadPool;

thread_local!(static POOL: ThreadPool = ThreadPool::new(4));

pub fn spawn<F: FnOnce() -> () + Send + 'static>(what: F) {
  POOL.with(|thread| {
    thread.execute(move || {
      what();
    });
  });
}

pub fn execute_promise<T: 'static, F: FnOnce() -> Result<String, String> + Send + 'static>(
  webview: &mut WebView<'_, T>,
  what: F,
  callback: String,
  error: String,
) {
  let handle = webview.handle();
  POOL.with(|thread| {
    thread.execute(move || {
      let callback_string = rpc::format_callback_result(what(), callback, error);
      handle
        .dispatch(move |_webview| _webview.eval(callback_string.as_str()))
        .unwrap()
    });
  });
}
