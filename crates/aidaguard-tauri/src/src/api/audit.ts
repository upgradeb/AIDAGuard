import { invoke } from "@tauri-apps/api/core";
import type { AuditGroup, AuditStats, DetectionRecord } from "../types";

export interface AuditListParams {
  limit: number;
  offset: number;
  ruleIdFilter?: string;
  pathFilter?: string;
  dateFromMs?: number;
  dateToMs?: number;
  strategyFilter?: string;
}

export interface AuditListResponse {
  records: DetectionRecord[];
  total: number;
}

export interface AuditGroupResponse {
  groups: AuditGroup[];
  total: number;
}

export const listAudit = (params: AuditListParams): Promise<AuditListResponse> =>
  invoke("list_audit", {
    limit: params.limit,
    offset: params.offset,
    ruleIdFilter: params.ruleIdFilter ?? null,
    pathFilter: params.pathFilter ?? null,
    dateFromMs: params.dateFromMs ?? null,
    dateToMs: params.dateToMs ?? null,
    strategyFilter: params.strategyFilter ?? null,
  });

export const listAuditGroups = (params: {
  limit: number;
  offset: number;
  ruleIdFilter?: string;
  pathFilter?: string;
  dateFromMs?: number;
  dateToMs?: number;
}): Promise<AuditGroupResponse> =>
  invoke("list_audit_groups", {
    limit: params.limit,
    offset: params.offset,
    ruleIdFilter: params.ruleIdFilter ?? null,
    pathFilter: params.pathFilter ?? null,
    dateFromMs: params.dateFromMs ?? null,
    dateToMs: params.dateToMs ?? null,
  });

export const getAuditDetail = (
  recordId: string
): Promise<DetectionRecord | null> =>
  invoke("get_audit_detail", { recordId });

export const deleteAudit = (recordId: string): Promise<boolean> =>
  invoke("delete_audit", { recordId });

export const exportAudit = (params: {
  format: "csv" | "json";
  ruleIdFilter?: string;
  dateFromMs?: number;
  dateToMs?: number;
}): Promise<string> =>
  invoke("export_audit", {
    format: params.format,
    ruleIdFilter: params.ruleIdFilter ?? null,
    dateFromMs: params.dateFromMs ?? null,
    dateToMs: params.dateToMs ?? null,
  });

export const getAuditStats = (): Promise<AuditStats> =>
  invoke("get_audit_stats");

export const getRecentEvents = (): Promise<DetectionRecord[]> =>
  invoke("get_recent_events");
