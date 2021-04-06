use super::InvokeResponse;
use crate::runtime::{window::Window, Dispatch, Params};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[cfg(global_shortcut_all)]
use crate::api::shortcuts::ShortcutManager;

#[cfg(global_shortcut_all)]
type ShortcutManagerHandle = Arc<Mutex<ShortcutManager>>;

#[cfg(global_shortcut_all)]
pub fn manager_handle() -> &'static ShortcutManagerHandle {
  static MANAGER: Lazy<ShortcutManagerHandle> = Lazy::new(Default::default);
  &MANAGER
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Register a global shortcut.
  Register { shortcut: String, handler: String },
  /// Register a list of global shortcuts.
  RegisterAll {
    shortcuts: Vec<String>,
    handler: String,
  },
  /// Unregister a global shortcut.
  Unregister { shortcut: String },
  /// Unregisters all registered shortcuts.
  UnregisterAll,
  /// Determines whether the given hotkey is registered or not.
  IsRegistered { shortcut: String },
}

#[cfg(global_shortcut_all)]
fn register_shortcut<D: Dispatch>(
  dispatcher: D,
  manager: &mut ShortcutManager,
  shortcut: String,
  handler: String,
) -> crate::Result<()> {
  manager.register(shortcut.clone(), move || {
    let callback_string = crate::api::rpc::format_callback(
      handler.to_string(),
      serde_json::Value::String(shortcut.clone()),
    );
    let _ = dispatcher.eval_script(callback_string.as_str());
  })?;
  Ok(())
}

#[cfg(not(global_shortcut_all))]
impl Cmd {
  pub fn run<M: Params>(self, _window: Window<M>) -> crate::Result<InvokeResponse> {
    Err(crate::Error::ApiNotAllowlisted(
      "globalShortcut > all".to_string(),
    ))
  }
}

#[cfg(global_shortcut_all)]
impl Cmd {
  pub fn run<M: Params>(self, window: Window<M>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Register { shortcut, handler } => {
        let dispatcher = window.dispatcher();
        let mut manager = manager_handle().lock().unwrap();
        register_shortcut(dispatcher, &mut manager, shortcut, handler)?;
        Ok(().into())
      }
      Self::RegisterAll { shortcuts, handler } => {
        let dispatcher = window.dispatcher();
        let mut manager = manager_handle().lock().unwrap();
        for shortcut in shortcuts {
          let dispatch = dispatcher.clone();
          register_shortcut(dispatch, &mut manager, shortcut, handler.clone())?;
        }
        Ok(().into())
      }
      Self::Unregister { shortcut } => {
        let mut manager = manager_handle().lock().unwrap();
        manager.unregister(shortcut)?;
        Ok(().into())
      }
      Self::UnregisterAll => {
        let mut manager = manager_handle().lock().unwrap();
        manager.unregister_all()?;
        Ok(().into())
      }
      Self::IsRegistered { shortcut } => {
        let manager = manager_handle().lock().unwrap();
        Ok(manager.is_registered(shortcut)?.into())
      }
    }
  }
}
