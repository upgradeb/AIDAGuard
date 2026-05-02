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
    rule_id_filter: params.ruleIdFilter ?? null,
    path_filter: params.pathFilter ?? null,
    date_from_ms: params.dateFromMs ?? null,
    date_to_ms: params.dateToMs ?? null,
  });

export const getAuditDetail = (
  recordId: string
): Promise<DetectionRecord | null> =>
  invoke("get_audit_detail", { record_id: recordId });

export const deleteAudit = (recordId: string): Promise<boolean> =>
  invoke("delete_audit", { record_id: recordId });

export const exportAudit = (params: {
  format: "csv" | "json";
  ruleIdFilter?: string;
  dateFromMs?: number;
  dateToMs?: number;
}): Promise<string> =>
  invoke("export_audit", {
    format: params.format,
    rule_id_filter: params.ruleIdFilter ?? null,
    date_from_ms: params.dateFromMs ?? null,
    date_to_ms: params.dateToMs ?? null,
  });

export const getAuditStats = (): Promise<AuditStats> =>
  invoke("get_audit_stats");
