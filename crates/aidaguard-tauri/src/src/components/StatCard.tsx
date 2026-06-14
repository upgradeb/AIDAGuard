import { Card, CardContent } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { ReactNode } from "react";
import { useEffect, useState } from "react";

interface StatCardProps {
  title: string;
  value: string | number;
  icon?: ReactNode;
  color?: string;
  gradient?: boolean;
}

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
      if (step >= steps) clearInterval(timer);
    }, duration / steps);

    return () => clearInterval(timer);
  }, [value]);

  return <span className="stat-number">{displayValue.toLocaleString()}</span>;
}

const gradientMap: Record<string, string> = {
  success: "bg-gradient-to-br from-[#11998e] to-[#38ef7d]",
  warning: "bg-gradient-to-br from-[#f093fb] to-[#f5576c]",
  primary: "bg-gradient-to-br from-[#667eea] to-[#764ba2]",
};

export default function StatCard({ title, value, icon, color, gradient }: StatCardProps) {
  const numericValue =
    typeof value === "number" ? value : parseInt(value.replace(/[^0-9]/g, "")) || 0;

  return (
    <Card
      className={cn(
        "rounded-xl shadow-sm hover:-translate-y-0.5 transition-transform",
        gradient && color ? gradientMap[color] || gradientMap.primary : ""
      )}
    >
      <CardContent className="p-4">
        <div className="flex items-center gap-3">
          {icon && (
            <div
              className={cn(
                "w-10 h-10 rounded-[10px] flex items-center justify-center text-lg",
                gradient
                  ? "bg-white/20 text-white"
                  : color
                    ? "text-white"
                    : "bg-primary/10 text-preset"
              )}
              style={!gradient && color ? { background: color } : undefined}
            >
              {icon}
            </div>
          )}
          <div>
            <span
              className={cn(
                "block text-[13px] min-h-[20px] truncate",
                gradient ? "text-white/85" : "text-muted-foreground"
              )}
            >
              {title}
            </span>
            <span
              className={cn(
                "text-2xl leading-8 font-bold",
                gradient ? "text-white" : ""
              )}
            >
              {typeof value === "number" ? (
                <AnimatedNumber value={numericValue} />
              ) : (
                <span className="stat-number">{value}</span>
              )}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
