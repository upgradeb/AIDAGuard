import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";

interface RuleDistribution {
  ruleId: string;
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
  if (data.length === 0) {
    return (
      <div
        style={{
          textAlign: "center",
          padding: 32,
          color: "#9ca3af",
          fontSize: 14,
        }}
      >
        暂无数据
      </div>
    );
  }

  const chartData = data.map((d) => ({
    name: d.ruleId,
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
