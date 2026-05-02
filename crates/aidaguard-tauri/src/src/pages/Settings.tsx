import { useEffect } from "react";
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
import ThemeSwitcher from "../components/ThemeSwitcher";
import type { Config } from "../types";

export default function Settings() {
  const { token } = theme.useToken();
  const config = useConfigStore((s) => s.config);
  const saving = useConfigStore((s) => s.saving);
  const fetchConfig = useConfigStore((s) => s.fetchConfig);
  const save = useConfigStore((s) => s.saveConfig);

  const [form] = Form.useForm<Config>();

  useEffect(() => {
    fetchConfig();
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
      message.success("配置已保存");
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
    <div style={{ maxWidth: 720 }}>
      <Form
        form={form}
        layout="vertical"
        initialValues={config || undefined}
      >
        {/* 代理设置 */}
        <Card title="代理设置" size="small" style={cardStyle}>
          <Form.Item
            name="port"
            label="监听端口"
            rules={[{ required: true }]}
            extra="默认 19000，修改后需重启代理"
          >
            <InputNumber min={1024} max={65535} style={{ width: 200 }} />
          </Form.Item>
          <Form.Item
            name="target_url"
            label="上游 LLM API 地址"
            rules={[{ required: true }]}
          >
            <Input placeholder="https://qianfan.baidubce.com/v2/coding" />
          </Form.Item>
          <Form.Item
            name="api_key"
            label="API Key"
          >
            <Input.Password placeholder="sk-..." />
          </Form.Item>
          <Form.Item
            name="max_body_size_mb"
            label="请求体大小限制 (MB)"
          >
            <InputNumber min={1} max={100} style={{ width: 200 }} />
          </Form.Item>
        </Card>

        {/* 存储设置 */}
        <Card title="存储设置" size="small" style={cardStyle}>
          <Form.Item
            name={["storage", "enabled"]}
            label="启用审计记录"
            valuePropName="checked"
            extra="开启后敏感数据检测记录将被持久化存储"
          >
            <Switch />
          </Form.Item>
          <Form.Item
            name={["storage", "db_path"]}
            label="数据库文件路径"
          >
            <Input placeholder="./data/aidaguard.db" />
          </Form.Item>
          <Form.Item
            name={["storage", "encryption_key"]}
            label="加密密钥"
            extra="用于加密存储的敏感数据原文"
          >
            <Input.Password placeholder="留空使用内置默认密钥" />
          </Form.Item>
        </Card>

        {/* 日志设置 */}
        <Card title="日志设置" size="small" style={cardStyle}>
          <Form.Item name="log_level" label="日志级别">
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

        {/* 外观 */}
        <Card title="外观" size="small" style={cardStyle}>
          <div style={{ marginBottom: 8 }}>
            <ThemeSwitcher />
          </div>
        </Card>

        {/* 关于 */}
        <Card title="关于" size="small" style={cardStyle}>
          <Descriptions column={1} size="small">
            <Descriptions.Item label="产品">Aidaguard</Descriptions.Item>
            <Descriptions.Item label="版本">0.1.0</Descriptions.Item>
            <Descriptions.Item label="许可证">MIT</Descriptions.Item>
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
          保存设置
        </Button>
      </Form>
    </div>
  );
}
