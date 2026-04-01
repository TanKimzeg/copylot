/// @see: https://github.com/drl990114/MarkFlowy/blob/f63e66dfab81444bc134504e4e922bc6bf4856a9/apps/desktop/src-tauri/src/app/conf.rs
use tauri::{AppHandle, Manager};
use tauri_plugin_store::{Store, StoreBuilder};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub translation_api_key: Option<String>,
    pub translation_model: Option<String>,
    pub translation_base_url: Option<String>,

    pub hotkey: Option<String>,
}

fn create_store(app: &AppHandle) -> Result<std::sync::Arc<Store<tauri::Wry>>, String> {
    let store_path = "copylot_store.bin";

    StoreBuilder::new(app.app_handle(), store_path)
        .build()
        .map_err(|e| format!("Failed to build store: {:?}", e))
}

pub const STORE_KEY: &str = "copylot_config";

impl AppConfig {
    pub fn new() -> Self {
        Self {
            translation_api_key: None,
            translation_model: Some("deepseek-chat".to_string()),
            translation_base_url: Some("https://api.deepseek.com/v1".to_string()),

            hotkey: Some("Ctrl+Alt+Q".to_string()),
        }
    }

    pub fn load_from_store(app: &AppHandle) -> Result<Self, String> {
        let store = create_store(app)?;
        match store.get(STORE_KEY) {
            Some(config_json) => serde_json::from_value::<AppConfig>(config_json)
                .map_err(|e| format!("Failed to parse config JSON: {:?}", e)),
            None => {
                let default_config = Self::new();
                let _ = store.set(
                    STORE_KEY,
                    serde_json::to_value(&default_config)
                        .map_err(|e| format!("Failed to serialize default config: {:?}", e))?,
                );
                Ok(default_config)
            }
        }
    }

    pub fn write_to_store(self, app: &AppHandle) -> Result<Self, String> {
        let store = create_store(app)?;

        let value = serde_json::to_value(&self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        store.set(STORE_KEY.to_string(), value);
        store
            .save()
            .map_err(|e| format!("Failed to save store: {:?}", e))?;
        Ok(self)
    }

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

    pub fn read_with_app(app: &AppHandle) -> Self {
        match Self::load_from_store(app) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to load config from store: {e}");
                Self::default()
            }
        }
    }

    pub fn write_with_app(self, app: &AppHandle) -> Self {
        match self.clone().write_to_store(app) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to write config to store: {e}");
                Self::default()
            }
        }
    }

    pub fn reset_with_app(self, app: &AppHandle) -> Self {
        let store = match create_store(app) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        store.delete(STORE_KEY);
        if store.save().is_ok() {
            return self
                .clone()
                .write_to_store(app)
                .unwrap_or_else(|_| Self::default());
        }

        Self::default()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub mod cmd {
    use log;

    use super::AppConfig;
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
        match AppConfig::load_from_store(&app) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to load config from store: {e}");
                AppConfig::default()
            }
        }
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
