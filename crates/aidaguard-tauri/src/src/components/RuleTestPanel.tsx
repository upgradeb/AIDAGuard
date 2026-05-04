import { useState } from "react";
import { Drawer, Input, Button, Space, Typography, Tag, Card, Divider } from "antd";
import { PlayCircleOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import type { TestRuleResult } from "../types";

interface RuleTestPanelProps {
  open: boolean;
  testing: boolean;
  result: TestRuleResult | null;
  onTest: (pattern: string, text: string) => Promise<void>;
  onClose: () => void;
}

export default function RuleTestPanel({
  open,
  testing,
  result,
  onTest,
  onClose,
}: RuleTestPanelProps) {
  const [pattern, setPattern] = useState("");
  const [text, setText] = useState("");
  const { t } = useTranslation();

  return (
    <Drawer
      title={t("规则测试")}
      placement="right"
      width={640}
      open={open}
      onClose={onClose}
    >
      <Space direction="vertical" style={{ width: "100%" }} size={16}>
        <div>
          <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
            {t("正则表达式")}
          </Typography.Text>
          <Input
            value={pattern}
            onChange={(e) => setPattern(e.target.value)}
            placeholder={t("如 1[3-9]\\d{9}")}
          />
        </div>

        <div>
          <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
            {t("测试文本")}
          </Typography.Text>
          <Input.TextArea
            rows={5}
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t("输入包含敏感数据的测试文本...")}
          />
        </div>

        <Button
          type="primary"
          icon={<PlayCircleOutlined />}
          onClick={() => onTest(pattern, text)}
          loading={testing}
          disabled={!pattern || !text}
        >
          {t("运行测试")}
        </Button>

        {result && (
          <>
            <Divider />

            <Card size="small" title={t("匹配结果: {{count}} 处", { count: result.matches.length })}>
              {result.matches.map((m, i) => (
                <div
                  key={i}
                  style={{
                    padding: "8px 0",
                    borderBottom: "1px solid #f0f0f0",
                  }}
                >
                  <Space wrap size={8}>
                    <Tag color="orange">{m.text}</Tag>
                    <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                      pos {m.start}-{m.end}
                    </Typography.Text>
                    <Tag>{m.strategy}</Tag>
                    <Tag color={m.mode === "filter" ? "blue" : "default"}>
                      {m.mode === "filter" ? t("过滤") : t("检测")}
                    </Tag>
                  </Space>
                </div>
              ))}
              {result.matches.length === 0 && (
                <Typography.Text type="secondary">{t("无匹配")}</Typography.Text>
              )}
            </Card>

            <Card size="small" title={t("替换后文本")}>
              <pre
                style={{
                  background: "#f5f5f5",
                  padding: 12,
                  borderRadius: 6,
                  fontSize: 13,
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-all",
                }}
              >
                {result.sanitizedText}
              </pre>
            </Card>
          </>
        )}
      </Space>
    </Drawer>
  );
}
