type ToastKind = "ok" | "error" | "";

function ensureContainer(): HTMLElement | null {
  let el = document.querySelector<HTMLElement>("#toast-container");
  if (el) return el;

  el = document.createElement("div");
  el.id = "toast-container";
  el.className = "toast-container";
  document.body.appendChild(el);
  return el;
}

export function toast(msg: string, kind: ToastKind = "") {
  const container = ensureContainer();
  if (!container) return;

  const t = document.createElement("div");
  t.className = `toast ${kind}`;
  t.textContent = msg;
  container.appendChild(t);

  setTimeout(() => t.remove(), 2500);
}
