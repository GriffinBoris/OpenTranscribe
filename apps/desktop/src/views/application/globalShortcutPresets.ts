const supportedKeys = new Set([
  "Space",
  ...Array.from({ length: 12 }, (_, index) => `F${index + 1}`),
]);

export const defaultGlobalShortcut = "CommandOrControl+Shift+R";

export function shortcutFromKeyboardEvent(event: KeyboardEvent) {
  const key = shortcutKey(event);

  if (!key || (!event.metaKey && !event.ctrlKey)) {
    return null;
  }

  const parts = ["CommandOrControl"];

  if (event.altKey) {
    parts.push("Alt");
  }

  if (event.shiftKey) {
    parts.push("Shift");
  }

  parts.push(key);
  return parts.join("+");
}

export function formatGlobalShortcut(shortcut: string) {
  const isMac = navigator.platform.toLowerCase().includes("mac");
  const labels: Record<string, string> = {
    CommandOrControl: isMac ? "⌘" : "Ctrl",
    Shift: isMac ? "⇧" : "Shift",
    Alt: isMac ? "⌥" : "Alt",
  };

  return shortcut
    .split("+")
    .map((part) => labels[part] ?? part)
    .join(" ");
}

function shortcutKey(event: KeyboardEvent) {
  if (/^Key[A-Z]$/.test(event.code)) {
    return event.code.slice(3);
  }

  if (/^Digit[0-9]$/.test(event.code)) {
    return event.code.slice(5);
  }

  if (event.code === "Space") {
    return "Space";
  }

  return supportedKeys.has(event.key) ? event.key : null;
}
