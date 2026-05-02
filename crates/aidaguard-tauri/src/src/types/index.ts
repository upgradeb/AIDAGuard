export interface ProxyStatus {
  status: "stopped" | "running" | "error";
  port: number;
  uptimeSecs: number;
  rulesCount: number;
  storageEnabled: boolean;
  errorMessage?: string;
}

export interface AuditStats {
  totalCount: number;
  todayCount: number;
  weekCount: number;
  ruleDistribution: { ruleId: string; ruleName: string; count: number }[];
  dbSizeBytes: number;
}

export interface DetectionRecord {
  id: string;
  timestampMs: number;
  ruleId: string;
  strategy: string;
  placeholder: string;
  original: string;
  context: string;
  requestPath: string;
  sanitizedBody: string;
  responseStatus: number;
  toolName: string;
}

export interface RuleDef {
  id: string;
  name: string;
  pattern: string;
  enabled: boolean;
  strategy: "placeholder" | "mask";
  priority: number;
  category?: string;
}

export interface MatchInfo {
  ruleId: string;
  start: number;
  end: number;
  text: string;
  priority: number;
  strategy: "placeholder" | "mask";
}

export interface TestRuleResult {
  matches: MatchInfo[];
  sanitizedText: string;
}

export interface StorageConfig {
  enabled: boolean;
  db_path: string;
  encryption_key?: string;
}

export interface Config {
  api_key: string;
  port: number;
  target_url: string;
  rules_dir: string;
  log_level: string;
  max_body_size_mb: number;
  storage: StorageConfig;
  upstreams: UpstreamConfig[];
}

export interface UpstreamConfig {
  name: string;
  url: string;
  api_key?: string;
  default: boolean;
  timeout_secs: number;
  rate_limit_qps: number;
  models: string[];
}

export interface DetectionEvent {
  timestampMs: number;
  ruleId: string;
  strategy: string;
  placeholder: string;
  requestPath: string;
  responseStatus: number;
  toolName: string;
}
