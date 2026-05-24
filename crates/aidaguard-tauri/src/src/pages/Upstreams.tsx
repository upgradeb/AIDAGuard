import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Card,
  Table,
  Tag,
  Switch,
  Space,
  Button,
  Input,
  Modal,
  Form,
  InputNumber,
  Popconfirm,
  message,
  theme,
  Alert,
  Typography,
  Radio,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ApiOutlined,
} from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import { useUpstreamStore } from "../store/useUpstreamStore";
import type { UpstreamConfig } from "../types";

export default function Upstreams() {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const upstreams = useUpstreamStore((s) => s.upstreams);
  const loading = useUpstreamStore((s) => s.loading);
  const saving = useUpstreamStore((s) => s.saving);
  const testing = useUpstreamStore((s) => s.testing);
  const testResult = useUpstreamStore((s) => s.testResult);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);
  const add = useUpstreamStore((s) => s.addUpstream);
  const update = useUpstreamStore((s) => s.updateUpstream);
  const remove = useUpstreamStore((s) => s.deleteUpstream);
  const testConn = useUpstreamStore((s) => s.testConnectivity);
  const clearTestResult = useUpstreamStore((s) => s.clearTestResult);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<UpstreamConfig | null>(null);
  const [form] = Form.useForm<UpstreamConfig>();

  useEffect(() => {
    fetchUpstreams();
  }, []);

  useEffect(() => {
    if (editorOpen) {
      if (editingRecord) {
        form.setFieldsValue(editingRecord);
      } else {
        form.resetFields();
      }
    }
  }, [editorOpen]);

  const handleSave = async () => {
    const values = await form.validateFields();
    const upstream: UpstreamConfig = {
      name: values.name,
      url: values.url,
      api_key: values.api_key || undefined,
      default: values.default || false,
      timeout_secs: values.timeout_secs || 300,
      rate_limit_qps: values.rate_limit_qps || 0,
      models: values.models || [],
      protocol: values.protocol || "openai",
    };
    try {
      if (editingRecord) {
        await update(editingRecord.name, upstream);
      } else {
        await add(upstream);
      }
      message.success(t("Upstream Saved"));
      setEditorOpen(false);
      setEditingRecord(null);
      fetchUpstreams();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleDelete = async (name: string) => {
    try {
      await remove(name);
      message.success(t("Upstream Deleted"));
      fetchUpstreams();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleTest = async (record: UpstreamConfig) => {
    await testConn(
      record.name,
      record.url,
      record.api_key || "",
      record.timeout_secs || 10
    );
  };

  const columns: ColumnsType<UpstreamConfig> = [
    {
      title: t("Default"),
      dataIndex: "default",
      key: "default",
      width: 70,
      render: (val: boolean) =>
        val ? <Tag color="blue">{t("Default")}</Tag> : null,
    },
    {
      title: t("Name"),
      dataIndex: "name",
      key: "name",
      width: 120,
    },
    {
      title: t("URL"),
      dataIndex: "url",
      key: "url",
      ellipsis: true,
    },
    {
      title: t("Timeout(s)"),
      dataIndex: "timeout_secs",
      key: "timeout_secs",
      width: 90,
    },
    {
      title: t("QPS"),
      dataIndex: "rate_limit_qps",
      key: "rate_limit_qps",
      width: 90,
      render: (v: number) => (v > 0 ? v : t("Unlimited")),
    },
    {
      title: t("Model"),
      dataIndex: "models",
      key: "models",
      width: 200,
      render: (v: string[]) =>
        v.length > 0
          ? v.map((m) => <Tag key={m}>{m}</Tag>)
          : <span style={{ color: token.colorTextQuaternary }}>{t("Unspecified")}</span>,
    },
    {
      title: t("Protocol"),
      dataIndex: "protocol",
      key: "protocol",
      width: 110,
      render: (val: string) =>
        val === "anthropic" ? <Tag color="orange">Anthropic</Tag> : <Tag color="blue">OpenAI</Tag>,
    },
    {
      title: t("Actions"),
      key: "actions",
      width: 200,
      render: (_, record) => (
        <Space size={4}>
          <Button
            size="small"
            icon={<ApiOutlined />}
            loading={testing === record.name}
            onClick={() => handleTest(record)}
          >
            {t("Test")}
          </Button>
          <Button
            size="small"
            icon={<EditOutlined />}
            onClick={() => {
              setEditingRecord(record);
              setEditorOpen(true);
            }}
          />
          <Popconfirm
            title={t("Delete this upstream?")}
            onConfirm={() => handleDelete(record.name)}
            okText={t("Delete")}
            cancelText={t("Cancel")}
          >
            <Button size="small" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div style={{ maxWidth: 960, height: "100%", overflow: "auto" }}>
      <Card
        size="small"
        style={{
          borderRadius: 12,
          border: `1px solid ${token.colorBorderSecondary}`,
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: 16,
          }}
        >
          <span style={{ color: token.colorTextSecondary, fontSize: 13 }}>
            {t("Manage LLM Upstream Services")}
          </span>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => {
              setEditingRecord(null);
              clearTestResult();
              setEditorOpen(true);
            }}
          >
            {t("Add Upstream")}
          </Button>
        </div>

        <Table
          columns={columns}
          dataSource={upstreams}
          rowKey="name"
          loading={loading}
          size="small"
          pagination={false}
          locale={{ emptyText: t("No Upstreams Configured") }}
        />
      </Card>

      {/* 连接测试结果 */}
      {testResult && (
        <Alert
          type={testResult.startsWith("✓") ? "success" : "error"}
          message={t("Connectivity Test Result")}
          description={
            <Typography.Paragraph
              style={{ margin: 0, whiteSpace: "pre-wrap", fontSize: 12 }}
            >
              {testResult}
            </Typography.Paragraph>
          }
          closable
          onClose={clearTestResult}
          style={{ marginTop: 16, borderRadius: 8 }}
        />
      )}

      {/* 添加/编辑弹窗 */}
      <Modal
        title={editingRecord ? t("Edit Upstream") : t("Add Upstream")}
        open={editorOpen}
        onOk={handleSave}
        onCancel={() => {
          setEditorOpen(false);
          setEditingRecord(null);
        }}
        confirmLoading={saving}
        okText={t("Save")}
        cancelText={t("Cancel")}
        width={560}
      >
        <Form
          form={form}
          layout="vertical"
          style={{ marginTop: 16 }}
          initialValues={{
            timeout_secs: 300,
            rate_limit_qps: 0,
            default: false,
            protocol: "openai",
          }}
        >
          <Form.Item
            name="name"
            label={t("Name")}
            rules={[{ required: true, message: t("Please enter name") }]}
          >
            <Input placeholder={t("e.g. qianfan-pro")} disabled={!!editingRecord} />
          </Form.Item>
          <Form.Item
            name="url"
            label={t("API URL")}
            rules={[{ required: true, message: t("Please enter API URL") }]}
          >
            <Input placeholder="https://qianfan.baidubce.com/v2/coding" />
          </Form.Item>
          <Form.Item name="api_key" label="API Key">
            <Input.Password placeholder={t("Leave empty to skip authentication header")} />
          </Form.Item>
          <Space size={16}>
            <Form.Item name="timeout_secs" label={t("Timeout (s)")}>
              <InputNumber min={1} max={600} style={{ width: 140 }} />
            </Form.Item>
            <Form.Item name="rate_limit_qps" label={t("QPS Limit")}>
              <InputNumber min={0} max={100} style={{ width: 140 }} />
            </Form.Item>
            <Form.Item
              name="default"
              label={t("Set as Default")}
              valuePropName="checked"
            >
              <Switch />
            </Form.Item>
          </Space>
          <Form.Item
            name="protocol"
            label={t("Protocol Type")}
            extra={t("Select the upstream LLM API protocol format")}
          >
            <Radio.Group>
              <Radio.Button value="openai">{t("OpenAI Compatible")}</Radio.Button>
              <Radio.Button value="anthropic">{t("Anthropic Compatible")}</Radio.Button>
            </Radio.Group>
          </Form.Item>
          <Form.Item
            name="models"
            label={t("Model List")}
            extra={t("Separate multiple models with commas")}
            getValueFromEvent={(e) =>
              e.target.value
                .split(",")
                .map((s: string) => s.trim())
                .filter(Boolean)
            }
            getValueProps={(v: string[]) => ({
              value: v ? v.join(", ") : "",
            })}
          >
            <Input.TextArea
              rows={2}
              placeholder="gpt-4, claude-3-opus"
            />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
