use objc2::ClassType;
use objc2_app_kit::{NSFrameRect, NSWindow};
use objc2_foundation::{CGFloat, CGPoint, NSRect};
use tao::platform::macos::WindowExtMacOS;

impl WindowExt for tao::window::Window {
  // based on electron implementation
  // https://github.com/electron/electron/blob/15db63e26df3e3d59ce6281f030624f746518511/shell/browser/native_window_mac.mm#L474
  fn set_enabled(&self, enabled: bool) {
    let ns_window: &NSWindow = unsafe { &*window.ns_window().cast() };
    if (!enabled) {
      let frame = ns_window.frame();
      let allocated = NSWindow::alloc();
      let sheet = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          allocated,
          frame,
          NSWindowStyleMaskTitled,
          NSBackingStoreBuffered,
          false,
        )
      };
      unsafe { sheet.setAlphaValue(0.5) };
      ns_window.bebeginSheet_completionHandler(sheet, None)
    } else if let Some(attached) = unsafe { ns_window.attachedSheet() } {
      unsafe { ns_window.endSheet(&attached) };
    }
  }

  fn is_enabled(&self) -> bool {
    let ns_window: &NSWindow = unsafe { &*window.ns_window().cast() };
    unsafe { ns_window.attachedSheet() }.is_some()
  }

  fn center(&self) {
    let ns_window: &NSWindow = unsafe { &*window.ns_window().cast() };
    ns_window.center();
  }
}
