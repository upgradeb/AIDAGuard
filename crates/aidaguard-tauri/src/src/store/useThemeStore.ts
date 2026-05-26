import { create } from "zustand";

type Theme = "light" | "dark" | "system";
export type ThemePreset = "emerald" | "indigo";

export interface PresetColors {
  colorPrimary: string;
  borderRadius: number;
}

export const PRESETS: Record<ThemePreset, PresetColors> = {
  emerald: { colorPrimary: "#10b981", borderRadius: 8 },
  indigo: { colorPrimary: "#6366f1", borderRadius: 8 },
};

interface ThemeState {
  theme: Theme;
  resolved: "light" | "dark";
  preset: ThemePreset;
  presetColors: PresetColors;
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

const storedTheme = (localStorage.getItem("aidaguard-theme") as Theme) || "light";
const storedPreset = (localStorage.getItem("aidaguard-preset") as ThemePreset) || "emerald";

export const useThemeStore = create<ThemeState>((set) => ({
  theme: storedTheme,
  resolved: resolveTheme(storedTheme),
  preset: storedPreset,
  presetColors: PRESETS[storedPreset],

  setTheme: (t: Theme) => {
    localStorage.setItem("aidaguard-theme", t);
    set({ theme: t, resolved: resolveTheme(t) });
  },

  setPreset: (p: ThemePreset) => {
    localStorage.setItem("aidaguard-preset", p);
    set({ preset: p, presetColors: PRESETS[p] });
  },
}));

// Listen for system theme changes
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    const current = useThemeStore.getState().theme;
    if (current === "system") {
      useThemeStore.setState({ resolved: resolveTheme("system") });
    }
  });
