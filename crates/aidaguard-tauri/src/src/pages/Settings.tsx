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
  message,
  theme,
} from "antd";
import { SaveOutlined } from "@ant-design/icons";
import { useConfigStore } from "../store/useConfigStore";
import { getAppVersion } from "../api/config";
import ThemeSwitcher from "../components/ThemeSwitcher";
import type { Config } from "../types";

export default function Settings() {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const config = useConfigStore((s) => s.config);
  const saving = useConfigStore((s) => s.saving);
  const fetchConfig = useConfigStore((s) => s.fetchConfig);
  const save = useConfigStore((s) => s.saveConfig);
  const [appVersion, setAppVersion] = useState("");

  const [form] = Form.useForm<Config>();

  useEffect(() => {
    fetchConfig();
    getAppVersion().then(setAppVersion).catch(() => {});
  }, []);

  useEffect(() => {
    if (config) {
      form.setFieldsValue(config);
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

  const cardStyle = {
    borderRadius: 12,
    border: `1px solid ${token.colorBorderSecondary}`,
    marginBottom: 16,
  };

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
