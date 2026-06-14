import { create } from "zustand";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Theme = "light" | "dark" | "system";
export type ThemePreset = "emerald" | "indigo";

interface ThemeState {
  theme: Theme;
  resolved: "light" | "dark";
  preset: ThemePreset;
  setTheme: (t: Theme) => void;
  setPreset: (p: ThemePreset) => void;
}

function resolveTheme(t: Theme): "light" | "dark" {
  if (t === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return t;
}

function applyThemeToDOM(resolved: "light" | "dark", preset: ThemePreset) {
  const root = document.documentElement;
  root.classList.toggle("dark", resolved === "dark");
  root.classList.toggle("preset-indigo", preset === "indigo");
  try {
    getCurrentWindow().setTheme?.(resolved);
  } catch {
    /* non-Tauri env */
  }
}

const storedTheme = (localStorage.getItem("aidaguard-theme") as Theme) || "light";
const storedPreset = (localStorage.getItem("aidaguard-preset") as ThemePreset) || "emerald";

// Apply theme on load
applyThemeToDOM(resolveTheme(storedTheme), storedPreset);

export const useThemeStore = create<ThemeState>((set) => ({
  theme: storedTheme,
  resolved: resolveTheme(storedTheme),
  preset: storedPreset,

  setTheme: (t: Theme) => {
    localStorage.setItem("aidaguard-theme", t);
    const resolved = resolveTheme(t);
    applyThemeToDOM(resolved, useThemeStore.getState().preset);
    set({ theme: t, resolved });
  },

  setPreset: (p: ThemePreset) => {
    localStorage.setItem("aidaguard-preset", p);
    applyThemeToDOM(useThemeStore.getState().resolved, p);
    set({ preset: p });
  },
}));

// Listen for system theme changes
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    const current = useThemeStore.getState().theme;
    if (current === "system") {
      const resolved = resolveTheme("system");
      applyThemeToDOM(resolved, useThemeStore.getState().preset);
      useThemeStore.setState({ resolved });
    }
  });
