import { List, Space, Tag, Typography, theme } from "antd";
import { WarningOutlined } from "@ant-design/icons";
import type { DetectionRecord } from "../types";
import dayjs from "dayjs";

interface EventFeedProps {
  records: DetectionRecord[];
  onClickRecord?: (id: string) => void;
}

function strategyLabel(strategy: string) {
  switch (strategy) {
    case "placeholder":
    case "filter":
      return { label: "已过滤", color: "#22c55e" as const };
    case "detect":
      return { label: "仅检测", color: "#f59e0b" as const };
    case "mask":
      return { label: "已掩码", color: "#8b5cf6" as const };
    default:
      return { label: strategy, color: "#3b82f6" as const };
  }
}

export default function EventFeed({ records, onClickRecord }: EventFeedProps) {
  const { token } = theme.useToken();

  if (records.length === 0) {
    return (
      <div
        style={{
          textAlign: "center",
          padding: 32,
          color: token.colorTextQuaternary,
        }}
      >
        <WarningOutlined style={{ fontSize: 32, marginBottom: 8 }} />
        <br />
        暂无检测事件
      </div>
    );
  }

  return (
    <List
      dataSource={records}
      renderItem={(item) => {
        const strat = strategyLabel(item.strategy);
        return (
          <List.Item
            style={{
              padding: "10px 0",
              borderBottom: `1px solid ${token.colorBorderSecondary}`,
              cursor: onClickRecord ? "pointer" : "default",
            }}
            onClick={() => onClickRecord?.(item.id)}
          >
            <div style={{ width: "100%" }}>
              {/* 第一行：策略 + 规则名 + 工具 + 时间 */}
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  marginBottom: 6,
                }}
              >
                <Space size={6}>
                  <Tag color={strat.color}>{strat.label}</Tag>
                  <Typography.Text strong style={{ fontSize: 13 }}>
                    {item.ruleName || item.ruleId}
                  </Typography.Text>
                  {item.toolName && (
                    <Tag color="geekblue" style={{ fontSize: 11 }}>{item.toolName}</Tag>
                  )}
                </Space>
                <Typography.Text type="secondary" style={{ fontSize: 11 }}>
                  {dayjs(item.timestampMs).format("MM-DD HH:mm:ss")}
                </Typography.Text>
              </div>

              {/* 第二行：检测数据（左）+ 请求路径（右） */}
              <div
                style={{
                  display: "flex",
                  alignItems: "flex-start",
                  gap: 8,
                }}
              >
                <Typography.Text
                  style={{
                    flex: 1,
                    fontSize: 13,
                    color: "#ef4444",
                    background: "#fef2f2",
                    padding: "2px 8px",
                    borderRadius: 4,
                    wordBreak: "break-all",
                    lineHeight: 1.5,
                  }}
                  copyable
                >
                  {item.original || "—"}
                </Typography.Text>
                <Typography.Text
                  type="secondary"
                  style={{ fontSize: 11, whiteSpace: "nowrap", flexShrink: 0, maxWidth: 120 }}
                  ellipsis
                >
                  {item.requestPath || "—"}
                </Typography.Text>
              </div>
            </div>
          </List.Item>
        );
      }}
    />
  );
}
