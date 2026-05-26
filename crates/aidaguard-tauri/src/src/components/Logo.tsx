import { theme } from "antd";

interface LogoProps {
  size?: number;
  collapsed?: boolean;
}

export default function Logo({ size = 32, collapsed = false }: LogoProps) {
  const { token } = theme.useToken();

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: collapsed ? 0 : 10,
        userSelect: "none",
      }}
    >
      <img
        src="/logo.png"
        alt="AIDAGuard"
        style={{ width: size, height: size, borderRadius: size * 0.22 }}
      />
      {!collapsed && (
        <h1 style={{ fontSize: 20, fontWeight: 700, margin: 0, whiteSpace: "nowrap" }}>
          <span style={{ color: token.colorPrimary }}>AIDA</span>
          <span style={{ color: token.colorText }}>Guard</span>
        </h1>
      )}
    </div>
  );
}
