use crate::app::StoreExt;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub translation_api_key: Option<String>,
    pub translation_model: Option<String>,
    pub translation_base_url: Option<String>,

    pub hotkey: Option<String>,
}

const CONFIG_STORE_KEY: &str = "copylot_config";

impl StoreExt for AppConfig {
    fn store_key() -> &'static str {
        CONFIG_STORE_KEY
    }
}

impl AppConfig {
    fn normalize_opt_str(v: Option<&str>) -> Option<String> {
        v.map(|s| s.trim().trim_matches('"').to_string())
    }

    pub fn patch(self, json: serde_json::Value) -> Self {
        log::trace!("Patching config with JSON: {:?}", json);
        let mut config = self.clone();

        if let Some(api_key) =
            Self::normalize_opt_str(json.get("translation_api_key").and_then(|v| v.as_str()))
        {
            let masked = api_key.chars().take(8).collect::<String>().to_string() + "******";
            log::trace!("Updating translation_api_key to: {}", masked);
            config.translation_api_key = Some(api_key);
        }

        if let Some(model) =
            Self::normalize_opt_str(json.get("translation_model").and_then(|v| v.as_str()))
        {
            log::trace!("Updating translation_model to: {}", model);
            config.translation_model = Some(model);
        }

        if let Some(base_url) =
            Self::normalize_opt_str(json.get("translation_base_url").and_then(|v| v.as_str()))
        {
            log::trace!("Updating translation_base_url to: {}", base_url);
            config.translation_base_url = Some(base_url);
        }

        if let Some(hotkey) = Self::normalize_opt_str(json.get("hotkey").and_then(|v| v.as_str())) {
            log::trace!("Updating hotkey to: {}", hotkey);
            config.hotkey = Some(hotkey);
        }

        config
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            translation_api_key: None,
            translation_model: Some("deepseek-chat".to_string()),
            translation_base_url: Some("https://api.deepseek.com/v1".to_string()),

            hotkey: Some("Ctrl+Alt+Q".to_string()),
        }
    }
}

pub mod cmd {
    use super::{AppConfig, StoreExt};
    use tauri::{command, AppHandle};

    #[cfg(desktop)]
    pub(crate) fn apply_hotkey(app: &AppHandle, new_hotkey_str: Option<&str>) {
        use std::str::FromStr;
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

        let Some(new_hotkey_str) = new_hotkey_str else {
            return;
        };
        let Ok(new_hotkey) = Shortcut::from_str(new_hotkey_str) else {
            log::error!("invalid hotkey string: {new_hotkey_str}");
            return;
        };

        // 旧 hotkey（如果存在）
        let old_hotkey = AppConfig::read_with_app(app)
            .hotkey
            .and_then(|s| Shortcut::from_str(&s).ok());

        if let Some(old) = old_hotkey {
            if app.global_shortcut().is_registered(old) {
                if let Err(e) = app.global_shortcut().unregister(old) {
                    log::error!("unregister old hotkey failed: {e:?}");
                }
            }
        }

        log::info!("register hotkey: {}", new_hotkey);
        if let Err(e) = app.global_shortcut().register(new_hotkey) {
            log::error!("register new hotkey failed: {e:?}");
            return;
        }
    }

    #[command]
    pub fn get_app_conf(app: AppHandle) -> AppConfig {
        AppConfig::read_with_app(&app)
    }

    #[command]
    pub fn update_app_conf(app: AppHandle, patch: serde_json::Value) {
        let prev = AppConfig::read_with_app(&app);
        let next = prev.clone().patch(patch);

        // 热键变化：立即生效
        if prev.hotkey.as_deref() != next.hotkey.as_deref() {
            apply_hotkey(&app, next.hotkey.as_deref());
        }
        next.write_with_app(&app);
    }

    #[command]
    pub fn reset_app_conf(app: AppHandle) -> AppConfig {
        let next = AppConfig::default().reset_with_app(&app);
        apply_hotkey(&app, next.hotkey.as_deref());
        next
    }
}
