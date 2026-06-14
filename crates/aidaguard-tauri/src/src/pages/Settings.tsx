import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Save, Info } from "lucide-react";
import { toast } from "sonner";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Checkbox } from "@/components/ui/checkbox";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";

import { useConfigStore } from "../store/useConfigStore";
import { getAppVersion } from "../api/config";
import ThemeSwitcher from "../components/ThemeSwitcher";
import PresetSwitcher from "../components/PresetSwitcher";
import type { Config } from "../types";

// ── Constants ────────────────────────────────────────────────────────────────

const REGION_OPTIONS = [
  { value: "global", labelKey: "Global (All Regions)" },
  { value: "cn", labelKey: "China (PIPL)" },
  { value: "us", labelKey: "United States (CCPA/HIPAA)" },
  { value: "eu", labelKey: "European Union (GDPR)" },
  { value: "gb", labelKey: "United Kingdom (UK DPA)" },
];

const INDUSTRIES_BY_REGION: Record<string, string[]> = {
  global: [],
  cn: ["general", "finance", "medical", "personal"],
  us: ["general", "finance", "medical"],
  eu: ["general", "finance"],
  gb: ["general"],
};

const LOG_LEVELS = ["trace", "debug", "info", "warn", "error"] as const;

const NLP_LANGUAGES = [
  { value: "en", label: "English (bert-base-NER)" },
  { value: "zh", label: "中文 (bert-base-chinese-ner)" },
] as const;

// ── Zod schema ──────────────────────────────────────────────────────────────

const configSchema = z.object({
  api_key: z.string(),
  port: z.number().min(1024).max(65535),
  target_url: z.string(),
  rules_dir: z.string(),
  log_level: z.string(),
  max_body_size_mb: z.number().min(1).max(100),
  region: z.string(),
  rule_industries: z.array(z.string()),
  storage: z.object({
    enabled: z.boolean(),
    db_path: z.string(),
    encryption_key: z.string().optional(),
  }),
  upstreams: z.array(
    z.object({
      name: z.string(),
      url: z.string(),
      api_key: z.string().optional(),
      default: z.boolean(),
      timeout_secs: z.number(),
      rate_limit_qps: z.number(),
      models: z.array(z.string()),
      protocol: z.enum(["openai", "anthropic"]),
    }),
  ),
  notification: z.object({
    enabled: z.boolean(),
    rate_limit_secs: z.number().min(10).max(600),
  }),
  nlp: z.object({
    enabled: z.boolean(),
    default_language: z.string(),
    cache_dir: z.string().optional(),
  }),
});

type ConfigFormValues = z.infer<typeof configSchema>;

// ── Component ────────────────────────────────────────────────────────────────

