use crate::app::StoreExt;
use tauri::AppHandle;

const HISTORY_STORAGE_KEY: &str = "translation_history";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TranslationHistory {
    entries: Vec<String>,
}

impl StoreExt for TranslationHistory {
    fn store_key() -> &'static str {
        HISTORY_STORAGE_KEY
    }
}

impl TranslationHistory {
    pub fn add_record(app: &AppHandle, entry: String) -> Self {
        let mut history = Self::read_with_app(app);
        history.entries.push(entry);
        history.write_with_app(app)
    }

    pub fn delete_record(app: &AppHandle, index: usize) -> Self {
        let mut history = Self::read_with_app(app);
        if index < history.entries.len() {
            history.entries.remove(index);
            history.write_with_app(app)
        } else {
            history
        }
    }
}

impl Default for TranslationHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

pub mod cmd {
    use super::{StoreExt, TranslationHistory};
    use tauri::{command, AppHandle};

    #[command]
    pub fn get_history(app: AppHandle) -> TranslationHistory {
        TranslationHistory::read_with_app(&app)
    }

    #[command]
    pub fn clear_history(app: AppHandle) -> TranslationHistory {
        TranslationHistory::default().reset_with_app(&app)
    }

    #[tauri::command]
    pub fn delete_history_record(app: AppHandle, index: usize) -> TranslationHistory {
        TranslationHistory::delete_record(&app, index)
    }
}
