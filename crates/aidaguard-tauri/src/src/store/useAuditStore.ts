import { create } from "zustand";
import {
  listAudit,
  listAuditGroups,
  getAuditDetail,
  deleteAudit,
  exportAudit,
  getAuditStats,
  getRecentEvents,
} from "../api/audit";
import type { AuditListParams } from "../api/audit";
import type { AuditGroup, AuditStats, DetectionRecord } from "../types";

interface AuditState {
  groups: AuditGroup[];
  groupTotal: number;

  expandedRecords: Record<string, DetectionRecord[]>;
  expandedLoading: Record<string, boolean>;

  currentPathFilter?: string;
  currentDateFromMs?: number;
  currentDateToMs?: number;

  page: number;
  pageSize: number;

  stats: AuditStats | null;
  recentEvents: DetectionRecord[];
  selectedRecord: DetectionRecord | null;
  detailOpen: boolean;
  loading: boolean;
  statsLoading: boolean;
  error: string | null;

  fetchGroups: (params?: Partial<AuditListParams>) => Promise<void>;
  expandGroup: (ruleId: string, strategy: string) => Promise<void>;
  fetchDetail: (id: string) => Promise<void>;
  removeRecord: (id: string, ruleId?: string, strategy?: string) => Promise<void>;
  doExport: (format: "csv" | "json", filters?: Partial<AuditListParams>) => Promise<string>;
  fetchStats: () => Promise<void>;
  fetchRecentEvents: () => Promise<void>;
  setPage: (page: number, pageSize?: number) => void;
  closeDetail: () => void;
}

const groupKey = (ruleId: string, strategy: string) => `${ruleId}:${strategy}`;

export const useAuditStore = create<AuditState>((set, get) => ({
  groups: [],
  groupTotal: 0,
  expandedRecords: {},
  expandedLoading: {},
  currentPathFilter: undefined,
  currentDateFromMs: undefined,
  currentDateToMs: undefined,
  page: 1,
  pageSize: 20,
  stats: null,
  recentEvents: [],
  selectedRecord: null,
  detailOpen: false,
  loading: false,
  statsLoading: false,
  error: null,

  fetchGroups: async (params) => {
    const { page, pageSize } = get();
    set({ loading: true, error: null });
    try {
      const filters = {
        pathFilter: params?.pathFilter,
        dateFromMs: params?.dateFromMs,
        dateToMs: params?.dateToMs,
      };
      set({
        currentPathFilter: filters.pathFilter,
        currentDateFromMs: filters.dateFromMs,
        currentDateToMs: filters.dateToMs,
      });
      const res = await listAuditGroups({
        limit: params?.limit ?? pageSize,
        offset: params?.offset ?? (page - 1) * pageSize,
        ruleIdFilter: params?.ruleIdFilter,
        pathFilter: filters.pathFilter,
        dateFromMs: filters.dateFromMs,
        dateToMs: filters.dateToMs,
      });
      set({ groups: res.groups, groupTotal: res.total, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  expandGroup: async (ruleId, strategy) => {
    const key = groupKey(ruleId, strategy);
    const { expandedRecords, expandedLoading, currentPathFilter, currentDateFromMs, currentDateToMs } = get();
    if (expandedRecords[key]) return;

    set({ expandedLoading: { ...expandedLoading, [key]: true } });
    try {
      const res = await listAudit({
        limit: 10000,
        offset: 0,
        ruleIdFilter: ruleId,
        strategyFilter: strategy,
        pathFilter: currentPathFilter,
        dateFromMs: currentDateFromMs,
        dateToMs: currentDateToMs,
      });
      set({
        expandedRecords: { ...get().expandedRecords, [key]: res.records },
        expandedLoading: { ...get().expandedLoading, [key]: false },
      });
    } catch (e) {
      set({
        error: String(e),
        expandedLoading: { ...get().expandedLoading, [key]: false },
      });
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

  removeRecord: async (id, ruleId, strategy) => {
    try {
      await deleteAudit(id);
      await get().fetchGroups();
      if (ruleId && strategy) {
        const key = groupKey(ruleId, strategy);
        const { expandedRecords } = get();
        const updated = { ...expandedRecords };
        delete updated[key];
        set({ expandedRecords: updated });
        await get().expandGroup(ruleId, strategy);
      }
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

  fetchRecentEvents: async () => {
    try {
      const recentEvents = await getRecentEvents();
      set({ recentEvents });
    } catch (e) {
      console.error("获取最近事件失败:", e);
    }
  },

  setPage: (page, pageSize) => {
    set({ page, ...(pageSize ? { pageSize } : {}) });
  },

  closeDetail: () => set({ detailOpen: false, selectedRecord: null }),
}));
