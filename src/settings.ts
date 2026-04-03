import { getAppConfig, resetAppConfig, updateAppConfig, type AppConfig } from "./config";
import { qs } from "./ui/dom";
import { toast } from "./ui/toast";
import { setupTabs } from "./ui/tabs";
import { initHistoryTab } from "./history";

function fillForm(cfg: AppConfig) {
  (qs<HTMLInputElement>("#cfg-model")).value = cfg.translation_model ?? "";
  (qs<HTMLInputElement>("#cfg-base-url")).value = cfg.translation_base_url ?? "";
  (qs<HTMLInputElement>("#cfg-api-key")).value = cfg.translation_api_key ?? "";
  (qs<HTMLInputElement>("#cfg-hotkey")).value = cfg.hotkey ?? "";
}

function setupApiKeyToggle() {
  const input = qs<HTMLInputElement>("#cfg-api-key");
  const btn = qs<HTMLButtonElement>("#btn-toggle-api-key");

  const sync = () => {
    const visible = input.type === "text";
    btn.setAttribute("aria-pressed", visible ? "true" : "false");
    btn.setAttribute("aria-label", visible ? "隐藏 API Key" : "显示 API Key");
  };

  btn.addEventListener("click", () => {
    const prev = input.type;
    try {
      input.type = input.type === "password" ? "text" : "password";
    } catch {
      input.type = prev;
    }
    sync();
    input.focus();
  });

  sync();
}

async function load() {
  try {
    const cfg = await getAppConfig();
    fillForm(cfg);
  } catch (e) {
    toast(`加载配置失败：${String(e)}`, "error");
  }
}

async function save() {
  const patch: Partial<AppConfig> = {
    translation_model: qs<HTMLInputElement>("#cfg-model").value.trim(),
    translation_base_url: qs<HTMLInputElement>("#cfg-base-url").value.trim(),
    translation_api_key: qs<HTMLInputElement>("#cfg-api-key").value.trim(),
    hotkey: qs<HTMLInputElement>("#cfg-hotkey").value.trim(),
  };
  try {
    await updateAppConfig(patch);
    toast("已保存", "ok");
  } catch (e) {
    toast(`保存失败：${String(e)}`, "error");
  }
}

async function reset() {
  try {
    const cfg = await resetAppConfig();
    fillForm(cfg);
    toast("已重置", "ok");
  } catch (e) {
    toast(`重置失败：${String(e)}`, "error");
  }
}

window.addEventListener("DOMContentLoaded", () => {
  setupApiKeyToggle();

  // tabs + history
  const history = initHistoryTab();
  setupTabs({
    tabSelector: "[role=tab]",
    panelSelector: "[role=tabpanel]",
    activeClass: "is-active",
    onActivate: (id) => {
      if (id === "tab-history") void history.refresh();
    },
  });

  qs<HTMLFormElement>("#settings-form").addEventListener("submit", (e) => {
    e.preventDefault();
    void save();
  });
  qs<HTMLButtonElement>("#btn-reload").addEventListener("click", () => void load());
  qs<HTMLButtonElement>("#btn-reset").addEventListener("click", () => void reset());
  void load();
});
