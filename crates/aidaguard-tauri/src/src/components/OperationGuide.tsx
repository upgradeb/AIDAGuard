import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Card, Steps, Typography, theme, Button } from "antd";
import {
  ApiOutlined,
  CloudServerOutlined,
  ToolOutlined,
  SafetyOutlined,
  ThunderboltOutlined,
  CheckCircleFilled,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useProxyStore } from "../store/useProxyStore";
import { useUpstreamStore } from "../store/useUpstreamStore";
import { useToolsStore } from "../store/useToolsStore";
import { useRulesStore } from "../store/useRulesStore";

export default function OperationGuide() {
  const { token } = theme.useToken();
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
      icon: <ApiOutlined />,
      path: "/",
      desc: t("Port {{port}} Ready", { port: proxyPort }),
    },
    {
      title: t("Configure LLM Endpoint"),
      done: hasUpstream,
      icon: <CloudServerOutlined />,
      path: "/upstreams",
      desc: hasUpstream
        ? t("{{count}} Upstreams Configured", { count: upstreams.length })
        : t("Go to LLM Upstreams"),
    },
    {
      title: t("Configure AI Tool Proxy"),
      done: hasConfiguredTool,
      icon: <ToolOutlined />,
      path: "/tools",
      desc: hasConfiguredTool
        ? t("{{count}} Tools Configured", { count: tools.filter((t) => t.configured).length })
        : t("Go to AI Tools Config"),
    },
    {
      title: t("Enable Detection Rules"),
      done: hasEnabledRule,
      icon: <SafetyOutlined />,
      path: "/rules",
      desc: hasEnabledRule
        ? t("{{count}} Rules Enabled", { count: rules.filter((r) => r.enabled).length })
        : t("Go to Rules to Enable"),
    },
    {
      title: t("Start Proxy Service"),
      done: isRunning,
      icon: <ThunderboltOutlined />,
      path: "/",
      desc: isRunning ? t("Proxy Running") : t("Click \"Start Proxy\" on Dashboard"),
    },
  ];

  const currentStep = steps.findIndex((s) => !s.done);

  return (
    <Card
      size="small"
      title={
        <span style={{ fontSize: 14 }}>
          <ThunderboltOutlined style={{ marginRight: 8, color: token.colorPrimary }} />
          {t("Getting Started")}
        </span>
      }
      style={{ borderRadius: 12, border: `1px solid ${token.colorBorderSecondary}`, marginBottom: 24 }}
      styles={{ body: { padding: "16px 24px" } }}
    >
      <Steps
        direction="horizontal"
        size="small"
        current={currentStep === -1 ? steps.length : currentStep}
        labelPlacement="vertical"
        responsive
        items={steps.map((step, i) => ({
          title: step.title,
          status: step.done ? "finish" : i === currentStep ? "process" : "wait",
          icon: step.done ? <CheckCircleFilled style={{ color: token.colorSuccess }} /> : step.icon,
          description: (
            <div style={{ fontSize: 12 }}>
              <Typography.Text
                type={step.done ? "success" : "secondary"}
                style={{ fontSize: 12 }}
              >
                {step.desc}
              </Typography.Text>
              {!step.done && (
                <div style={{ marginTop: 4 }}>
                  <Button
                    size="small"
                    type="link"
                    style={{ padding: 0, fontSize: 12, height: "auto" }}
                    onClick={() => navigate(step.path)}
                  >
                    {t("Configure →")}
                  </Button>
                </div>
              )}
            </div>
          ),
        }))}
      />
    </Card>
  );
}
