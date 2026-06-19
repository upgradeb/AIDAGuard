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
  ruleName: string;
  strategy: string;
  placeholder: string;
  original: string;
  context: string;
  requestPath: string;
  sanitizedBody: string;
  responseStatus: number;
  toolName: string;
}

export interface AuditGroup {
  ruleId: string;
  ruleName: string;
  strategy: string;
  count: number;
  latestTimestampMs: number;
}

export interface RuleDef {
  id: string;
  name: string;
  pattern: string;
  exclude?: string;
  enabled: boolean;
  strategy: "placeholder" | "mask";
  mode: "detect" | "filter";
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
  mode: "detect" | "filter";
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

export interface NotificationConfig {
  enabled: boolean;
  rate_limit_secs: number;
}

export interface DetectionRegion {
  primary_region: string;
  additional_regions: string[];
}

export interface RegionInfo {
  code: string;
  name: string;
}

export interface NlpConfig {
  enabled: boolean;
  default_language: string;
  cache_dir?: string;
}

export interface Config {
  api_key: string;
  port: number;
  target_url: string;
  rules_dir: string;
  log_level: string;
  max_body_size_mb: number;
  /** @deprecated Use detection_region instead */
  region: string;
  /** @deprecated No longer used with flat rule structure */
  rule_industries: string[];
  detection_region: DetectionRegion;
  storage: StorageConfig;
  upstreams: UpstreamConfig[];
  notification: NotificationConfig;
  nlp: NlpConfig;
}

export interface UpstreamConfig {
  name: string;
  url: string;
  api_key?: string;
  default: boolean;
  timeout_secs: number;
  rate_limit_qps: number;
  models: string[];
  protocol: "openai" | "anthropic";
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

export interface ToolInfo {
  toolId: string;
  toolName: string;
  installed: boolean;
  configured: boolean;
  configPath: string;
  currentEndpoint?: string;
  currentModel?: string;
  previewEndpoint?: string;
  version: string;
  description: string;
  author: string;
  categories: string[];
  enabled: boolean;
}
