import { create } from "zustand";
import {
  getUpstreams,
  addUpstream,
  updateUpstream,
  deleteUpstream,
  setDefaultUpstream,
  testConnectivity,
} from "../api/upstream";
import type { UpstreamConfig } from "../types";

interface UpstreamState {
  upstreams: UpstreamConfig[];
  loading: boolean;
  saving: boolean;
  testing: string | null;
  testResult: string | null;
  error: string | null;

  fetchUpstreams: () => Promise<void>;
  addUpstream: (upstream: UpstreamConfig) => Promise<void>;
  updateUpstream: (name: string, upstream: UpstreamConfig) => Promise<void>;
  deleteUpstream: (name: string) => Promise<void>;
  setDefaultUpstream: (name: string) => Promise<void>;
  testConnectivity: (name: string, url: string, apiKey: string, timeoutSecs: number) => Promise<void>;
  clearTestResult: () => void;
}

export const useUpstreamStore = create<UpstreamState>((set) => ({
  upstreams: [],
  loading: false,
  saving: false,
  testing: null,
  testResult: null,
  error: null,

  fetchUpstreams: async () => {
    set({ loading: true, error: null });
    try {
      const upstreams = await getUpstreams();
      set({ upstreams, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addUpstream: async (upstream) => {
    set({ saving: true, error: null });
    try {
      await addUpstream(upstream);
      set({ saving: false });
    } catch (e) {
      set({ error: String(e), saving: false });
      throw e;
    }
  },

  updateUpstream: async (name, upstream) => {
    set({ saving: true, error: null });
    try {
      await updateUpstream(name, upstream);
      set({ saving: false });
    } catch (e) {
      set({ error: String(e), saving: false });
      throw e;
    }
  },

  deleteUpstream: async (name) => {
    set({ error: null });
    try {
      await deleteUpstream(name);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  setDefaultUpstream: async (name) => {
    set({ error: null });
    try {
      await setDefaultUpstream(name);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  testConnectivity: async (name, url, apiKey, timeoutSecs) => {
    set({ testing: name, testResult: null, error: null });
    try {
      const result = await testConnectivity(url, apiKey, timeoutSecs);
      set({ testResult: result, testing: null });
    } catch (e) {
      set({ testResult: String(e), testing: null });
    }
  },

  clearTestResult: () => set({ testResult: null }),
}));
