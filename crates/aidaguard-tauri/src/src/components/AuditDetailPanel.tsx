import { Descriptions, Typography, Tag } from "antd";
import type { DetectionRecord } from "../types";
import dayjs from "dayjs";

interface AuditDetailPanelProps {
  record: DetectionRecord;
}

export default function AuditDetailPanel({ record }: AuditDetailPanelProps) {
  return (
    <div style={{ padding: "0 24px 16px" }}>
      <Descriptions
        bordered
        size="small"
        column={2}
        style={{ marginBottom: 16 }}
      >
        <Descriptions.Item label="记录 ID" span={2}>
          <Typography.Text copyable style={{ fontSize: 12 }}>
            {record.id}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label="时间">
          {dayjs(record.timestampMs).format("YYYY-MM-DD HH:mm:ss.SSS")}
        </Descriptions.Item>
        <Descriptions.Item label="响应状态">
          <Tag color={record.responseStatus < 300 ? "green" : "red"}>
            {record.responseStatus}
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label="工具名">
          {record.toolName || "—"}
        </Descriptions.Item>
        <Descriptions.Item label="规则">
          <Tag color="orange">{record.ruleId}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="策略">
          <Tag>{record.strategy}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label="占位符" span={2}>
          <Typography.Text code>{record.placeholder}</Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label="请求路径" span={2}>
          {record.requestPath || "/"}
        </Descriptions.Item>
        <Descriptions.Item label="原始数据" span={2}>
          <Typography.Text
            copyable
            style={{ color: "#ef4444", wordBreak: "break-all" }}
          >
            {record.original}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label="上下文" span={2}>
          <Typography.Paragraph
            style={{ fontSize: 13, wordBreak: "break-all" }}
          >
            {record.context || "—"}
          </Typography.Paragraph>
        </Descriptions.Item>
      </Descriptions>

      <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
        替换后请求体
      </Typography.Text>
      <pre
        style={{
          background: "#f5f5f5",
          padding: 12,
          borderRadius: 6,
          fontSize: 12,
          maxHeight: 300,
          overflow: "auto",
          whiteSpace: "pre-wrap",
          wordBreak: "break-all",
        }}
      >
        {record.sanitizedBody || "—"}
      </pre>
    </div>
  );
}
