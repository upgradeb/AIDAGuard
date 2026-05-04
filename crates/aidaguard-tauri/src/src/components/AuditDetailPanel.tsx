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
    <div style={{ padding: "0 24px 16px" }}>
      <Descriptions
        bordered
        size="small"
        column={2}
        style={{ marginBottom: 16 }}
      >
        <Descriptions.Item label={t("记录 ID")} span={2}>
          <Typography.Text copyable style={{ fontSize: 12 }}>
            {record.id}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("时间")}>
          {dayjs(record.timestampMs).format("YYYY-MM-DD HH:mm:ss.SSS")}
        </Descriptions.Item>
        <Descriptions.Item label={t("响应状态")}>
          <Tag color={record.responseStatus < 300 ? "green" : "red"}>
            {record.responseStatus}
          </Tag>
        </Descriptions.Item>
        <Descriptions.Item label={t("工具名")}>
          {record.toolName || "—"}
        </Descriptions.Item>
        <Descriptions.Item label={t("规则名")}>
          <Tag color="orange">{record.ruleName || record.ruleId}</Tag>
        </Descriptions.Item>
        <Descriptions.Item label={t("审计策略")}>
          {record.strategy === "detect" ? (
            <Tag color="orange">{t("仅检测")}</Tag>
          ) : record.strategy === "mask" ? (
            <Tag color="purple">{t("部分掩码")}</Tag>
          ) : (
            <Tag color="blue">{t("占位符替换")}</Tag>
          )}
        </Descriptions.Item>
        <Descriptions.Item label={t("占位符")} span={2}>
          <Typography.Text code>{record.placeholder}</Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("大模型/模型")} span={2}>
          {record.requestPath || "—"}
        </Descriptions.Item>
        <Descriptions.Item label={t("原始数据")} span={2}>
          <Typography.Text
            copyable
            style={{ color: "#ef4444", wordBreak: "break-all" }}
          >
            {record.original}
          </Typography.Text>
        </Descriptions.Item>
        <Descriptions.Item label={t("上下文")} span={2}>
          <Typography.Paragraph
            style={{ fontSize: 13, wordBreak: "break-all" }}
          >
            {record.context || "—"}
          </Typography.Paragraph>
        </Descriptions.Item>
      </Descriptions>

      <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
        {t("替换后请求体")}
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
