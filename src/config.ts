import { invoke } from "@tauri-apps/api/core";

export type AppConfig = {
  translation_api_key?: string | null;
  translation_model?: string | null;
  translation_base_url?: string | null;
  hotkey?: string | null;
};

export async function getAppConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_app_conf");
}

export async function updateAppConfig(patch: Partial<AppConfig>): Promise<void> {
  // 后端接收 serde_json::Value，这里直接传对象即可
  await invoke<void>("update_app_conf", { patch });
}

export async function resetAppConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("reset_app_conf");
}
