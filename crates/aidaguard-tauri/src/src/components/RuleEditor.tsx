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
      title={isEdit ? t("Edit Rule") : t("Add Rule")}
      open={open}
      onOk={handleOk}
      onCancel={onCancel}
      afterOpenChange={(visible) => { if (visible) handleOpen(); }}
      okText={t("Save")}
      cancelText={t("Cancel")}
      width={560}
    >
      <Form
        form={form}
        layout="vertical"
        style={{ marginTop: 16, maxHeight: "60vh", overflow: "auto", paddingRight: 8 }}
      >
        <Form.Item
          name="id"
          label={t("Rule ID")}
          rules={[
            { required: true, message: t("Please enter Rule ID") },
            { pattern: /^[a-z0-9_]+$/, message: t("Only lowercase letters, digits and underscores") },
          ]}
        >
          <Input placeholder={t("e.g. phone_cn")} />
        </Form.Item>
        <Form.Item
          name="name"
          label={t("Name")}
          rules={[{ required: true, message: t("Please enter Rule Name") }]}
        >
          <Input placeholder={t("e.g. Chinese Phone Number")} />
        </Form.Item>
        <Form.Item
          name="pattern"
          label={t("Regex Pattern")}
          rules={[{ required: true, message: t("Please enter Regex Pattern") }]}
        >
          <Input.TextArea rows={3} placeholder={t("e.g. 1[3-9]\\d{9}")} />
        </Form.Item>
        <Form.Item name="strategy" label={t("Strategy")}>
          <Select
            options={[
              { value: "placeholder", label: t("Placeholder — Replace Entire Match") },
              { value: "mask", label: t("Mask — Partial Masking") },
            ]}
          />
        </Form.Item>
        <Form.Item name="mode" label={t("Mode")}>
          <Select
            options={[
              { value: "filter", label: t("Filter — Detect and Replace") },
              { value: "detect", label: t("Detect — Log Only, No Replacement") },
            ]}
          />
        </Form.Item>
        <Form.Item name="priority" label={t("Priority")}>
          <InputNumber min={1} max={999} />
        </Form.Item>
        <Form.Item name="enabled" label={t("Enable")} valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item
          name="category"
          label={t("Category")}
          rules={[{ required: true, message: t("Please Select Category") }]}
        >
          <Select
            options={ruleFiles.map((f) => ({ value: f, label: f }))}
          />
        </Form.Item>
      </Form>
    </Modal>
  );
}
