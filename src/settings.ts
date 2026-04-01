import { getAppConfig, resetAppConfig, updateAppConfig, type AppConfig } from "./config";

function qs<T extends HTMLElement>(sel: string): T {
  const el = document.querySelector(sel);
  if (!el) throw new Error(`Missing element: ${sel}`);
  return el as T;
}

function setStatus(msg: string, kind: "ok" | "error" | "" = "") {
  const el = document.querySelector<HTMLElement>("#cfg-status");
  if (!el) return;
  el.textContent = msg;
  el.classList.remove("status-ok", "status-error");
  if (kind === "ok") el.classList.add("status-ok");
  if (kind === "error") el.classList.add("status-error");
}

function fillForm(cfg: AppConfig) {
  (qs<HTMLInputElement>("#cfg-model")).value = cfg.translation_model ?? "";
  (qs<HTMLInputElement>("#cfg-base-url")).value = cfg.translation_base_url ?? "";
  (qs<HTMLInputElement>("#cfg-api-key")).value = cfg.translation_api_key ?? "";
  (qs<HTMLInputElement>("#cfg-hotkey")).value = cfg.hotkey ?? "";
}

function setupApiKeyToggle() {
  const input = qs<HTMLInputElement>("#cfg-api-key");
  const btn = qs<HTMLButtonElement>("#btn-toggle-api-key");
  const sr = btn.querySelector<HTMLElement>(".sr-only");
  const eye = btn.querySelector<HTMLElement>(".icon-eye");
  const eyeOff = btn.querySelector<HTMLElement>(".icon-eye-off");

  const sync = () => {
    const visible = input.type === "text";

    btn.setAttribute("aria-pressed", visible ? "true" : "false");
    btn.setAttribute("aria-label", visible ? "Hide API key" : "Show API key");
    if (sr) sr.textContent = "";

    if (eye) eye.style.display = visible ? "block" : "none";
    if (eyeOff) eyeOff.style.display = visible ? "none" : "block";
  };

  btn.addEventListener("click", () => {
    try {
      input.type = input.type === "password" ? "text" : "password";
    } catch {
      // ignored
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
    setStatus("已加载", "ok");
  } catch (e) {
    setStatus(`加载失败：${String(e)}`, "error");
  }
}

async function save() {
  setStatus("保存中…");
  const patch: Partial<AppConfig> = {
    translation_model: qs<HTMLInputElement>("#cfg-model").value.trim(),
    translation_base_url: qs<HTMLInputElement>("#cfg-base-url").value.trim(),
    translation_api_key: qs<HTMLInputElement>("#cfg-api-key").value.trim(),
    hotkey: qs<HTMLInputElement>("#cfg-hotkey").value.trim(),
  };
  try {
    await updateAppConfig(patch);
    setStatus("已保存", "ok");
  } catch (e) {
    setStatus(`保存失败：${String(e)}`, "error");
  }
}

async function reset() {
  setStatus("重置中…");
  try {
    const cfg = await resetAppConfig();
    fillForm(cfg);
    setStatus("已重置", "ok");
  } catch (e) {
    setStatus(`重置失败：${String(e)}`, "error");
  }
}

window.addEventListener("DOMContentLoaded", () => {
  setupApiKeyToggle();

  qs<HTMLFormElement>("#settings-form").addEventListener("submit", (e) => {
    e.preventDefault();
    void save();
  });
  qs<HTMLButtonElement>("#btn-reload").addEventListener("click", () => void load());
  qs<HTMLButtonElement>("#btn-reset").addEventListener("click", () => void reset());
  void load();
});
