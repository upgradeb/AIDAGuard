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
      message.success(t("配置已保存"));
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
        <Card title={t("代理设置")} size="small" style={cardStyle}>
          <Form.Item
            name="port"
            label={t("监听端口")}
            rules={[{ required: true }]}
            extra={t("默认 19000，修改后需重启代理")}
          >
            <InputNumber min={1024} max={65535} style={{ width: 200 }} />
          </Form.Item>
          <Form.Item
            name="rules_dir"
            label={t("规则文件目录")}
            extra={t("YAML 规则文件存放路径")}
          >
            <Input placeholder="./rules" />
          </Form.Item>
          <Form.Item
            name="max_body_size_mb"
            label={t("请求体大小限制 (MB)")}
          >
            <InputNumber min={1} max={100} style={{ width: 200 }} />
          </Form.Item>
        </Card>

        {/* 存储设置 */}
        <Card title={t("存储设置")} size="small" style={cardStyle}>
          <Form.Item
            name={["storage", "enabled"]}
            label={t("启用审计记录")}
            valuePropName="checked"
            extra={t("开启后敏感数据检测记录将被持久化存储")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["storage", "db_path"]}
            label={t("数据库文件路径")}
          >
            <Input placeholder="./data/aidaguard.db" />
          </Form.Item>
          <Form.Item
            name={["storage", "encryption_key"]}
            label={t("加密密钥")}
            extra={t("用于加密存储的敏感数据原文")}
          >
            <Input.Password placeholder={t("留空使用内置默认密钥")} />
          </Form.Item>
        </Card>

        {/* 日志设置 */}
        <Card title={t("日志设置")} size="small" style={cardStyle}>
          <Form.Item name="log_level" label={t("日志级别")}>
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
        <Card title={t("通知设置")} size="small" style={cardStyle}>
          <Form.Item
            name={["notification", "enabled"]}
            label={t("桌面通知")}
            valuePropName="checked"
            extra={t("检测到敏感数据时发送系统通知，重启代理后生效")}
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["notification", "rate_limit_secs"]}
            label={t("通知间隔 (秒)")}
            extra={t("同一规则最短通知间隔，避免刷屏")}
          >
            <InputNumber min={10} max={600} style={{ width: 200 }} />
          </Form.Item>
        </Card>

        {/* 外观 */}
        <Card title={t("外观")} size="small" style={cardStyle}>
          <div style={{ marginBottom: 8 }}>
            <ThemeSwitcher />
          </div>
        </Card>

        {/* 关于 */}
        <Card title={t("关于")} size="small" style={cardStyle}>
          <Descriptions column={1} size="small">
            <Descriptions.Item label={t("产品")}>Aidaguard</Descriptions.Item>
            <Descriptions.Item label={t("版本")}>{appVersion || "—"}</Descriptions.Item>
            <Descriptions.Item label={t("许可证")}>MIT</Descriptions.Item>
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
          {t("保存设置")}
        </Button>
      </Form>
    </div>
  );
}
