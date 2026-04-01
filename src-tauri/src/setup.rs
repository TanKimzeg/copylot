use crate::app::config::AppConfig;
use crate::app::config::cmd as config_cmd;
use crate::window_manager::*;
use tauri::App;

pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        let app_handle = app.handle().clone();
        match ensure_translator_window(&app_handle) {
            Ok(w) => {
                log::trace!("translator window ensured: {}", w.label());
            }
            Err(e) => {
                log::error!("ensure_translator_window on startup failed: {e:?}");
            }
        }

        // 从 AppConfig（copylot_store.bin）读取配置
        let cfg = AppConfig::read_with_app(&app_handle);
        log::info!(
            "Hotkey loaded from AppConfig: {}",
            cfg.hotkey.as_deref().unwrap_or("Ctrl+Alt+Q")
        );

        // 复用同一套热键应用逻辑（包含 unregister/register + manage state）
        config_cmd::apply_hotkey(&app_handle, cfg.hotkey.as_deref());
    }

    Ok(())
}
