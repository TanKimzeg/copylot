// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use log;
mod app;
mod setup;
use app::{window_manager, selection, config};
mod llm;
use llm::translation;
use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
struct SelectedTextPayload {
    text: String,
}

// 仅支持 Windows 桌面端
#[cfg(all(windows, not(mobile)))]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        // NOTE: store 插件不要重复注册
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app_handle, shotcut, event| {
                    use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
                    // 这里直接从 AppConfig 读取字符串并解析为 Shortcut，实现即时生效。
                    let cfg = config::AppConfig::read_with_app(&app_handle);
                    if let Some(hk) = cfg.hotkey.as_deref() {
                        if let Ok(hotkey) = hk.parse::<Shortcut>() {
                            log::debug!("shortcut pressed={shotcut}, current={hotkey}");
                            if shotcut != &hotkey {
                                return;
                            }
                        }
                    }
                    if !matches!(event.state(), ShortcutState::Pressed) {
                        return;
                    }

                    let app_handle = app_handle.clone();

                    tauri::async_runtime::spawn(async move {
                        let text = selection::windows::get_selected_text()
                            .await
                            .replace("\r\n", "\n")
                            .trim()
                            .to_string();

                        match window_manager::ensure_translator_window(&app_handle) {
                            Ok(w) => {
                                if let Err(e) =
                                    window_manager::show_translator_right_side(&app_handle, &w)
                                {
                                    log::error!("show_translator_right_side failed: {e:?}");
                                }

                                if !text.is_empty() {
                                    if let Err(e) = w.emit(
                                        "selected-text",
                                        SelectedTextPayload {
                                            // 调用翻译接口
                                            text: translation::invoke(&app_handle, &text).await,
                                        },
                                    ) {
                                        log::error!("emit selected-text failed: {e:?}");
                                    }
                                } else {
                                    log::info!("未选择文本，显示上一次的翻译结果");
                                }
                            }
                            Err(e) => {
                                log::error!("ensure_translator_window failed: {e:?}");
                            }
                        }
                    });
                })
                .build(),
        )
        .setup(|app| setup::init(app))
        .invoke_handler(tauri::generate_handler![
            config::cmd::get_app_conf,
            config::cmd::update_app_conf,
            config::cmd::reset_app_conf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 非 Windows（或 mobile）不支持：明确失败
#[cfg(not(all(windows, not(mobile))))]
pub fn run() {
    panic!("Copylot 当前仅支持 Windows 桌面端（windows desktop）。");
}