export default function Settings() {
  const { t } = useTranslation();
  const config = useConfigStore((s) => s.config);
  const saving = useConfigStore((s) => s.saving);
  const fetchConfig = useConfigStore((s) => s.fetchConfig);
  const save = useConfigStore((s) => s.saveConfig);
  const [appVersion, setAppVersion] = useState("");
  const [region, setRegion] = useState<string>("global");

  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configSchema),
    defaultValues: {
      api_key: "",
      port: 19000,
      target_url: "",
      rules_dir: "",
      log_level: "info",
      max_body_size_mb: 10,
      region: "global",
      rule_industries: [],
      storage: { enabled: false, db_path: "", encryption_key: "" },
      upstreams: [],
      notification: { enabled: false, rate_limit_secs: 60 },
      nlp: { enabled: false, default_language: "en", cache_dir: "" },
    },
  });

  useEffect(() => {
    fetchConfig();
    getAppVersion().then(setAppVersion).catch(() => {});
  }, []);

  useEffect(() => {
    if (config) {
      form.reset(config as ConfigFormValues);
      setRegion(config.region || "global");
    }
  }, [config]);

  const handleSave = form.handleSubmit(async (values) => {
    const merged = { ...config, ...values } as Config;
    try {
      await save(merged);
      toast.success(t("Configuration Saved"));
    } catch (e) {
      toast.error(String(e));
    }
  });

  const handleRegionChange = (value: string) => {
    setRegion(value);
    form.setValue("region", value);
    form.setValue("rule_industries", []);
  };

  const industries = INDUSTRIES_BY_REGION[region] || [];

  return (
    <div className="h-full overflow-auto p-4">
      <form onSubmit={handleSave} className="space-y-4">
        {/* ── Proxy Settings ──────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t("Proxy Settings")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="port">{t("Listen Port")}</Label>
              <Controller
                name="port"
                control={form.control}
                render={({ field }) => (
                  <Input
                    id="port"
                    type="number"
                    min={1024}
                    max={65535}
                    className="w-[200px]"
                    {...field}
                    onChange={(e) => field.onChange(e.target.valueAsNumber || 0)}
                  />
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t("Default 19000. Restart proxy after changing.")}
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="rules_dir">{t("Rules Directory")}</Label>
              <Controller
                name="rules_dir"
                control={form.control}
                render={({ field }) => (
                  <Input id="rules_dir" placeholder="./rules" {...field} />
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t("Path to YAML rule files")}
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="max_body_size_mb">
                {t("Max Request Body (MB)")}
              </Label>
              <Controller
                name="max_body_size_mb"
                control={form.control}
                render={({ field }) => (
                  <Input
                    id="max_body_size_mb"
                    type="number"
                    min={1}
                    max={100}
                    className="w-[200px]"
                    {...field}
                    onChange={(e) => field.onChange(e.target.valueAsNumber || 0)}
                  />
                )}
              />
            </div>
          </CardContent>
        </Card>

        {/* ── Detection Policy ─────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t("Detection Policy")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>{t("Region / Country")}</Label>
              <Controller
                name="region"
                control={form.control}
                render={({ field }) => (
                  <Select
                    value={field.value}
                    onValueChange={(v) => {
                      field.onChange(v);
                      handleRegionChange(v);
                    }}
                  >
                    <SelectTrigger className="w-[280px]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {REGION_OPTIONS.map((opt) => (
                        <SelectItem key={opt.value} value={opt.value}>
                          {t(opt.labelKey)}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t(
                  "Select region or country for applicable detection rules",
                )}
              </p>
            </div>

            {region === "global" ? (
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription>
                  {t(
                    "Global baseline rules are always loaded regardless of region selection.",
                  )}
                </AlertDescription>
              </Alert>
            ) : (
              <div className="space-y-2">
                <Label>{t("Rule Industries")}</Label>
                <Controller
                  name="rule_industries"
                  control={form.control}
                  render={({ field }) => (
                    <div className="flex flex-wrap gap-4">
                      {industries.map((ind) => {
                        const checked = field.value?.includes(ind) ?? false;
                        return (
                          <label
                            key={ind}
                            className="flex items-center gap-2 text-sm cursor-pointer"
                          >
                            <Checkbox
                              checked={checked}
                              onCheckedChange={(checked) => {
                                const next = checked
                                  ? [...(field.value ?? []), ind]
                                  : (field.value ?? []).filter(
                                      (v: string) => v !== ind,
                                    );
                                field.onChange(next);
                              }}
                            />
                            {t(ind)}
                          </label>
                        );
                      })}
                    </div>
                  )}
                />
                <p className="text-xs text-muted-foreground">
                  {t(
                    "Select industries within the region for domain-specific rules",
                  )}
                </p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* ── NLP Settings ─────────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t("NLP Settings")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert>
              <Info className="h-4 w-4" />
              <AlertDescription>
                {t(
                  "NLP is disabled by default to reduce CPU usage. Enable only when you need to detect unstructured entities like names and addresses.",
                )}
              </AlertDescription>
            </Alert>

            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label>{t("NER Model")}</Label>
                <p className="text-xs text-muted-foreground">
                  {t(
                    "Enable NLP-based detection of unstructured entities (names, addresses, organizations). Increases CPU usage by ~40%.",
                  )}
                </p>
              </div>
              <Controller
                name="nlp.enabled"
                control={form.control}
                render={({ field }) => (
                  <Switch
                    checked={field.value}
                    onCheckedChange={field.onChange}
                  />
                )}
              />
            </div>

            <div className="space-y-2">
              <Label>{t("Model Language")}</Label>
              <Controller
                name="nlp.default_language"
                control={form.control}
                render={({ field }) => (
                  <Select
                    value={field.value}
                    onValueChange={field.onChange}
                  >
                    <SelectTrigger className="w-[180px]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {NLP_LANGUAGES.map((opt) => (
                        <SelectItem key={opt.value} value={opt.value}>
                          {opt.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t(
                  "Select the language for the NER model. The model will be downloaded on first use (~400MB).",
                )}
              </p>
            </div>
          </CardContent>
        </Card>

        {/* ── Storage Settings ─────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t("Storage Settings")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label>{t("Enable Audit Log")}</Label>
                <p className="text-xs text-muted-foreground">
                  {t(
                    "Sensitive data detection records will be persisted when enabled",
                  )}
                </p>
              </div>
              <Controller
                name="storage.enabled"
                control={form.control}
                render={({ field }) => (
                  <Switch
                    checked={field.value}
                    onCheckedChange={field.onChange}
                  />
                )}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="storage.db_path">{t("Database File Path")}</Label>
              <Controller
                name="storage.db_path"
                control={form.control}
                render={({ field }) => (
                  <Input
                    id="storage.db_path"
                    placeholder="./data/aidaguard.db"
                    {...field}
                  />
                )}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="storage.encryption_key">
                {t("Encryption Key")}
              </Label>
              <Controller
                name="storage.encryption_key"
                control={form.control}
                render={({ field }) => (
                  <Input
                    id="storage.encryption_key"
                    type="password"
                    placeholder={t(
                      "Leave empty to use built-in default key",
                    )}
                    {...field}
                  />
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t("Used to encrypt stored sensitive data content")}
              </p>
            </div>
          </CardContent>
        </Card>

        {/* ── Logging Settings ─────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t("Logging Settings")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>{t("Log Level")}</Label>
              <Controller
                name="log_level"
                control={form.control}
                render={({ field }) => (
                  <Select
                    value={field.value}
                    onValueChange={field.onChange}
                  >
                    <SelectTrigger className="w-[160px]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {LOG_LEVELS.map((lvl) => (
                        <SelectItem key={lvl} value={lvl}>
                          {lvl}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                )}
              />
            </div>
          </CardContent>
        </Card>

        {/* ── Notification Settings ────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t("Notification Settings")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label>{t("Desktop Notifications")}</Label>
                <p className="text-xs text-muted-foreground">
                  {t(
                    "Send system notification when sensitive data is detected. Takes effect after proxy restart.",
                  )}
                </p>
              </div>
              <Controller
                name="notification.enabled"
                control={form.control}
                render={({ field }) => (
                  <Switch
                    checked={field.value}
                    onCheckedChange={field.onChange}
                  />
                )}
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="notification.rate_limit_secs">
                {t("Notification Interval (s)")}
              </Label>
              <Controller
                name="notification.rate_limit_secs"
                control={form.control}
                render={({ field }) => (
                  <Input
                    id="notification.rate_limit_secs"
                    type="number"
                    min={10}
                    max={600}
                    className="w-[200px]"
                    {...field}
                    onChange={(e) => field.onChange(e.target.valueAsNumber || 0)}
                  />
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t(
                  "Minimum interval between notifications for the same rule to avoid spam",
                )}
              </p>
            </div>
          </CardContent>
        </Card>

        {/* ── Appearance ───────────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t("Appearance")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <span className="block text-xs text-muted-foreground mb-2">
                {t("Theme Preset")}
              </span>
              <PresetSwitcher />
            </div>
            <div>
              <span className="block text-xs text-muted-foreground mb-2">
                {t("Theme Mode")}
              </span>
              <ThemeSwitcher />
            </div>
          </CardContent>
        </Card>

        {/* ── About ────────────────────────────────────────────────────── */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t("About")}</CardTitle>
          </CardHeader>
          <CardContent>
            <dl className="grid grid-cols-[120px_1fr] gap-x-4 gap-y-2 text-sm">
              <dt className="text-muted-foreground">{t("Product")}</dt>
              <dd>Aidaguard</dd>
              <dt className="text-muted-foreground">{t("Version")}</dt>
              <dd>{appVersion || "—"}</dd>
              <dt className="text-muted-foreground">{t("License")}</dt>
              <dd>MIT</dd>
            </dl>
          </CardContent>
        </Card>

        {/* ── Save button ──────────────────────────────────────────────── */}
        <Button type="submit" size="lg" disabled={saving} className="mt-2">
          <Save className="mr-2 h-4 w-4" />
          {saving ? t("Saving...") : t("Save Settings")}
        </Button>
      </form>
    </div>
  );
}
