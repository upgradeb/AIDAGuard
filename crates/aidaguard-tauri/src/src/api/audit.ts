import { invoke } from "@tauri-apps/api/core";
import type { AuditStats, DetectionRecord } from "../types";

export interface AuditListParams {
  limit: number;
  offset: number;
  ruleIdFilter?: string;
  pathFilter?: string;
  dateFromMs?: number;
  dateToMs?: number;
}

export interface AuditListResponse {
  records: DetectionRecord[];
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
