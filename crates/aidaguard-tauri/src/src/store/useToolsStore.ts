import { create } from "zustand";
import { detectTools, applyToolConfig, restoreToolConfig, restoreAllTools, enablePlugin, disablePlugin } from "../api/tools";
import type { ToolInfo } from "../types";

interface ToolsState {
  tools: ToolInfo[];
  loading: boolean;
  applying: string | null; // toolId being applied
  error: string | null;

  fetchTools: () => Promise<void>;
  apply: (toolId: string) => Promise<void>;
  restore: (toolId: string) => Promise<void>;
  restoreAll: () => Promise<void>;
  togglePlugin: (toolId: string) => Promise<void>;
}

export const useToolsStore = create<ToolsState>((set) => ({
  tools: [],
  loading: false,
  applying: null,
  error: null,

  fetchTools: async () => {
    set({ loading: true, error: null });
    try {
      const tools = await detectTools();
      set({ tools, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  apply: async (toolId) => {
    set({ applying: toolId, error: null });
    try {
      await applyToolConfig(toolId);
      // Refresh after apply
      const tools = await detectTools();
      set({ tools, applying: null });
    } catch (e) {
      set({ error: String(e), applying: null });
      throw e;
    }
  },

  restore: async (toolId) => {
    set({ applying: toolId, error: null });
    try {
      await restoreToolConfig(toolId);
      const tools = await detectTools();
      set({ tools, applying: null });
    } catch (e) {
      set({ error: String(e), applying: null });
      throw e;
    }
  },

  restoreAll: async () => {
    set({ loading: true, error: null });
    try {
      await restoreAllTools();
      const tools = await detectTools();
      set({ tools, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  togglePlugin: async (toolId) => {
    const tool = useToolsStore.getState().tools.find((t) => t.toolId === toolId);
    if (!tool) return;
    set({ error: null });
    try {
      if (tool.enabled) {
        await disablePlugin(toolId);
      } else {
        await enablePlugin(toolId);
      }
      const tools = await detectTools();
      set({ tools });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },
}));
