import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import {
  CirclePlay,
  CirclePause,
  RefreshCw,
  Zap,
  Clock,
  Database,
  Server,
  Shield,
  Globe,
  CircleAlert,
  X,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useProxyStore } from "../store/useProxyStore";
import { useAuditStore } from "../store/useAuditStore";
import { useUpstreamStore } from "../store/useUpstreamStore";
import StatCard from "../components/StatCard";
import EventFeed from "../components/EventFeed";
import RuleHitChart from "../components/RuleHitChart";
import OperationGuide from "../components/OperationGuide";

export default function Dashboard() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const status = useProxyStore((s) => s.status);
  const loading = useProxyStore((s) => s.loading);
  const error = useProxyStore((s) => s.error);
  const recentRecords = useAuditStore((s) => s.recentEvents);
  const fetchRecentEvents = useAuditStore((s) => s.fetchRecentEvents);
  const start = useProxyStore((s) => s.startProxy);
  const stop = useProxyStore((s) => s.stopProxy);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const stats = useAuditStore((s) => s.stats);
  const fetchStats = useAuditStore((s) => s.fetchStats);
  const upstreams = useUpstreamStore((s) => s.upstreams);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);
  const setDefaultUpstream = useUpstreamStore((s) => s.setDefaultUpstream);

  useEffect(() => {
    fetchStatus();
    fetchStats();
    fetchUpstreams();
    fetchRecentEvents();
    const interval = setInterval(() => {
      fetchStatus();
      fetchStats();
      fetchRecentEvents();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const isRunning = status?.status === "running";
  const defaultUpstream = upstreams.find((u) => u.default);
  const proxyUrl = `http://127.0.0.1:${status?.port || 19000}`;

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatUptime = (secs: number) => {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  };

  return (
    <div className="h-full overflow-auto">
      {error && (
        <Alert variant="destructive" className="mb-4 rounded-lg">
          <CircleAlert className="h-4 w-4" />
          <AlertTitle>{t("Proxy Operation Failed")}</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
          <button
            className="absolute right-3 top-3 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100"
            onClick={() => useProxyStore.setState({ error: null })}
          >
            <X className="h-4 w-4" />
          </button>
        </Alert>
      )}

      {/* Operation Guide */}
      <OperationGuide />

      {/* Proxy Info Card */}
      <Card className="mb-6 rounded-xl">
        <CardContent className="p-4">
          {/* Row 1: Status + Address + Action Buttons */}
          <div className="flex items-center justify-between flex-wrap gap-3">
            <div className="flex items-center gap-4 flex-wrap">
              <div className="flex items-center gap-2">
                <span
                  className="inline-block h-2.5 w-2.5 rounded-full"
                  style={{ background: isRunning ? "#22c55e" : "#9ca3af" }}
                />
                <span className="text-[15px] font-semibold">
                  {isRunning ? t("Proxy Running") : t("Proxy Stopped")}
                </span>
              </div>

              <Badge
                variant={isRunning ? "default" : "secondary"}
                className="gap-1.5"
              >
                <Globe className="h-3 w-3" />
                {proxyUrl}
              </Badge>

              {isRunning && (
                <>
                  <Badge variant="secondary" className="gap-1.5">
                    <Clock className="h-3 w-3" />
                    {t("Up {{uptime}}", { uptime: formatUptime(status?.uptimeSecs || 0) })}
                  </Badge>
                  <Badge variant="secondary" className="gap-1.5">
                    <Shield className="h-3 w-3" />
                    {t("{{count}} Rules", { count: status?.rulesCount ?? 0 })}
                  </Badge>
                  <Badge variant="secondary" className="gap-1.5">
                    <Database className="h-3 w-3" />
                    {t("Storage")} {status?.storageEnabled ? t("On") : t("Off")}
                  </Badge>
                </>
              )}
            </div>

            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => { fetchStatus(); fetchStats(); }}
              >
                <RefreshCw className="h-4 w-4" />
              </Button>
              {isRunning ? (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={stop}
                  disabled={loading}
                >
                  <CirclePause className="h-4 w-4" />
                  {t("Stop")}
                </Button>
              ) : (
                <Button
                  size="sm"
                  onClick={start}
                  disabled={loading}
                >
                  <CirclePlay className="h-4 w-4" />
                  {t("Start Proxy")}
                </Button>
              )}
            </div>
          </div>

          {/* Row 2: Upstream Model Selector */}
          <div className="mt-4 pt-4 border-t border-border flex items-center gap-3 flex-wrap">
            <div className="flex items-center gap-2">
              <Server className="h-4 w-4 text-primary" />
              <span className="text-[13px]">{t("Upstream Model")}</span>
            </div>
            <Select
              value={defaultUpstream?.name || undefined}
              onValueChange={async (name) => {
                await setDefaultUpstream(name);
                fetchUpstreams();
              }}
            >
              <SelectTrigger className="w-[280px]">
                <SelectValue placeholder={t("Select Default Upstream LLM")} />
              </SelectTrigger>
              <SelectContent>
                {upstreams.length === 0 ? (
                  <div className="p-2 text-sm text-muted-foreground">
                    {t("No upstream configured. Go to LLM Upstreams to add one.")}
                  </div>
                ) : (
                  upstreams.map((u) => (
                    <SelectItem key={u.name} value={u.name}>
                      {u.name} — {u.url}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
            {defaultUpstream && (
              <Badge variant="secondary" className="text-xs">
                {defaultUpstream.url}
                {defaultUpstream.models.length > 0 &&
                  ` (${defaultUpstream.models.join(", ")})`}
              </Badge>
            )}
            <span className="text-[11px] text-muted-foreground">
              {isRunning ? t("Restart proxy after switching to apply") : t("Select target model before starting proxy")}
            </span>
          </div>
        </CardContent>
      </Card>

      {/* Stat Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <StatCard
          title={t("Today")}
          value={stats?.todayCount ?? 0}
          icon={<Zap />}
          color="#3b82f6"
        />
        <StatCard
          title={t("This Week")}
          value={stats?.weekCount ?? 0}
          icon={<Zap />}
          color="#8b5cf6"
        />
        <StatCard
          title={t("Total")}
          value={stats?.totalCount ?? 0}
          icon={<Zap />}
          color="#f59e0b"
        />
        <StatCard
          title={t("DB Size")}
          value={formatBytes(stats?.dbSizeBytes ?? 0)}
          icon={<Database />}
          color="#22c55e"
        />
      </div>

      {/* Chart + Live Events */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card className="rounded-xl">
          <CardHeader className="p-4 pb-2">
            <CardTitle className="text-sm font-medium">{t("Rule Hit Distribution")}</CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-0">
            <RuleHitChart data={stats?.ruleDistribution ?? []} />
          </CardContent>
        </Card>
        <Card className="rounded-xl max-h-[380px] overflow-auto">
          <CardHeader className="p-4 pb-2">
            <CardTitle className="text-sm font-medium">{t("Recent Events")}</CardTitle>
          </CardHeader>
          <CardContent className="p-4 pt-0">
            <EventFeed
              records={recentRecords}
              onClickRecord={(id) => navigate("/audit", { state: { openRecordId: id } })}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
