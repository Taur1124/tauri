use tauri_hotkey::{parse_hotkey, HotkeyManager};

/// The shortcut manager builder.
#[derive(Default)]
pub struct ShortcutManager(HotkeyManager);

impl ShortcutManager {
  /// Initializes a new instance of the shortcut manager.
  pub fn new() -> Self {
    Default::default()
  }

  /// Determines whether the given hotkey is registered or not.
  pub fn is_registered(&self, shortcut: String) -> crate::Result<bool> {
    let hotkey = parse_hotkey(&shortcut)?;
    Ok(self.0.is_registered(&hotkey))
  }

  /// Registers a new shortcut handler.
  pub fn register<H: FnMut() + Send + 'static>(
    &mut self,
    shortcut: String,
    handler: H,
  ) -> crate::Result<()> {
    let hotkey = parse_hotkey(&shortcut)?;
    self.0.register(hotkey, handler)?;
    Ok(())
  }

  /// Unregister a previously registered shortcut handler.
  pub fn unregister(&mut self, shortcut: String) -> crate::Result<()> {
    let hotkey = parse_hotkey(&shortcut)?;
    self.0.unregister(&hotkey)?;
    Ok(())
  }

  /// Unregisters all shortcuts registered by this application.
  pub fn unregister_all(&mut self) -> crate::Result<()> {
    self.0.unregister_all()?;
    Ok(())
  }
}
