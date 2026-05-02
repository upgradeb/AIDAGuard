import { Routes, Route } from "react-router-dom";
import { Layout, theme } from "antd";
import {
  DashboardOutlined,
  AuditOutlined,
  SafetyOutlined,
  ApiOutlined,
  SettingOutlined,
} from "@ant-design/icons";
import { useNavigate, useLocation } from "react-router-dom";
import { useEffect } from "react";
import Dashboard from "./pages/Dashboard";
import AuditLog from "./pages/AuditLog";
import Rules from "./pages/Rules";
import Settings from "./pages/Settings";
import Upstreams from "./pages/Upstreams";
import { useProxyStore } from "./store/useProxyStore";
import { useNotification } from "./hooks/useNotification";

const { Sider, Header, Content } = Layout;

const menuItems = [
  { key: "/", icon: <DashboardOutlined />, label: "仪表盘" },
  { key: "/audit", icon: <AuditOutlined />, label: "审计记录" },
  { key: "/rules", icon: <SafetyOutlined />, label: "规则管理" },
  { key: "/upstreams", icon: <ApiOutlined />, label: "大模型接入" },
  { key: "/settings", icon: <SettingOutlined />, label: "设置" },
];

export default function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const status = useProxyStore((s) => s.status);
  const { token } = theme.useToken();
  const startListening = useProxyStore((s) => s.startListening);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);

  useEffect(() => {
    fetchStatus();
    const cleanup = startListening();
    return cleanup;
  }, []);

  useNotification();

  const statusColor =
    status?.status === "running"
      ? "#22c55e"
      : status?.status === "error"
        ? "#ef4444"
        : "#9ca3af";

  return (
    <Layout style={{ minHeight: "100vh" }}>
      <Sider
        width={220}
        style={{
          background: token.colorBgContainer,
          borderRight: `1px solid ${token.colorBorderSecondary}`,
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

        {/* Proxy status indicator */}
        <div
          style={{
            padding: "12px 16px",
            display: "flex",
            alignItems: "center",
            gap: 8,
            fontSize: 13,
            color: token.colorTextSecondary,
          }}
        >
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: statusColor,
              display: "inline-block",
            }}
          />
          {status?.status === "running" ? "代理运行中" : "代理已停止"}
        </div>

        {/* Navigation */}
        <div style={{ padding: "8px 12px" }}>
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
              "仪表盘"}
          </span>
        </Header>

        <Content
          style={{
            padding: 24,
            background: token.colorBgLayout,
            overflow: "auto",
          }}
        >
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/audit" element={<AuditLog />} />
            <Route path="/rules" element={<Rules />} />
            <Route path="/upstreams" element={<Upstreams />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  );
}
