import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { useTranslation } from "react-i18next";

interface RuleDistribution {
  ruleId: string;
  ruleName: string;
  count: number;
}

const COLORS = [
  "#3b82f6",
  "#22c55e",
  "#f59e0b",
  "#ef4444",
  "#8b5cf6",
  "#06b6d4",
  "#ec4899",
  "#84cc16",
];

interface RuleHitChartProps {
  data: RuleDistribution[];
}

export default function RuleHitChart({ data }: RuleHitChartProps) {
  const { t } = useTranslation();

  if (data.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground text-sm">
        {t("No Data")}
      </div>
    );
  }

  const chartData = data.map((d) => ({
    name: d.ruleName || d.ruleId,
    value: d.count,
  }));

  return (
    <ResponsiveContainer width="100%" height={280}>
      <PieChart>
        <Pie
          data={chartData}
          cx="50%"
          cy="50%"
          innerRadius={60}
          outerRadius={100}
          paddingAngle={2}
          dataKey="value"
        >
          {chartData.map((_, i) => (
            <Cell key={`cell-${i}`} fill={COLORS[i % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip />
        <Legend />
      </PieChart>
    </ResponsiveContainer>
  );
}
