import { useState } from "react";
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
} from "@ant-design/icons";
import { generateRule, type GeneratedRule } from "../api/rules";
import type { RuleDef } from "../types";

interface GenerateRuleModalProps {
  open: boolean;
  onApply: (rule: RuleDef) => void;
  onClose: () => void;
}

export default function GenerateRuleModal({
  open,
  onApply,
  onClose,
}: GenerateRuleModalProps) {
  const [sampleText, setSampleText] = useState("");
  const [generating, setGenerating] = useState(false);
  const [result, setResult] = useState<GeneratedRule | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    if (!sampleText.trim()) {
      message.warning("请输入测试样例");
      return;
    }
    setGenerating(true);
    setError(null);
    setResult(null);
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
      id: "",
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

  return (
    <Modal
      title={
        <Space>
          <RobotOutlined />
          大模型生成规则
        </Space>
      }
      open={open}
      onCancel={handleClose}
      footer={
        result
          ? [
              <Button key="cancel" onClick={handleClose}>
                关闭
              </Button>,
              <Button
                key="apply"
                type="primary"
                icon={<EditOutlined />}
                onClick={handleApply}
              >
                应用到编辑器
              </Button>,
            ]
          : [
              <Button key="cancel" onClick={handleClose}>
                取消
              </Button>,
            ]
      }
      width={640}
      destroyOnClose
    >
      <Typography.Paragraph type="secondary" style={{ fontSize: 13 }}>
        输入包含敏感数据的测试样例，由大模型自动分析并生成检测规则。生成后可在规则编辑器中进一步调整。
      </Typography.Paragraph>

      <Input.TextArea
        value={sampleText}
        onChange={(e) => setSampleText(e.target.value)}
        placeholder={`例如：\n患者张三，电话13812345678，身份证320102199001011234`}
        rows={4}
        style={{ marginBottom: 12 }}
      />

      <Button
        type="primary"
        icon={<ThunderboltOutlined />}
        onClick={handleGenerate}
        loading={generating}
        disabled={!sampleText.trim()}
        block
      >
        {generating ? "生成中..." : "生成规则"}
      </Button>

      {generating && (
        <div style={{ textAlign: "center", padding: 24 }}>
          <Spin tip="大模型正在分析样例..." />
        </div>
      )}

      {error && (
        <Alert
          type="error"
          showIcon
          message="生成失败"
          description={error}
          closable
          style={{ marginTop: 12, borderRadius: 8 }}
        />
      )}

      {result && (
        <div
          style={{
            marginTop: 16,
            padding: 16,
            background: "#f6ffed",
            border: "1px solid #b7eb8f",
            borderRadius: 8,
          }}
        >
          <Typography.Text strong style={{ color: "#52c41a", marginBottom: 12, display: "block" }}>
            生成完成
          </Typography.Text>
          <Descriptions column={1} size="small" bordered>
            <Descriptions.Item label="规则名">
              <Typography.Text strong>{result.name}</Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label="正则">
              <Typography.Text code copyable>
                {result.pattern}
              </Typography.Text>
            </Descriptions.Item>
            <Descriptions.Item label="策略">
              <Tag color={result.strategy === "placeholder" ? "blue" : "purple"}>
                {result.strategy}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="模式">
              <Tag color={result.mode === "filter" ? "green" : "orange"}>
                {result.mode}
              </Tag>
            </Descriptions.Item>
            <Descriptions.Item label="优先级">
              {result.priority}
            </Descriptions.Item>
          </Descriptions>
        </div>
      )}
    </Modal>
  );
}
