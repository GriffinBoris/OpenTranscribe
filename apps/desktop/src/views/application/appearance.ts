import type { AppSettings } from "@/types/domain";

export function applyAppearance(settings: AppSettings) {
  applyTheme(settings.appearance.theme);
  applyReducedMotion(settings.appearance.reduced_motion);
}

export function applyTheme(theme: AppSettings["appearance"]["theme"]) {
  document.documentElement.dataset.theme = theme;
}

export function applyReducedMotion(value: boolean) {
  document.documentElement.dataset.reducedMotion = String(value);
}
