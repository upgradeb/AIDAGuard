import { Modal, Form, Input, Select, InputNumber, Switch } from "antd";
import type { RuleDef } from "../types";
import type { RuleWithCategory } from "../api/rules";

interface RuleEditorProps {
  open: boolean;
  editing: RuleWithCategory | null;
  ruleFiles: string[];
  onSave: (rule: RuleDef, category: string) => Promise<void>;
  onCancel: () => void;
}

export default function RuleEditor({
  open,
  editing,
  ruleFiles,
  onSave,
  onCancel,
}: RuleEditorProps) {
  const [form] = Form.useForm();
  const isEdit = !!editing;

  // Reset form when opening
  const handleOpen = () => {
    if (editing) {
      form.setFieldsValue({
        id: editing.id,
        name: editing.name,
        pattern: editing.pattern,
        strategy: editing.strategy,
        priority: editing.priority,
        enabled: editing.enabled,
        category: editing.category,
      });
    } else {
      form.resetFields();
      form.setFieldsValue({
        strategy: "placeholder",
        priority: 100,
        enabled: true,
        category: ruleFiles[0] || "custom",
      });
    }
  };

  const handleOk = async () => {
    const values = await form.validateFields();
    const rule: RuleDef = {
      id: values.id,
      name: values.name,
      pattern: values.pattern,
      strategy: values.strategy,
      priority: values.priority,
      enabled: values.enabled,
    };
    await onSave(rule, values.category);
    form.resetFields();
  };

  return (
    <Modal
      title={isEdit ? "编辑规则" : "添加规则"}
      open={open}
      onOk={handleOk}
      onCancel={onCancel}
      afterOpenChange={(visible) => { if (visible) handleOpen(); }}
      okText="保存"
      cancelText="取消"
      width={560}
    >
      <Form form={form} layout="vertical" style={{ marginTop: 16 }}>
        <Form.Item
          name="id"
          label="规则 ID"
          rules={[
            { required: true, message: "请输入规则 ID" },
            { pattern: /^[a-z0-9_]+$/, message: "仅支持小写字母、数字和下划线" },
          ]}
        >
          <Input placeholder="如 phone_cn" disabled={isEdit} />
        </Form.Item>
        <Form.Item
          name="name"
          label="名称"
          rules={[{ required: true, message: "请输入规则名称" }]}
        >
          <Input placeholder="如 中国手机号" />
        </Form.Item>
        <Form.Item
          name="pattern"
          label="正则表达式"
          rules={[{ required: true, message: "请输入正则表达式" }]}
        >
          <Input.TextArea rows={3} placeholder="如 1[3-9]\d{9}" />
        </Form.Item>
        <Form.Item name="strategy" label="策略">
          <Select
            options={[
              { value: "placeholder", label: "Placeholder — 整体替换为占位符" },
              { value: "mask", label: "Mask — 部分掩码" },
            ]}
          />
        </Form.Item>
        <Form.Item name="priority" label="优先级">
          <InputNumber min={1} max={999} />
        </Form.Item>
        <Form.Item name="enabled" label="启用" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item
          name="category"
          label="分类"
          rules={[{ required: true, message: "请选择分类" }]}
        >
          <Select
            options={ruleFiles.map((f) => ({ value: f, label: f }))}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}
