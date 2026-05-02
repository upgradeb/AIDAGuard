import { create } from "zustand";
import {
  listAudit,
  getAuditDetail,
  deleteAudit,
  exportAudit,
  getAuditStats,
} from "../api/audit";
import type { AuditListParams } from "../api/audit";
import type { AuditStats, DetectionRecord } from "../types";

interface AuditState {
  records: DetectionRecord[];
  total: number;
  page: number;
  pageSize: number;
  stats: AuditStats | null;
  selectedRecord: DetectionRecord | null;
  detailOpen: boolean;
  loading: boolean;
  statsLoading: boolean;
  error: string | null;

  fetchList: (params?: Partial<AuditListParams>) => Promise<void>;
  fetchDetail: (id: string) => Promise<void>;
  removeRecord: (id: string) => Promise<void>;
  doExport: (format: "csv" | "json", filters?: Partial<AuditListParams>) => Promise<string>;
  fetchStats: () => Promise<void>;
  setPage: (page: number, pageSize?: number) => void;
  closeDetail: () => void;
}

export const useAuditStore = create<AuditState>((set, get) => ({
  records: [],
  total: 0,
  page: 1,
  pageSize: 20,
  stats: null,
  selectedRecord: null,
  detailOpen: false,
  loading: false,
  statsLoading: false,
  error: null,

  fetchList: async (params) => {
    const { page, pageSize } = get();
    set({ loading: true, error: null });
    try {
      const res = await listAudit({
        limit: params?.limit ?? pageSize,
        offset: params?.offset ?? (page - 1) * pageSize,
        ruleIdFilter: params?.ruleIdFilter,
        pathFilter: params?.pathFilter,
        dateFromMs: params?.dateFromMs,
        dateToMs: params?.dateToMs,
      });
      set({ records: res.records, total: res.total, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchDetail: async (id) => {
    try {
      const record = await getAuditDetail(id);
      if (record) {
        set({ selectedRecord: record, detailOpen: true });
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  removeRecord: async (id) => {
    try {
      await deleteAudit(id);
      get().fetchList();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  doExport: async (format, filters) => {
    const path = await exportAudit({
      format,
      ruleIdFilter: filters?.ruleIdFilter,
      dateFromMs: filters?.dateFromMs,
      dateToMs: filters?.dateToMs,
    });
    return path;
  },

  fetchStats: async () => {
    set({ statsLoading: true });
    try {
      const stats = await getAuditStats();
      set({ stats, statsLoading: false });
    } catch (e) {
      set({ statsLoading: false });
    }
  },

  setPage: (page, pageSize) => {
    set({ page, ...(pageSize ? { pageSize } : {}) });
  },

  closeDetail: () => set({ detailOpen: false, selectedRecord: null }),
}));
