import { create } from "zustand";
import { getConfig, saveConfig } from "../api/config";
import type { Config } from "../types";

interface ConfigState {
  config: Config | null;
  loading: boolean;
  saving: boolean;
  error: string | null;

  fetchConfig: () => Promise<void>;
  saveConfig: (config: Config) => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set) => ({
  config: null,
  loading: false,
  saving: false,
  error: null,

  fetchConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await getConfig();
      set({ config, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  saveConfig: async (config) => {
    set({ saving: true, error: null });
    try {
      await saveConfig(config);
      set({ config, saving: false });
    } catch (e) {
      set({ error: String(e), saving: false });
      throw e;
    }
  },
}));
