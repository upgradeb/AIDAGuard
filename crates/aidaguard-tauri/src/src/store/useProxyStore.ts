import { create } from "zustand";
import { startProxy, stopProxy, getProxyStatus } from "../api/proxy";
import { onDetectionEvent, onProxyStatusChanged } from "../api/events";
import type { ProxyStatus, DetectionEvent } from "../types";

interface ProxyState {
  status: ProxyStatus | null;
  loading: boolean;
  error: string | null;
  recentEvents: DetectionEvent[];

  fetchStatus: () => Promise<void>;
  startProxy: () => Promise<void>;
  stopProxy: () => Promise<void>;
  startListening: () => () => void;
}

export const useProxyStore = create<ProxyState>((set) => ({
  status: null,
  loading: false,
  error: null,
  recentEvents: [],

  fetchStatus: async () => {
    try {
      const status = await getProxyStatus();
      set({ status, error: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startProxy: async () => {
    set({ loading: true, error: null });
    try {
      await startProxy();
      const status = await getProxyStatus();
      set({ status, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  stopProxy: async () => {
    set({ loading: true, error: null });
    try {
      await stopProxy();
      const status = await getProxyStatus();
      set({ status, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  startListening: () => {
    const unsubDetection = onDetectionEvent((event) => {
      set((s) => ({
        recentEvents: [event, ...s.recentEvents].slice(0, 50),
      }));
    });

    const unsubStatus = onProxyStatusChanged((status) => {
      set((s) => ({
        status: s.status ? { ...s.status, status: status.status } : null,
      }));
    });

    // Resolve unlisten promises
    let unlisteners: (() => void)[] = [];
    unsubDetection.then((un) => unlisteners.push(un));
    unsubStatus.then((un) => unlisteners.push(un));

    return () => {
      unlisteners.forEach((un) => un());
    };
  },
}));
