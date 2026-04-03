import { invoke } from "@tauri-apps/api/core";
import { qs } from "./ui/dom";
import { toast } from "./ui/toast";

export type TranslationHistory = {
  entries: string[];
};

export async function getHistory(): Promise<TranslationHistory> {
  return invoke<TranslationHistory>("get_history");
}

export async function clearHistory(): Promise<TranslationHistory> {
  return invoke<TranslationHistory>("clear_history");
}

export async function deleteHistoryRecord(index: number): Promise<TranslationHistory> {
  return invoke<TranslationHistory>("delete_history_record", { index });
}

function escapeHtml(s: string) {
  return s
    .split("&")
    .join("&amp;")
    .split("<")
    .join("&lt;")
    .split(">")
    .join("&gt;")
    .split('"')
    .join("&quot;")
    .split("'")
    .join("&#039;");
}

function renderHistory(history: TranslationHistory) {
  const panel = qs<HTMLElement>("#tab-panel-history");
  const list = qs<HTMLOListElement>("#history-list");

  let empty = panel.querySelector<HTMLElement>(".history-empty");
  if (!empty) {
    empty = document.createElement("div");
    empty.className = "history-empty muted";
    empty.textContent = "暂无历史记录";
    list.insertAdjacentElement("beforebegin", empty);
  }

  const entries = history.entries ?? [];
  qs<HTMLElement>("#history-count").textContent = String(entries.length);
  list.innerHTML = "";

  if (entries.length === 0) {
    empty.style.display = "block";
    list.style.display = "none";
    return;
  }

  empty.style.display = "none";
  list.style.display = "flex";

  for (let i = entries.length - 1; i >= 0; i--) {
    const entry = entries[i] ?? "";
    const li = document.createElement("li");
    li.className = "history-item";

    const text = escapeHtml(entry);
    li.innerHTML = `
      <div class="history-item-content" title="${text}">${text}</div>
      <div class="history-item-actions">
        <button class="btn-icon btn-copy" type="button" data-action="copy" data-index="${i}" title="复制内容">
          <iconify-icon icon="lucide:copy" width="16"></iconify-icon>
        </button>
        <button class="btn-icon btn-delete danger" type="button" data-action="delete" data-index="${i}" title="删除记录">
          <iconify-icon icon="lucide:trash-2" width="16"></iconify-icon>
        </button>
      </div>
    `;

    list.appendChild(li);
  }
}

async function loadHistory(opts: { silentOk?: boolean } = {}) {
  try {
    const history = await getHistory();
    renderHistory(history);
    if (!opts.silentOk) toast("已更新", "ok");
  } catch (e) {
    toast(`加载失败：${String(e)}`, "error");
  }
}

export function initHistoryTab() {
  qs<HTMLButtonElement>("#btn-history-reload").addEventListener("click", () => void loadHistory());

  qs<HTMLButtonElement>("#btn-history-clear").addEventListener("click", async () => {
    if (!confirm("确定清空全部历史记录？")) return;
    try {
      const h = await clearHistory();
      renderHistory(h);
      toast("已清空", "ok");
    } catch (e) {
      toast(`清空失败：${String(e)}`, "error");
    }
  });

  document.addEventListener("click", async (ev) => {
    const t = ev.target as HTMLElement | null;
    const btn = t?.closest<HTMLButtonElement>("button[data-action]");
    if (!btn) return;

    const action = btn.dataset.action;
    const index = Number(btn.dataset.index);
    if (!Number.isFinite(index)) return;

    if (action === "delete") {
      try {
        const h = await deleteHistoryRecord(index);
        renderHistory(h);
        toast("已删除", "ok");
      } catch (e) {
        toast(`删除失败：${String(e)}`, "error");
      }
    }

    if (action === "copy") {
      try {
        const history = await getHistory();
        const entry = history.entries?.[index] ?? "";
        await navigator.clipboard.writeText(entry);
        toast("已复制", "ok");
      } catch (e) {
        toast(`复制失败：${String(e)}`, "error");
      }
    }
  });

  return {
    refresh: () => loadHistory({ silentOk: true }),
  };
}
