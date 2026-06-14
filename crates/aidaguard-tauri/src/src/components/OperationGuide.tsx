import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import {
  Network,
  Cloud,
  Wrench,
  Shield,
  Zap,
  CheckCircle2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useProxyStore } from "../store/useProxyStore";
import { useUpstreamStore } from "../store/useUpstreamStore";
import { useToolsStore } from "../store/useToolsStore";
import { useRulesStore } from "../store/useRulesStore";

const stepIcons = [Network, Cloud, Wrench, Shield, Zap];

export default function OperationGuide() {
  const navigate = useNavigate();
  const { t } = useTranslation();

  const proxyStatus = useProxyStore((s) => s.status);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const upstreams = useUpstreamStore((s) => s.upstreams);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);
  const tools = useToolsStore((s) => s.tools);
  const fetchTools = useToolsStore((s) => s.fetchTools);
  const rules = useRulesStore((s) => s.rules);
  const fetchRules = useRulesStore((s) => s.fetchRules);

  useEffect(() => {
    fetchStatus();
    fetchUpstreams();
    fetchTools();
    fetchRules();
  }, []);

  const isRunning = proxyStatus?.status === "running";
  const hasUpstream = upstreams.length > 0;
  const hasConfiguredTool = tools.filter((t) => t.configured).length > 0;
  const hasEnabledRule = rules.filter((r) => r.enabled).length > 0;
  const proxyPort = proxyStatus?.port || 19000;

  const steps = [
    {
      title: t("Confirm Proxy Port"),
      done: true,
      path: "/",
      desc: t("Port {{port}} Ready", { port: proxyPort }),
    },
    {
      title: t("Configure LLM Endpoint"),
      done: hasUpstream,
      path: "/upstreams",
      desc: hasUpstream
        ? t("{{count}} Upstreams Configured", { count: upstreams.length })
        : t("Go to LLM Upstreams"),
    },
    {
      title: t("Configure AI Tool Proxy"),
      done: hasConfiguredTool,
      path: "/tools",
      desc: hasConfiguredTool
        ? t("{{count}} Tools Configured", { count: tools.filter((t) => t.configured).length })
        : t("Go to AI Tools Config"),
    },
    {
      title: t("Enable Detection Rules"),
      done: hasEnabledRule,
      path: "/rules",
      desc: hasEnabledRule
        ? t("{{count}} Rules Enabled", { count: rules.filter((r) => r.enabled).length })
        : t("Go to Rules to Enable"),
    },
    {
      title: t("Start Proxy Service"),
      done: isRunning,
      path: "/",
      desc: isRunning ? t("Proxy Running") : t("Click \"Start Proxy\" on Dashboard"),
    },
  ];

  const currentStep = steps.findIndex((s) => !s.done);

  return (
    <Card className="rounded-xl mb-6">
      <CardHeader className="pb-3">
        <CardTitle className="text-sm flex items-center gap-2">
          <Zap className="h-4 w-4 text-preset" />
          {t("Getting Started")}
        </CardTitle>
      </CardHeader>
      <CardContent className="px-6 pb-4">
        <div className="flex items-start justify-between gap-2">
          {steps.map((step, i) => {
            const Icon = stepIcons[i];
            return (
              <div key={i} className="flex flex-col items-center flex-1">
                <div
                  className={cn(
                    "w-8 h-8 rounded-full flex items-center justify-center text-sm shrink-0",
                    step.done
                      ? "bg-green-500 text-white"
                      : i === currentStep
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted text-muted-foreground"
                  )}
                >
                  {step.done ? (
                    <CheckCircle2 className="h-4 w-4" />
                  ) : (
                    <Icon className="h-4 w-4" />
                  )}
                </div>
                <span className="mt-1 text-xs text-center font-medium leading-tight">
                  {step.title}
                </span>
                <span
                  className={cn(
                    "text-[11px] text-center leading-tight",
                    step.done ? "text-green-600" : "text-muted-foreground"
                  )}
                >
                  {step.desc}
                </span>
                {!step.done && (
                  <Button
                    variant="link"
                    className="h-auto p-0 mt-1 text-xs"
                    onClick={() => navigate(step.path)}
                  >
                    {t("Configure →")}
                  </Button>
                )}
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
