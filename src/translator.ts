import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

function qs<T extends HTMLElement>(sel: string): T {
  const el = document.querySelector(sel);
  if (!el) throw new Error(`Missing element: ${sel}`);
  return el as T;
}

const elSelectedText = qs<HTMLElement>("#selected-text");
const btnCopy = qs<HTMLButtonElement>("#btn-copy");

async function init() {
  btnCopy.addEventListener("click", async () => {
    const text = elSelectedText.textContent ?? "";
    if (!text.trim()) return;
    try {
      await writeText(text);
    } catch (e) {
      console.error("copy failed", e);
    }
  });

  await listen<{ text: string }>("selected-text", (event) => {
    console.log("selected-text received", event.payload?.text?.slice(0, 60));
    elSelectedText.textContent = event.payload?.text ?? "";
  });
}

void init();
