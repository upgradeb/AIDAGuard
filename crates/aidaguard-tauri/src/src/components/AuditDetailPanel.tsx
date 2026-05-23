import { useState } from "react";
import { Descriptions, Typography, Tag, theme, Button } from "antd";
import { DownOutlined, UpOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import type { DetectionRecord } from "../types";
import dayjs from "dayjs";

interface AuditDetailPanelProps {
  record: DetectionRecord;
}

const PREVIEW_MAX = 300;

export default function AuditDetailPanel({ record }: AuditDetailPanelProps) {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const [bodyExpanded, setBodyExpanded] = useState(false);

  const body = record.sanitizedBody || "";
  const bodyLen = body.length;
  const truncated = !bodyExpanded && bodyLen > PREVIEW_MAX;
  const displayBody = truncated ? body.slice(0, PREVIEW_MAX) + "…" : body;

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

      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
        <Typography.Text strong>
          {t("Sanitized Request Body")}
        </Typography.Text>
        <Typography.Text type="secondary" style={{ fontSize: 12 }}>
          ({bodyLen.toLocaleString()} chars)
        </Typography.Text>
      </div>
      {body ? (
        <>
          <pre
            style={{
              background: token.colorFillAlter,
              color: token.colorText,
              padding: 12,
              borderRadius: 6,
              fontSize: 12,
              maxHeight: truncated ? 120 : 320,
              overflow: "auto",
              whiteSpace: "pre-wrap",
              wordBreak: "break-all",
              margin: 0,
            }}
          >
            {displayBody}
          </pre>
          {bodyLen > PREVIEW_MAX && (
            <Button
              type="link"
              size="small"
              icon={bodyExpanded ? <UpOutlined /> : <DownOutlined />}
              onClick={() => setBodyExpanded(!bodyExpanded)}
              style={{ padding: "4px 0" }}
            >
              {bodyExpanded ? t("Collapse") : t("Show Full Body")}
            </Button>
          )}
        </>
      ) : (
        <Typography.Text type="secondary">—</Typography.Text>
      )}
    </div>
  );
}
