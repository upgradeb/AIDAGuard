import { Card, Typography, theme } from "antd";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";

interface StatCardProps {
  title: string;
  value: string | number;
  icon?: ReactNode;
  color?: string;
  gradient?: boolean;
}

// Animated number counter
function AnimatedNumber({ value }: { value: number }) {
  const [displayValue, setDisplayValue] = useState(0);

  useEffect(() => {
    const duration = 500;
    const steps = 20;
    const increment = value / steps;
    let current = 0;
    let step = 0;

    const timer = setInterval(() => {
      step++;
      current = Math.min(Math.round(increment * step), value);
      setDisplayValue(current);
      if (step >= steps) {
        clearInterval(timer);
      }
    }, duration / steps);

    return () => clearInterval(timer);
  }, [value]);

  return <span className="stat-number">{displayValue.toLocaleString()}</span>;
}

export default function StatCard({ title, value, icon, color, gradient }: StatCardProps) {
  const { token } = theme.useToken();
  const numericValue = typeof value === "number" ? value : parseInt(value.replace(/[^0-9]/g, "")) || 0;

  return (
    <Card
      size="small"
      className="stat-card"
      style={{
        borderRadius: 12,
        border: `1px solid ${token.colorBorderSecondary}`,
        boxShadow: "0 2px 8px rgba(0,0,0,0.06)",
        background: gradient ? (color === "success" ? "linear-gradient(135deg, #11998e 0%, #38ef7d 100%)" :
                       color === "warning" ? "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)" :
                       "linear-gradient(135deg, #667eea 0%, #764ba2 100%)") : undefined,
        color: gradient ? "#fff" : undefined,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        {icon && (
          <div
            style={{
              width: 40,
              height: 40,
              borderRadius: 10,
              background: gradient ? "rgba(255,255,255,0.2)" : (color || token.colorPrimaryBg),
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: gradient ? "#fff" : (color ? "#fff" : token.colorPrimary),
              fontSize: 18,
            }}
          >
            {icon}
          </div>
        )}
        <div>
          <Typography.Text
            type={gradient ? undefined : "secondary"}
            style={{ fontSize: 13, display: "block", color: gradient ? "rgba(255,255,255,0.85)" : undefined, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis", minHeight: 20 }}
          >
            {title}
          </Typography.Text>
          <Typography.Text
            strong
            style={{ fontSize: 24, lineHeight: "32px", color: gradient ? "#fff" : undefined }}
          >
            {typeof value === "number" ? (
              <AnimatedNumber value={numericValue} />
            ) : (
              <span className="stat-number">{value}</span>
            )}
          </Typography.Text>
        </div>
      </div>
    </Card>
  );
}
