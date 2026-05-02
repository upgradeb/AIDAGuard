import { Card, Typography, theme } from "antd";
import type { ReactNode } from "react";

interface StatCardProps {
  title: string;
  value: string | number;
  icon?: ReactNode;
  color?: string;
}

export default function StatCard({ title, value, icon, color }: StatCardProps) {
  const { token } = theme.useToken();

  return (
    <Card
      size="small"
      style={{
        borderRadius: 12,
        border: `1px solid ${token.colorBorderSecondary}`,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {icon && (
          <div
            style={{
              width: 40,
              height: 40,
              borderRadius: 10,
              background: color || token.colorPrimaryBg,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: color ? "#fff" : token.colorPrimary,
              fontSize: 18,
            }}
          >
            {icon}
          </div>
        )}
        <div>
          <Typography.Text
            type="secondary"
            style={{ fontSize: 13, display: "block" }}
          >
            {title}
          </Typography.Text>
          <Typography.Text
            strong
            style={{ fontSize: 24, lineHeight: "32px" }}
          >
            {value}
          </Typography.Text>
        </div>
      </div>
    </Card>
  );
}
