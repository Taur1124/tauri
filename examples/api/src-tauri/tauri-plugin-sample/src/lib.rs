use tauri::{
  plugin::{Builder, TauriPlugin},
  Runtime,
};

const PLUGIN_NAME: &str = "sample";
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.sample";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_sample(webview: tauri::cocoa::base::id);
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new(PLUGIN_NAME)
    .setup(|app| {
      #[cfg(target_os = "android")]
      app.initialize_android_plugin(PLUGIN_NAME, PLUGIN_IDENTIFIER, "ExamplePlugin")?;
      #[cfg(target_os = "ios")]
      app.initialize_ios_plugin(init_plugin_sample)?;
      Ok(())
    })
    .build()
}
