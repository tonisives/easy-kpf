use tauri::Manager;

pub fn activate_and_show_window(app_handle: &tauri::AppHandle) {
  if let Some(window) = app_handle.get_webview_window("main") {
    let _ = window.show();
    let _ = window.set_focus();
  }
  #[cfg(target_os = "macos")]
  {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    #[allow(unsafe_code)]
    if let Some(mtm) = MainThreadMarker::new() {
      let app = NSApplication::sharedApplication(mtm);
      #[allow(deprecated)]
      app.activateIgnoringOtherApps(true);
    }
  }
}
