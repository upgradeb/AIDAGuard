import { Routes, Route } from "react-router-dom";
import { Layout, theme, Button } from "antd";
import {
  DashboardOutlined,
  AuditOutlined,
  SafetyOutlined,
  ApiOutlined,
  SettingOutlined,
  ToolOutlined,
} from "@ant-design/icons";
import { useNavigate, useLocation } from "react-router-dom";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Dashboard from "./pages/Dashboard";
import AuditLog from "./pages/AuditLog";
import Rules from "./pages/Rules";
import Settings from "./pages/Settings";
import Upstreams from "./pages/Upstreams";
import ToolsConfig from "./pages/ToolsConfig";
import { useProxyStore } from "./store/useProxyStore";
import { useNotification } from "./hooks/useNotification";

const { Sider, Header, Content } = Layout;

function useMenuItems() {
  const { t } = useTranslation();
  return [
    { key: "/", icon: <DashboardOutlined />, label: t("仪表盘") },
    { key: "/audit", icon: <AuditOutlined />, label: t("审计记录") },
    { key: "/upstreams", icon: <ApiOutlined />, label: t("大模型接入") },
    { key: "/tools", icon: <ToolOutlined />, label: t("AI 工具配置") },
    { key: "/rules", icon: <SafetyOutlined />, label: t("规则管理") },
    { key: "/settings", icon: <SettingOutlined />, label: t("设置") },
  ];
}

export default function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const status = useProxyStore((s) => s.status);
  const { token } = theme.useToken();
  const startListening = useProxyStore((s) => s.startListening);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const { t, i18n } = useTranslation();
  const menuItems = useMenuItems();

  useEffect(() => {
    fetchStatus();
    const cleanup = startListening();
    return cleanup;
  }, []);

  useEffect(() => {
    document.body.style.backgroundColor = token.colorBgLayout;
    document.body.style.color = token.colorText;
    document.body.style.transition = "background-color 0.2s, color 0.2s";
    document.documentElement.style.colorScheme =
      token.colorBgLayout === "#000000" ? "dark" : "light";
    try {
      getCurrentWindow().setTheme?.(
        token.colorBgLayout === "#000000" ? "dark" : "light",
      );
    } catch { /* non-Tauri env */ }
  }, [token.colorBgLayout, token.colorText]);

  useNotification();

  const statusColor =
    status?.status === "running"
      ? "#22c55e"
      : status?.status === "error"
        ? "#ef4444"
        : "#9ca3af";

  const switchLang = () => {
    const next = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(next);
    localStorage.setItem("aidaguard-lang", next);
  };

  return (
    <Layout style={{ minHeight: "100vh" }}>
      <Sider
        width={220}
        style={{
          background: token.colorBgContainer,
          borderRight: `1px solid ${token.colorBorderSecondary}`,
          display: "flex",
          flexDirection: "column",
        }}
      >
        <div
          style={{
            height: 64,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            borderBottom: `1px solid ${token.colorBorderSecondary}`,
          }}
        >
          <h1 style={{ fontSize: 20, fontWeight: 700, margin: 0 }}>
            <span style={{ color: token.colorPrimary }}>Aida</span>
            <span>guard</span>
          </h1>
        </div>

        {/* Navigation */}
        <div style={{ padding: "8px 12px", flex: 1 }}>
          {menuItems.map((item) => {
            const isActive = location.pathname === item.key;
            return (
              <div
                key={item.key}
                onClick={() => navigate(item.key)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 10,
                  padding: "10px 12px",
                  marginBottom: 2,
                  borderRadius: 8,
                  cursor: "pointer",
                  fontSize: 14,
                  fontWeight: isActive ? 600 : 400,
                  color: isActive
                    ? token.colorPrimary
                    : token.colorText,
                  background: isActive
                    ? token.colorPrimaryBg
                    : "transparent",
                  transition: "all 0.2s",
                }}
              >
                {item.icon}
                {item.label}
              </div>
            );
          })}
        </div>

      </Sider>

      <Layout>
        <Header
          style={{
            background: token.colorBgContainer,
            borderBottom: `1px solid ${token.colorBorderSecondary}`,
            padding: "0 24px",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            height: 64,
          }}
        >
          <span style={{ fontSize: 16, fontWeight: 500 }}>
            {menuItems.find((m) => m.key === location.pathname)?.label ||
              t("仪表盘")}
          </span>

          <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                background: statusColor,
                display: "inline-block",
              }}
            />
            <span style={{ fontSize: 13, color: token.colorTextSecondary }}>
              {status?.status === "running" ? t("代理运行中") : t("代理已停止")}
            </span>
            <Button
              type="text"
              size="small"
              onClick={switchLang}
              style={{ fontSize: 12, color: token.colorTextSecondary }}
            >
              {i18n.language === "zh" ? "EN" : "中文"}
            </Button>
          </div>
        </Header>

        <Content
          style={{
            padding: 24,
            background: token.colorBgLayout,
            overflow: "hidden",
            height: "calc(100vh - 64px)",
          }}
        >
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/audit" element={<AuditLog />} />
            <Route path="/rules" element={<Rules />} />
            <Route path="/upstreams" element={<Upstreams />} />
            <Route path="/tools" element={<ToolsConfig />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}
