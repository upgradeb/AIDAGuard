import { Descriptions, Typography, Tag } from "antd";
import { useTranslation } from "react-i18next";
import type { DetectionRecord } from "../types";
import dayjs from "dayjs";

interface AuditDetailPanelProps {
  record: DetectionRecord;
}

export default function AuditDetailPanel({ record }: AuditDetailPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="detail-panel" style={{ padding: "0 24px 16px" }}>
      <Descriptions
        bordered
        size="small"
        column={2}
        style={{ marginBottom: 16 }}
      >
        <Descriptions.Item label={t("Record ID")} span={2}>
          <Typography.Text copyable style={{ fontSize: 12 }}>
            {record.id}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("Time")}>
          {dayjs(record.timestampMs).format("YYYY-MM-DD HH:mm:ss.SSS")}
        </Descriptions.Item>
        <Descriptions.Item label={t("Response Status")}>
          <Tag color={record.responseStatus < 300 ? "green" : "red"}>
            {record.responseStatus}
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label={t("Tool")}>
          {record.toolName || "—"}
        </Descriptions.Item>
        <Descriptions.Item label={t("Rule Name")}>
          <Tag color="orange">{record.ruleName || record.ruleId}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label={t("Audit Strategy")}>
          {record.strategy === "detect" ? (
            <Tag color="orange">{t("Detect Only")}</Tag>
          ) : record.strategy === "mask" ? (
            <Tag color="purple">{t("Partial Mask")}</Tag>
          ) : (
            <Tag color="blue">{t("Placeholder Replacement")}</Tag>
          )}
        </Descriptions.Item>
        <Descriptions.Item label={t("Placeholder")} span={2}>
          <Typography.Text code>{record.placeholder}</Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("LLM / Model")} span={2}>
          {record.requestPath || "—"}
        </Descriptions.Item>
        <Descriptions.Item label={t("Original Data")} span={2}>
          <Typography.Text
            copyable
            style={{ color: "#ef4444", wordBreak: "break-all" }}
          >
            {record.original}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("Context")} span={2}>
          <Typography.Paragraph
            style={{ fontSize: 13, wordBreak: "break-all" }}
          >
            {record.context || "—"}
          </Typography.Paragraph>
        </Descriptions.Item>
      </Descriptions>

      <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
        {t("Sanitized Request Body")}
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
