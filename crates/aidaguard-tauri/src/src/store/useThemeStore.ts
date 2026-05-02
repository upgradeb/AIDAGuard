import { create } from "zustand";

type Theme = "light" | "dark" | "system";

interface ThemeState {
  theme: Theme;
  resolved: "light" | "dark";
  setTheme: (t: Theme) => void;
}

function resolveTheme(t: Theme): "light" | "dark" {
  if (t === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return t;
}

const stored = (localStorage.getItem("aidaguard-theme") as Theme) || "light";

export const useThemeStore = create<ThemeState>((set) => ({
  theme: stored,
  resolved: resolveTheme(stored),

  setTheme: (t: Theme) => {
    localStorage.setItem("aidaguard-theme", t);
    set({ theme: t, resolved: resolveTheme(t) });
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
