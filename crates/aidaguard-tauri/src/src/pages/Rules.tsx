import { useEffect, useState } from "react";
import {
  Card,
  Table,
  Tag,
  Switch,
  Space,
  Button,
  Input,
  Select,
  Popconfirm,
  message,
  theme,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ReloadOutlined,
  ExperimentOutlined,
} from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import { useRulesStore } from "../store/useRulesStore";
import RuleEditor from "../components/RuleEditor";
import RuleTestPanel from "../components/RuleTestPanel";
import type { RuleWithCategory } from "../api/rules";
import type { RuleDef } from "../types";

export default function Rules() {
  const { token } = theme.useToken();
  const rules = useRulesStore((s) => s.rules);
  const ruleFiles = useRulesStore((s) => s.ruleFiles);
  const loading = useRulesStore((s) => s.loading);
  const testing = useRulesStore((s) => s.testing);
  const testResult = useRulesStore((s) => s.testResult);
  const fetchRules = useRulesStore((s) => s.fetchRules);
  const save = useRulesStore((s) => s.saveRule);
  const remove = useRulesStore((s) => s.deleteRule);
  const toggle = useRulesStore((s) => s.toggleRule);
  const test = useRulesStore((s) => s.testRule);
  const reload = useRulesStore((s) => s.reloadRules);
  const clearTestResult = useRulesStore((s) => s.clearTestResult);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<RuleWithCategory | null>(null);
  const [testOpen, setTestOpen] = useState(false);
  const [filterCat, setFilterCat] = useState<string>("");
  const [searchText, setSearchText] = useState("");

  useEffect(() => {
    fetchRules();
  }, []);

  const filtered = rules.filter((r) => {
    if (filterCat && r.category !== filterCat) return false;
    if (searchText && !r.id.includes(searchText) && !r.name.includes(searchText))
      return false;
    return true;
  });

  const handleSave = async (rule: RuleDef, category: string) => {
    try {
      await save(rule, category);
      message.success("规则已保存");
      setEditorOpen(false);
      setEditingRule(null);
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleDelete = async (ruleId: string, category: string) => {
    try {
      await remove(ruleId, category);
      message.success("规则已删除");
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const columns: ColumnsType<RuleWithCategory> = [
    {
      title: "启用",
      dataIndex: "enabled",
      key: "enabled",
      width: 60,
      render: (val: boolean, record) => (
        <Switch
          size="small"
          checked={val}
          onChange={(checked) => {
            toggle(record.id, checked);
            fetchRules();
          }}
        />
      ),
    },
    {
      title: "规则名",
      dataIndex: "name",
      key: "name",
      width: 140,
    },
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
      width: 150,
      render: (v: string) => <Tag>{v}</Tag>,
    },
    {
      title: "正则",
      dataIndex: "pattern",
      key: "pattern",
      ellipsis: true,
      render: (v: string) => (
        <code style={{ fontSize: 12 }}>{v}</code>
      ),
    },
    {
      title: "策略",
      dataIndex: "strategy",
      key: "strategy",
      width: 100,
      render: (v: string) => (
        <Tag color={v === "placeholder" ? "blue" : "purple"}>{v}</Tag>
      ),
    },
    {
      title: "优先级",
      dataIndex: "priority",
      key: "priority",
      width: 70,
    },
    {
      title: "分类",
      dataIndex: "category",
      key: "category",
      width: 90,
      render: (v: string) => <Tag color="green">{v}</Tag>,
    },
    {
      title: "操作",
      key: "actions",
      width: 130,
      render: (_, record) => (
        <Space size={4}>
          <Button
            type="link"
            size="small"
            icon={<EditOutlined />}
            onClick={() => {
              setEditingRule(record);
              setEditorOpen(true);
            }}
          />
          <Button
            type="link"
            size="small"
            icon={<ExperimentOutlined />}
            onClick={() => {
              setTestOpen(true);
              clearTestResult();
            }}
          />
          <Popconfirm
            title="确定删除此规则？"
            onConfirm={() => handleDelete(record.id, record.category)}
            okText="删除"
            cancelText="取消"
          >
            <Button
              type="link"
              size="small"
              danger
              icon={<DeleteOutlined />}
            />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <div>
      <Card
        size="small"
        style={{
          borderRadius: 12,
          border: `1px solid ${token.colorBorderSecondary}`,
        }}
      >
        {/* Toolbar */}
        <Space
          wrap
          style={{ marginBottom: 16, width: "100%", justifyContent: "space-between" }}
        >
          <Space wrap>
            <Select
              placeholder="分类筛选"
              allowClear
              style={{ width: 140 }}
              value={filterCat || undefined}
              onChange={(v) => setFilterCat(v || "")}
              options={ruleFiles.map((f) => ({ value: f, label: f }))}
            />
            <Input.Search
              placeholder="搜索规则名/ID"
              style={{ width: 220 }}
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              allowClear
            />
            <Button icon={<ReloadOutlined />} onClick={reload}>
              重载规则
            </Button>
          </Space>
          <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={() => {
              setEditingRule(null);
              setEditorOpen(true);
            }}
          >
            添加规则
          </Button>
        </Space>

        <Table
          columns={columns}
          dataSource={filtered}
          rowKey="id"
          loading={loading}
          size="small"
          pagination={false}
        />
      </Card>

      <RuleEditor
        open={editorOpen}
        editing={editingRule}
        ruleFiles={ruleFiles}
        onSave={handleSave}
        onCancel={() => {
          setEditorOpen(false);
          setEditingRule(null);
        }}
      />

      <RuleTestPanel
        open={testOpen}
        testing={testing}
        result={testResult}
        onTest={test}
        onClose={() => {
          setTestOpen(false);
          clearTestResult();
        }}
      />
    </div>
  );
}
