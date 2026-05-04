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
  const [sampleText, setSampleText] = useState("");
  const [generating, setGenerating] = useState(false);
  const [result, setResult] = useState<GeneratedRule | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    if (!sampleText.trim()) {
      message.warning(t("请输入测试样例"));
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
          {t("大模型生成规则")}
        </Space>
      }
      open={open}
      onCancel={handleClose}
      footer={
        result
          ? [
              <Button key="close" onClick={handleClose}>
                {t("关闭")}
              </Button>,
              <Button
                key="regenerate"
                icon={<ReloadOutlined />}
                onClick={handleGenerate}
                loading={generating}
              >
                {t("重新生成")}
              </Button>,
              <Button
                key="apply"
                type="primary"
                icon={<EditOutlined />}
                onClick={handleApply}
              >
                {t("应用到编辑器")}
              </Button>,
            ]
          : [
              <Button key="cancel" onClick={handleClose}>
                {t("取消")}
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
          background: "#f0f5ff",
          border: "1px solid #d6e4ff",
          fontSize: 13,
        }}
      >
        <ApiOutlined style={{ color: "#1677ff" }} />
        <Typography.Text style={{ color: "#1677ff" }}>
          {t("调用模型：")}<strong>{modelLabel}</strong>
        </Typography.Text>
      </div>

      <Typography.Paragraph type="secondary" style={{ fontSize: 13 }}>
        {t("输入包含敏感数据的测试样例，由大模型自动分析并生成检测规则。生成后可在规则编辑器中进一步调整。")}
      </Typography.Paragraph>

      <Input.TextArea
        value={sampleText}
        onChange={(e) => setSampleText(e.target.value)}
        placeholder={t("例如：\\n患者张三，电话13812345678，身份证320102199001011234")}
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
          {generating ? t("生成中...") : t("生成规则")}
        </Button>
      )}

      {generating && !result && (
        <div style={{ textAlign: "center", padding: 24 }}>
          <Spin tip={t("大模型正在分析样例...")} />
        </div>
      )}

      {error && (
        <Alert
          type="error"
          showIcon
          message={t("生成失败")}
          description={error}
          closable
          style={{ marginTop: 12, borderRadius: 8 }}
        />
      )}

      {result && (
        <Spin spinning={generating} tip={t("正在重新生成...")}>
          <div
            style={{
              marginTop: 16,
              padding: 16,
              background: generating ? "#fffbe6" : "#f6ffed",
              border: `1px solid ${generating ? "#ffe58f" : "#b7eb8f"}`,
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
              {generating ? t("重新生成中...") : t("生成完成 — {{upstreamName}} / {{model}}", { upstreamName: result.upstreamName, model: result.model })}
            </Typography.Text>
          <Descriptions column={1} size="small" bordered>
            <Descriptions.Item label={t("规则 ID")}>
              <Typography.Text code copyable>{result.id}</Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("规则名")}>
              <Typography.Text strong>{result.name}</Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("正则")}>
              <Typography.Text code copyable>
                {result.pattern}
              </Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label={t("策略")}>
              <Tag color={result.strategy === "placeholder" ? "blue" : "purple"}>
                {result.strategy === "placeholder" ? t("占位符替换") : t("部分掩码")}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label={t("模式")}>
              <Tag color={result.mode === "filter" ? "green" : "orange"}>
                {result.mode === "filter" ? t("过滤替换") : t("仅检测")}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label={t("优先级")}>
              {result.priority}
            </Descriptions.Item>
            <Descriptions.Item label={t("生成模型")}>
              <Typography.Text code>{result.upstreamName} / {result.model}</Typography.Text>
            </Descriptions.Item>
          </Descriptions>
          </div>
        </Spin>
      )}
    </Modal>
  );
}
