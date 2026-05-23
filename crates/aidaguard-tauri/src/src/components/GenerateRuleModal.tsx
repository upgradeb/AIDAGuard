import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Modal,
  Input,
  Button,
  Descriptions,
  Tag,
  Typography,
  Space,
  message,
  Alert,
  Spin,
} from "antd";
import { theme } from "antd";
import {
  RobotOutlined,
  ThunderboltOutlined,
  EditOutlined,
  ReloadOutlined,
  ApiOutlined,
} from "@ant-design/icons";
import { generateRule, type GeneratedRule } from "../api/rules";
import type { RuleDef } from "../types";

interface GenerateRuleModalProps {
  open: boolean;
  defaultModelLabel: string;
  onApply: (rule: RuleDef) => void;
  onClose: () => void;
}

export default function GenerateRuleModal({
  open,
  defaultModelLabel,
  onApply,
  onClose,
}: GenerateRuleModalProps) {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const [sampleText, setSampleText] = useState("");
  const [generating, setGenerating] = useState(false);
  const [result, setResult] = useState<GeneratedRule | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    if (!sampleText.trim()) {
      message.warning(t("Please enter test sample"));
      return;
    }
    setGenerating(true);
    setError(null);
    try {
      const rule = await generateRule(sampleText);
      setResult(rule);
    } catch (e) {
      setError(String(e));
    } finally {
      setGenerating(false);
    }
  };

  const handleApply = () => {
    if (!result) return;
    onApply({
      id: result.id || "",
      name: result.name,
      pattern: result.pattern,
      strategy: result.strategy as "placeholder" | "mask",
      mode: result.mode as "detect" | "filter",
      priority: result.priority,
      enabled: true,
    });
    setSampleText("");
    setResult(null);
    setError(null);
    onClose();
  };

  const handleClose = () => {
    setSampleText("");
    setResult(null);
    setError(null);
    onClose();
  };

  const modelLabel = result
    ? `${result.upstreamName} / ${result.model}`
    : defaultModelLabel;

  return (
    <Modal
      title={
        <Space>
          <RobotOutlined />
          {t("AI-Generated Rules")}
        </Space>
      }
      open={open}
      onCancel={handleClose}
      footer={
        result
          ? [
              <Button key="close" onClick={handleClose}>
                {t("Close")}
              </Button>,
              <Button
                key="regenerate"
                icon={<ReloadOutlined />}
                onClick={handleGenerate}
                loading={generating}
              >
                {t("Regenerate")}
              </Button>,
              <Button
                key="apply"
                type="primary"
                icon={<EditOutlined />}
                onClick={handleApply}
              >
                {t("Apply to Editor")}
              </Button>,
            ]
          : [
              <Button key="cancel" onClick={handleClose}>
                {t("Cancel")}
              </Button>,
            ]
      }
      width={640}
      destroyOnClose
    >
      {/* 模型信息 */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          marginBottom: 12,
          padding: "6px 12px",
          borderRadius: 6,
          background: token.colorPrimaryBg,
          border: `1px solid ${token.colorPrimaryBorder}`,
          fontSize: 13,
        }}
      >
        <ApiOutlined style={{ color: token.colorPrimary }} />
        <Typography.Text style={{ color: token.colorPrimary }}>
          {t("Model: ")}<strong>{modelLabel}</strong>
        </Typography.Text>
      </div>

      <Typography.Paragraph type="secondary" style={{ fontSize: 13 }}>
        {t("Enter a test sample containing sensitive data. The LLM will analyze it and generate detection rules automatically. You can further refine the result in the rule editor.")}
      </Typography.Paragraph>

      <Input.TextArea
        value={sampleText}
        onChange={(e) => setSampleText(e.target.value)}
        placeholder={t("Example:\nPatient Zhang San, Phone 13812345678, ID 320102199001011234")}
        rows={4}
        style={{ marginBottom: 12 }}
      />

      {!result && (
        <Button
          type="primary"
          icon={<ThunderboltOutlined />}
          onClick={handleGenerate}
          loading={generating}
          disabled={!sampleText.trim()}
          block
        >
          {generating ? t("Generating...") : t("Generate Rule")}
        </Button>
      )}

      {generating && !result && (
        <div style={{ textAlign: "center", padding: 24 }}>
          <Spin tip={t("LLM is analyzing sample...")} />
        </div>
      )}

      {error && (
        <Alert
          type="error"
          showIcon
          message={t("Generation Failed")}
          description={error}
          closable
          style={{ marginTop: 12, borderRadius: 8 }}
        />
      )}

      {result && (
        <Spin spinning={generating} tip={t("Regenerating...")}>
          <div
            style={{
              marginTop: 16,
              padding: 16,
              background: generating ? token.colorWarningBg : token.colorSuccessBg,
              border: `1px solid ${generating ? token.colorWarningBorder : token.colorSuccessBorder}`,
              borderRadius: 8,
              transition: "background 0.3s, border-color 0.3s",
            }}
          >
            <Typography.Text
              strong
              style={{
                color: generating ? "#faad14" : "#52c41a",
                marginBottom: 12,
                display: "block",
              }}
            >
              {generating ? t("Regenerating...") : t("Done — {{upstreamName}} / {{model}}", { upstreamName: result.upstreamName, model: result.model })}
            </Typography.Text>
          <Descriptions column={1} size="small" bordered>
            <Descriptions.Item label={t("Rule ID")}>
              <Typography.Text code copyable>{result.id}</Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("Rule Name")}>
              <Typography.Text strong>{result.name}</Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("Pattern")}>
              <Typography.Text code copyable>
                {result.pattern}
              </Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("Strategy")}>
              <Tag color={result.strategy === "placeholder" ? "blue" : "purple"}>
                {result.strategy === "placeholder" ? t("Placeholder Replacement") : t("Partial Mask")}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label={t("Mode")}>
              <Tag color={result.mode === "filter" ? "green" : "orange"}>
                {result.mode === "filter" ? t("Filter & Replace") : t("Detect Only")}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label={t("Priority")}>
              {result.priority}
            </Descriptions.Item>
            <Descriptions.Item label={t("Generation Model")}>
              <Typography.Text code>{result.upstreamName} / {result.model}</Typography.Text>
            </Descriptions.Item>
          </Descriptions>
          </div>
        </Spin>
      )}
    </Modal>
  );
}
