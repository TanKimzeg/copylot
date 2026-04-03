export type TabKey = string;

export function setupTabs(opts: {
  tabSelector: string;
  panelSelector: string;
  activeClass: string;
  onActivate?: (key: TabKey) => void;
}) {
  const tabs = Array.from(document.querySelectorAll<HTMLElement>(opts.tabSelector));
  const panels = Array.from(document.querySelectorAll<HTMLElement>(opts.panelSelector));

  const activate = (key: TabKey) => {
    for (const t of tabs) {
      const isActive = (t as HTMLElement).id === key;
      t.classList.toggle(opts.activeClass, isActive);
      t.setAttribute("aria-selected", isActive ? "true" : "false");
    }
    for (const p of panels) {
      const target = p.getAttribute("aria-labelledby");
      const isActive = target === key;
      p.classList.toggle(opts.activeClass, isActive);
    }

    opts.onActivate?.(key);
  };

  for (const t of tabs) {
    t.addEventListener("click", () => activate(t.id));
  }

  // default: first tab
  if (tabs.length > 0) activate(tabs[0]!.id);

  return { activate };
}

export function getTabIds() {
  return Array.from(document.querySelectorAll<HTMLElement>("[role=tab]"), (t) => t.id);
}
