use tauri::{Manager, Runtime};

#[tauri::command]
pub async fn create_settings<R: Runtime>(handle: tauri::AppHandle<R>) -> Result<(), tauri::Error> {
    // let _ = tauri::WebviewWindowBuilder::new(
    //     &handle,
    //     "settings",
    //     tauri::WebviewUrl::App("settings".into()),
    // )
    // .decorations(false)
    // .shadow(false)
    // .resizable(false)
    // .transparent(true)
    // .build()?;

    let window = handle.get_webview_window("settings").unwrap();
    window.show()?;

    Ok(())
}
