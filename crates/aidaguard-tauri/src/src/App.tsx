import { Routes, Route } from "react-router-dom";
import { useNavigate, useLocation } from "react-router-dom";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  LayoutDashboard,
  FileSearch,
  Shield,
  Settings as SettingsIcon,
  Wrench,
  Server,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import Dashboard from "./pages/Dashboard";
import AuditLog from "./pages/AuditLog";
import Rules from "./pages/Rules";
import Settings from "./pages/Settings";
import Upstreams from "./pages/Upstreams";
import ToolsConfig from "./pages/ToolsConfig";
import Logo from "./components/Logo";
import { useProxyStore } from "./store/useProxyStore";
import { useNotification } from "./hooks/useNotification";

const menuItems = [
  { key: "/", icon: LayoutDashboard, labelKey: "Dashboard" },
  { key: "/audit", icon: FileSearch, labelKey: "Audit Log" },
  { key: "/upstreams", icon: Server, labelKey: "LLM Upstreams" },
  { key: "/tools", icon: Wrench, labelKey: "AI Tools Config" },
  { key: "/rules", icon: Shield, labelKey: "Rules" },
  { key: "/settings", icon: SettingsIcon, labelKey: "Settings" },
];

export default function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const status = useProxyStore((s) => s.status);
  const startListening = useProxyStore((s) => s.startListening);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const { t, i18n } = useTranslation();

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

  const switchLang = () => {
    const next = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(next);
    localStorage.setItem("aidaguard-lang", next);
  };

  const currentLabel =
    menuItems.find((m) => m.key === location.pathname)?.labelKey || "Dashboard";

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside className="w-[230px] flex flex-col border-r bg-card shrink-0">
        <div className="h-16 flex items-center justify-center border-b">
          <Logo size={30} />
        </div>

        <nav className="p-2 flex-1">
          {menuItems.map((item) => {
            const isActive = location.pathname === item.key;
            const Icon = item.icon;
            return (
              <button
                key={item.key}
                onClick={() => navigate(item.key)}
                className={cn(
                  "flex items-center gap-2.5 w-full px-3 py-2.5 mb-0.5 rounded-lg text-sm transition-colors cursor-pointer",
                  isActive
                    ? "bg-primary/10 text-preset font-semibold"
                    : "text-foreground hover:bg-accent"
                )}
              >
                <Icon className="h-4 w-4 shrink-0" />
                <span className="truncate">{t(item.labelKey)}</span>
              </button>
            );
          })}
        </nav>
      </aside>

      {/* Main area */}
      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-16 flex items-center justify-between px-6 border-b bg-card shrink-0">
          <span className="text-base font-medium">{t(currentLabel)}</span>

          <div className="flex items-center gap-4">
            <span
              className="w-2 h-2 rounded-full shrink-0"
              style={{ background: statusColor }}
            />
            <span className="text-[13px] text-muted-foreground whitespace-nowrap min-w-[100px]">
              {status?.status === "running" ? t("Proxy Running") : t("Proxy Stopped")}
            </span>
            <Button
              variant="ghost"
              size="sm"
              onClick={switchLang}
              className="text-xs text-muted-foreground min-w-[44px]"
            >
              {i18n.language === "zh" ? "EN" : "中文"}
            </Button>
          </div>
        </header>

        <main className="flex-1 min-h-0 p-6 bg-background">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/audit" element={<AuditLog />} />
            <Route path="/rules" element={<Rules />} />
            <Route path="/upstreams" element={<Upstreams />} />
            <Route path="/tools" element={<ToolsConfig />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}
