import { useState } from "react";
import { Drawer, Input, Button, Space, Typography, Tag, Card, Divider, theme } from "antd";
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
  const { token } = theme.useToken();

  return (
    <Drawer
      title={t("Rule Test")}
      placement="right"
      width={640}
      open={open}
      onClose={onClose}
    >
      <Space direction="vertical" style={{ width: "100%" }} size={16}>
        <div>
          <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
            {t("Regex Pattern")}
          </Typography.Text>
          <Input
            value={pattern}
            onChange={(e) => setPattern(e.target.value)}
            placeholder={t("e.g. 1[3-9]\\d{9}")}
          />
        </div>

        <div>
          <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
            {t("Test Text")}
          </Typography.Text>
          <Input.TextArea
            rows={5}
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t("Enter test text containing sensitive data...")}
          />
        </div>

        <Button
          type="primary"
          icon={<PlayCircleOutlined />}
          onClick={() => onTest(pattern, text)}
          loading={testing}
          disabled={!pattern || !text}
        >
          {t("Run Test")}
        </Button>

        {result && (
          <>
            <Divider />

            <Card size="small" title={t("Matches: {{count}}", { count: result.matches.length })}>
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
                      {m.mode === "filter" ? t("Filter") : t("Detect")}
                    </Tag>
                  </Space>
                </div>
              ))}
              {result.matches.length === 0 && (
                <Typography.Text type="secondary">{t("No Matches")}</Typography.Text>
              )}
            </Card>

            <Card size="small" title={t("Sanitized Text")}>
              <pre
                style={{
                  background: token.colorFillAlter,
                  color: token.colorText,
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
