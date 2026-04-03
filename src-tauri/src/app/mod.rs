pub mod config;
pub mod history;
pub mod selection;
pub mod window_manager;

use tauri::{AppHandle, Manager};
use tauri_plugin_store::{Store, StoreBuilder};
pub(crate) fn create_store(app: &AppHandle) -> Result<std::sync::Arc<Store<tauri::Wry>>, String> {
    let store_path = "copylot_store.bin";

    StoreBuilder::new(app.app_handle(), store_path)
        .build()
        .map_err(|e| format!("Failed to build store: {:?}", e))
}

/// @see: https://github.com/drl990114/MarkFlowy/blob/f63e66dfab81444bc134504e4e922bc6bf4856a9/apps/desktop/src-tauri/src/app/conf.rs
pub(crate) trait StoreExt
where
    Self: Sized + serde::Serialize + serde::de::DeserializeOwned + Clone + Default,
{
    fn store_key() -> &'static str;

    fn load_from_store(app: &AppHandle) -> Result<Self, String> {
        let store = create_store(app)?;
        match store.get(Self::store_key()) {
            Some(json) => serde_json::from_value::<Self>(json)
                .map_err(|e| format!("Failed to parse JSON for {}: {:?}", Self::store_key(), e)),
            None => {
                let default = Self::default();
                let _ = store.set(
                    Self::store_key(),
                    serde_json::to_value(&default).map_err(|e| {
                        format!(
                            "Failed to serialize default for {}: {:?}",
                            Self::store_key(),
                            e
                        )
                    })?,
                );
                Ok(default)
            }
        }
    }

    fn write_to_store(self, app: &AppHandle) -> Result<Self, String> {
        let store = create_store(app)?;

        let value = serde_json::to_value(&self)
            .map_err(|e| format!("Failed to serialize {}: {}", Self::store_key(), e))?;

        store.set(Self::store_key().to_string(), value);
        store
            .save()
            .map_err(|e| format!("Failed to save store for {}: {:?}", Self::store_key(), e))?;
        Ok(self)
    }

    fn read_with_app(app: &AppHandle) -> Self {
        match Self::load_from_store(app) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to load {} from store: {e}", Self::store_key());
                Self::default()
            }
        }
    }

    fn write_with_app(self, app: &AppHandle) -> Self {
        match self.clone().write_to_store(app) {
            Ok(data) => data,
            Err(e) => {
                log::error!("Failed to write {} to store: {e}", Self::store_key());
                Self::default()
            }
        }
    }

    fn reset_with_app(self, app: &AppHandle) -> Self {
        let store = match create_store(app) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };
        store.delete(Self::store_key());
        if store.save().is_ok() {
            return self.clone().write_to_store(app).unwrap_or_default();
        }

        Self::default()
    }
}
