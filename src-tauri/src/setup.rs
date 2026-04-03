use crate::window_manager::*;
use crate::MAIN_WINDOW_LABEL;
use tauri::App;

pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        let app_handle = app.handle().clone();
        // 创建翻译窗口，以便后续快速显示和更新内容时能复用同一个窗口实例
        match ensure_translator_window(&app_handle) {
            Ok(w) => {
                log::trace!("translator window ensured: {}", w.label());
            }
            Err(e) => {
                log::error!("ensure_translator_window on startup failed: {e:?}");
            }
        }

        {
            use crate::app::config::cmd as config_cmd;
            use crate::app::config::AppConfig;
            use crate::app::StoreExt;
            // 从 AppConfig（copylot_store.bin）读取配置
            let cfg = AppConfig::read_with_app(&app_handle);
            log::info!(
                "Hotkey loaded from AppConfig: {}",
                cfg.hotkey.as_deref().unwrap_or("None")
            );

            // 复用同一套热键应用逻辑（包含 unregister/register + manage state）
            config_cmd::apply_hotkey(&app_handle, cfg.hotkey.as_deref());
        }

        {
            // 系统托盘和菜单
            use tauri::{
                menu::{Menu, MenuItem},
                tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
                Manager,
            };
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        log::trace!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        log::trace!("menu item {:?} not handled", event.id);
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        log::trace!("left click pressed and released");
                        // in this example, let's show and focus the main window when the tray is clicked
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;
        }
    }

    Ok(())
}
