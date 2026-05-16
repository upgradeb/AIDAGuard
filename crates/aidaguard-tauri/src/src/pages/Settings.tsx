import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Card,
  Form,
  Input,
  InputNumber,
  Select,
  Switch,
  Button,
  Descriptions,
  Checkbox,
  message,
  theme,
  Alert,
} from "antd";
import { SaveOutlined } from "@ant-design/icons";
import { useConfigStore } from "../store/useConfigStore";
import { getAppVersion } from "../api/config";
import ThemeSwitcher from "../components/ThemeSwitcher";
import type { Config } from "../types";

const REGION_OPTIONS = [
  { value: "global", labelKey: "Global (All Regions)" },
  { value: "cn", labelKey: "China (PIPL)" },
  { value: "us", labelKey: "United States (CCPA/HIPAA)" },
  { value: "eu", labelKey: "European Union (GDPR)" },
  { value: "gb", labelKey: "United Kingdom (UK DPA)" },
];

const INDUSTRIES_BY_REGION: Record<string, string[]> = {
  global: [],
  cn: ["general", "finance", "medical", "personal"],
  us: ["general", "finance", "medical"],
  eu: ["general", "finance"],
  gb: ["general"],
};

export default function Settings() {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const config = useConfigStore((s) => s.config);
  const saving = useConfigStore((s) => s.saving);
  const fetchConfig = useConfigStore((s) => s.fetchConfig);
  const save = useConfigStore((s) => s.saveConfig);
  const [appVersion, setAppVersion] = useState("");
  const [region, setRegion] = useState<string>("global");

  const [form] = Form.useForm<Config>();

  useEffect(() => {
    fetchConfig();
    getAppVersion().then(setAppVersion).catch(() => {});
  }, []);

  useEffect(() => {
    if (config) {
      form.setFieldsValue(config);
      setRegion(config.region || "global");
    }
  }, [config]);

  const handleSave = async () => {
    const values = await form.validateFields();
    try {
      await save(values);
      message.success(t("Configuration Saved"));
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRegionChange = (value: string) => {
    setRegion(value);
    form.setFieldValue("region", value);
    form.setFieldValue("rule_industries", []);
  };

  const cardStyle = {
    borderRadius: 12,
    border: `1px solid ${token.colorBorderSecondary}`,
    marginBottom: 16,
  };

  const industries = INDUSTRIES_BY_REGION[region] || [];

  return (
    <div style={{ height: "100%", overflow: "auto" }}>
      <Form
        form={form}
        layout="vertical"
        initialValues={config || undefined}
      >
        {/* 代理设置 */}
        <Card title={t("Proxy Settings")} size="small" style={cardStyle}>
          <Form.Item
            name="port"
            label={t("Listen Port")}
            rules={[{ required: true }]}
            extra={t("Default 19000. Restart proxy after changing.")}
          >
            <InputNumber min={1024} max={65535} style={{ width: 200 }} />
          </Form.Item>
          <Form.Item
            name="rules_dir"
            label={t("Rules Directory")}
            extra={t("Path to YAML rule files")}
          >
            <Input placeholder="./rules" />
          </Form.Item>
          <Form.Item
            name="max_body_size_mb"
            label={t("Max Request Body (MB)")}
          >
            <InputNumber min={1} max={100} style={{ width: 200 }} />
          </Form.Item>
        </Card>

        {/* 检测策略 */}
        <Card title={t("Detection Policy")} size="small" style={cardStyle}>
          <Form.Item
            name="region"
            label={t("Region / Country")}
            extra={t("Select region or country for applicable detection rules")}
          >
            <Select
              style={{ width: 280 }}
              onChange={handleRegionChange}
              options={REGION_OPTIONS.map((opt) => ({
                value: opt.value,
                label: t(opt.labelKey),
              }))}
            />
          </Form.Item>
          {region === "global" ? (
            <Alert
              type="info"
              showIcon
              message={t("Global baseline rules are always loaded regardless of region selection.")}
              style={{ marginBottom: 16 }}
            />
          ) : (
            <Form.Item
              name="rule_industries"
              label={t("Rule Industries")}
              extra={t("Select industries within the region for domain-specific rules")}
            >
              <Checkbox.Group
                options={industries.map((ind) => ({
                  label: t(ind),
                  value: ind,
                }))}
              />
            </Form.Item>
          )}
        </Card>

        {/* NLP 设置 */}
        <Card title={t("NLP Settings")} size="small" style={cardStyle}>
          <Alert
            type="info"
            showIcon
            message={t("NLP is disabled by default to reduce CPU usage. Enable only when you need to detect unstructured entities like names and addresses.")}
            style={{ marginBottom: 16 }}
          />
          <Form.Item
            name={["nlp", "enabled"]}
            label={t("NER Model")}
            valuePropName="checked"
            extra={t("Enable NLP-based detection of unstructured entities (names, addresses, organizations). Increases CPU usage by ~40%.")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["nlp", "default_language"]}
            label={t("Model Language")}
            extra={t("Select the language for the NER model. The model will be downloaded on first use (~400MB).")}
          >
            <Select
              style={{ width: 180 }}
              options={[
                { value: "en", label: "English (bert-base-NER)" },
                { value: "zh", label: "中文 (bert-base-chinese-ner)" },
              ]}
            />
          </Form.Item>
        </Card>

        {/* 存储设置 */}
        <Card title={t("Storage Settings")} size="small" style={cardStyle}>
          <Form.Item
            name={["storage", "enabled"]}
            label={t("Enable Audit Log")}
            valuePropName="checked"
            extra={t("Sensitive data detection records will be persisted when enabled")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["storage", "db_path"]}
            label={t("Database File Path")}
          >
            <Input placeholder="./data/aidaguard.db" />
          </Form.Item>
          <Form.Item
            name={["storage", "encryption_key"]}
            label={t("Encryption Key")}
            extra={t("Used to encrypt stored sensitive data content")}
          >
            <Input.Password placeholder={t("Leave empty to use built-in default key")} />
          </Form.Item>
        </Card>

        {/* 日志设置 */}
        <Card title={t("Logging Settings")} size="small" style={cardStyle}>
          <Form.Item name="log_level" label={t("Log Level")}>
            <Select
              style={{ width: 160 }}
              options={[
                { value: "trace", label: "trace" },
                { value: "debug", label: "debug" },
                { value: "info", label: "info" },
                { value: "warn", label: "warn" },
                { value: "error", label: "error" },
              ]}
            />
          </Form.Item>
        </Card>

        {/* 通知设置 */}
        <Card title={t("Notification Settings")} size="small" style={cardStyle}>
          <Form.Item
            name={["notification", "enabled"]}
            label={t("Desktop Notifications")}
            valuePropName="checked"
            extra={t("Send system notification when sensitive data is detected. Takes effect after proxy restart.")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["notification", "rate_limit_secs"]}
            label={t("Notification Interval (s)")}
            extra={t("Minimum interval between notifications for the same rule to avoid spam")}
          >
            <InputNumber min={10} max={600} style={{ width: 200 }} />
          </Form.Item>
        </Card>

        {/* 外观 */}
        <Card title={t("Appearance")} size="small" style={cardStyle}>
          <div style={{ marginBottom: 8 }}>
            <ThemeSwitcher />
          </div>
        </Card>

        {/* 关于 */}
        <Card title={t("About")} size="small" style={cardStyle}>
          <Descriptions column={1} size="small">
            <Descriptions.Item label={t("Product")}>Aidaguard</Descriptions.Item>
            <Descriptions.Item label={t("Version")}>{appVersion || "—"}</Descriptions.Item>
            <Descriptions.Item label={t("License")}>MIT</Descriptions.Item>
          </Descriptions>
        </Card>

        <Button
          type="primary"
          icon={<SaveOutlined />}
          size="large"
          onClick={handleSave}
          loading={saving}
          style={{ marginTop: 8 }}
        >
          {t("Save Settings")}
        </Button>
      </Form>
    </div>
  );
}
