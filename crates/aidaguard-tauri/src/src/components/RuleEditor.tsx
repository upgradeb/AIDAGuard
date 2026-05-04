import { Modal, Form, Input, Select, InputNumber, Switch } from "antd";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
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
        mode: editing.mode || "filter",
        priority: editing.priority,
        enabled: editing.enabled,
        category: editing.category,
      });
    } else {
      form.resetFields();
      form.setFieldsValue({
        strategy: "placeholder",
        mode: "filter",
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
      mode: values.mode,
      priority: values.priority,
      enabled: values.enabled,
    };
    await onSave(rule, values.category);
    form.resetFields();
  };

  return (
    <Modal
      title={isEdit ? t("编辑规则") : t("添加规则")}
      open={open}
      onOk={handleOk}
      onCancel={onCancel}
      afterOpenChange={(visible) => { if (visible) handleOpen(); }}
      okText={t("保存")}
      cancelText={t("取消")}
      width={560}
    >
      <Form
        form={form}
        layout="vertical"
        style={{ marginTop: 16, maxHeight: "60vh", overflow: "auto", paddingRight: 8 }}
      >
        <Form.Item
          name="id"
          label={t("规则 ID")}
          rules={[
            { required: true, message: t("请输入规则 ID") },
            { pattern: /^[a-z0-9_]+$/, message: t("仅支持小写字母、数字和下划线") },
          ]}
        >
          <Input placeholder={t("如 phone_cn")} />
        </Form.Item>
        <Form.Item
          name="name"
          label={t("名称")}
          rules={[{ required: true, message: t("请输入规则名称") }]}
        >
          <Input placeholder={t("如 中国手机号")} />
        </Form.Item>
        <Form.Item
          name="pattern"
          label={t("正则表达式")}
          rules={[{ required: true, message: t("请输入正则表达式") }]}
        >
          <Input.TextArea rows={3} placeholder={t("如 1[3-9]\\d{9}")} />
        </Form.Item>
        <Form.Item name="strategy" label={t("策略")}>
          <Select
            options={[
              { value: "placeholder", label: t("Placeholder — 整体替换为占位符") },
              { value: "mask", label: t("Mask — 部分掩码") },
            ]}
          />
        </Form.Item>
        <Form.Item name="mode" label={t("模式")}>
          <Select
            options={[
              { value: "filter", label: t("过滤 — 检测并替换为占位符") },
              { value: "detect", label: t("检测 — 仅记录，不替换") },
            ]}
          />
        </Form.Item>
        <Form.Item name="priority" label={t("优先级")}>
          <InputNumber min={1} max={999} />
        </Form.Item>
        <Form.Item name="enabled" label={t("启用")} valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item
          name="category"
          label={t("分类")}
          rules={[{ required: true, message: t("请选择分类") }]}
        >
          <Select
            options={ruleFiles.map((f) => ({ value: f, label: f }))}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}
