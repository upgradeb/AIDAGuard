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
  Modal,
  Typography,
  message,
  theme,
  Alert,
} from "antd";
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  ReloadOutlined,
  ExperimentOutlined,
  FolderOpenOutlined,
  SettingOutlined,
  RobotOutlined,
} from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import { useRulesStore } from "../store/useRulesStore";
import RuleEditor from "../components/RuleEditor";
import RuleTestPanel from "../components/RuleTestPanel";
import GenerateRuleModal from "../components/GenerateRuleModal";
import type { RuleWithCategory } from "../api/rules";
import type { RuleDef } from "../types";

function groupByCategory(rules: RuleWithCategory[]): Map<string, RuleWithCategory[]> {
  const map = new Map<string, RuleWithCategory[]>();
  for (const r of rules) {
    const cat = r.category || "未分类";
    if (!map.has(cat)) map.set(cat, []);
    map.get(cat)!.push(r);
  }
  return map;
}

export default function Rules() {
  const { token } = theme.useToken();
  const rules = useRulesStore((s) => s.rules);
  const ruleFiles = useRulesStore((s) => s.ruleFiles);
  const rulesDir = useRulesStore((s) => s.rulesDir);
  const error = useRulesStore((s) => s.error);
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
  const createCat = useRulesStore((s) => s.createCategory);
  const deleteCat = useRulesStore((s) => s.deleteCategory);
  const renameCat = useRulesStore((s) => s.renameCategory);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<RuleWithCategory | null>(null);
  const [testOpen, setTestOpen] = useState(false);
  const [generateOpen, setGenerateOpen] = useState(false);
  const [filterCat, setFilterCat] = useState<string>("");
  const [searchText, setSearchText] = useState("");
  const [catModalOpen, setCatModalOpen] = useState(false);
  const [newCatName, setNewCatName] = useState("");
  const [renameTarget, setRenameTarget] = useState<string | null>(null);
  const [renameNewName, setRenameNewName] = useState("");

  useEffect(() => {
    fetchRules();
  }, []);

  const filtered = rules.filter((r) => {
    if (filterCat && r.category !== filterCat) return false;
    if (searchText && !r.id.includes(searchText) && !r.name.includes(searchText))
      return false;
    return true;
  });

  const grouped = groupByCategory(filtered);
  const categories = Array.from(grouped.keys()).sort();

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

  const handleToggleMode = async (record: RuleWithCategory) => {
    const newMode = record.mode === "filter" ? "detect" : "filter";
    const updated: RuleDef = {
      id: record.id,
      name: record.name,
      pattern: record.pattern,
      strategy: record.strategy,
      mode: newMode,
      priority: record.priority,
      enabled: record.enabled,
    };
    try {
      await save(updated, record.category);
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleCreateCategory = async () => {
    const name = newCatName.trim();
    if (!name) return;
    try {
      await createCat(name);
      message.success(`分类 ${name} 已创建`);
      setNewCatName("");
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleDeleteCategory = async (name: string) => {
    try {
      await deleteCat(name);
      message.success(`分类 ${name} 已删除`);
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRenameCategory = async () => {
    if (!renameTarget) return;
    const newName = renameNewName.trim();
    if (!newName) return;
    try {
      await renameCat(renameTarget, newName);
      message.success(`已重命名为 ${newName}`);
      setRenameTarget(null);
      setRenameNewName("");
      fetchRules();
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleBulkToggleEnabled = async (category: string, enabled: boolean) => {
    const catRules = rules.filter((r) => r.category === category);
    for (const r of catRules) {
      const updated: RuleDef = {
        id: r.id, name: r.name, pattern: r.pattern,
        strategy: r.strategy, mode: r.mode, priority: r.priority,
        enabled,
      };
      try { await save(updated, category); } catch { /* continue */ }
    }
    fetchRules();
  };

  const handleBulkToggleMode = async (category: string, mode: "detect" | "filter") => {
    const catRules = rules.filter((r) => r.category === category);
    for (const r of catRules) {
      const updated: RuleDef = {
        id: r.id, name: r.name, pattern: r.pattern,
        strategy: r.strategy, mode, priority: r.priority,
        enabled: r.enabled,
      };
      try { await save(updated, category); } catch { /* continue */ }
    }
    fetchRules();
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
      title: "模式",
      dataIndex: "mode",
      key: "mode",
      width: 80,
      render: (val: string, record) => (
        <Switch
          size="small"
          checked={val === "filter"}
          checkedChildren="过滤"
          unCheckedChildren="检测"
          onChange={() => handleToggleMode(record)}
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
      render: (v: string) => <code style={{ fontSize: 12 }}>{v}</code>,
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
    <div style={{ height: "100%", display: "flex", flexDirection: "column", gap: 12 }}>
      {/* 顶部信息栏 + 工具栏 */}
      <div style={{ flexShrink: 0 }}>
        {error && (
          <Alert
            type="error"
            showIcon
            message="规则加载失败"
            description={error}
            closable
            style={{ marginBottom: 12, borderRadius: 8 }}
          />
        )}

        {!loading && !error && rules.length === 0 && (
          <Alert
            type="warning"
            showIcon
            message="未发现规则文件"
            description={
              <span>
                规则目录：<code>{rulesDir || "未知"}</code>
                。请确保目录下存在 <code>.yaml</code> 规则文件，或在「设置」中修改规则文件目录。
              </span>
            }
            style={{ marginBottom: 12, borderRadius: 8 }}
          />
        )}

        {rulesDir && (
          <div
            style={{
              marginBottom: 12,
              padding: "6px 12px",
              borderRadius: 6,
              background: token.colorFillSecondary,
              display: "flex",
              alignItems: "center",
              gap: 16,
              fontSize: 12,
            }}
          >
            <span>
              <FolderOpenOutlined style={{ marginRight: 4 }} />
              规则目录：<code style={{ fontSize: 12 }}>{rulesDir}</code>
            </span>
            <span style={{ color: token.colorTextSecondary }}>
              共 <strong>{rules.length}</strong> 条规则 · <strong>{ruleFiles.length}</strong> 个文件
            </span>
          </div>
        )}

        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            flexWrap: "wrap",
            gap: 8,
            marginBottom: 12,
          }}
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
          <Space>
            <Button
              icon={<SettingOutlined />}
              onClick={() => setCatModalOpen(true)}
            >
              管理分类
            </Button>
            <Button
              icon={<RobotOutlined />}
              onClick={() => setGenerateOpen(true)}
            >
              生成规则
            </Button>
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
        </div>
      </div>

      {/* 规则明细 — 仅此区域滚动 */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          minHeight: 0,
        }}
      >
        {categories.map((cat) => {
          const catRules = grouped.get(cat)!;
          const allEnabled = catRules.every((r) => r.enabled);
          const allFilter = catRules.every((r) => r.mode === "filter");
          return (
            <Card
              key={cat}
              size="small"
              title={
                <Space>
                  <Tag color="green">{cat}</Tag>
                  <span style={{ fontSize: 12, color: token.colorTextSecondary }}>
                    {catRules.length} 条规则
                  </span>
                </Space>
              }
              extra={
                <Space size={8}>
                  <Space size={2}>
                    <Typography.Text style={{ fontSize: 11, color: token.colorTextQuaternary }}>启用</Typography.Text>
                    <Switch
                      size="small"
                      checked={allEnabled}
                      onChange={(v) => handleBulkToggleEnabled(cat, v)}
                    />
                  </Space>
                  <Space size={2}>
                    <Typography.Text style={{ fontSize: 11, color: token.colorTextQuaternary }}>过滤</Typography.Text>
                    <Switch
                      size="small"
                      checked={allFilter}
                      checkedChildren="过滤"
                      unCheckedChildren="检测"
                      onChange={(v) => handleBulkToggleMode(cat, v ? "filter" : "detect")}
                    />
                  </Space>
                  <Button
                    type="link"
                    size="small"
                    style={{ fontSize: 12 }}
                    onClick={() => {
                      setRenameTarget(cat);
                      setRenameNewName(cat);
                    }}
                  >
                    重命名
                  </Button>
                  <Popconfirm
                    title={`确定删除分类 ${cat}？将同时删除该分类下的所有规则。`}
                    onConfirm={() => handleDeleteCategory(cat)}
                    okText="删除"
                    cancelText="取消"
                  >
                    <Button
                      type="link"
                      size="small"
                      danger
                      style={{ fontSize: 12 }}
                    >
                      删除
                    </Button>
                  </Popconfirm>
                </Space>
              }
              style={{ marginBottom: 12, borderRadius: 8 }}
              styles={{ body: { padding: 0 } }}
            >
              <Table
                columns={columns}
                dataSource={catRules}
                rowKey="id"
                loading={loading}
                size="small"
                pagination={false}
              />
            </Card>
          );
        })}

        {filtered.length === 0 && !loading && (
          <Alert
            type="info"
            message="无匹配规则"
            style={{ borderRadius: 8 }}
          />
        )}
      </div>

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

      <GenerateRuleModal
        open={generateOpen}
        onApply={(rule) => {
          setEditingRule({
            id: "",
            name: rule.name,
            pattern: rule.pattern,
            strategy: rule.strategy as "placeholder" | "mask",
            mode: rule.mode as "detect" | "filter",
            priority: rule.priority,
            enabled: true,
            category: ruleFiles[0] || "custom",
          });
          setEditorOpen(true);
        }}
        onClose={() => setGenerateOpen(false)}
      />

      {/* 分类管理弹窗 */}
      <Modal
        title="管理分类"
        open={catModalOpen}
        onCancel={() => {
          setCatModalOpen(false);
          setNewCatName("");
        }}
        footer={null}
        width={480}
      >
        <Space direction="vertical" style={{ width: "100%" }} size={16}>
          <div>
            <Space.Compact style={{ width: "100%" }}>
              <Input
                placeholder="输入新分类名（字母、数字、_、-）"
                value={newCatName}
                onChange={(e) => setNewCatName(e.target.value)}
                onPressEnter={handleCreateCategory}
              />
              <Button type="primary" onClick={handleCreateCategory}>
                创建
              </Button>
            </Space.Compact>
          </div>

          <div>
            <Typography.Text strong style={{ display: "block", marginBottom: 8 }}>
              现有分类
            </Typography.Text>
            {ruleFiles.map((f) => {
              const catRules = rules.filter((r) => r.category === f);
              return (
                <div
                  key={f}
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: "8px 0",
                    borderBottom: "1px solid #f0f0f0",
                  }}
                >
                  <Space>
                    <Tag color="green">{f}</Tag>
                    <span style={{ fontSize: 12, color: "#999" }}>
                      {catRules.length} 条规则
                    </span>
                  </Space>
                  <Popconfirm
                    title={`确定删除分类 ${f}？将同时删除该分类下的所有规则。`}
                    onConfirm={() => handleDeleteCategory(f)}
                    okText="删除"
                    cancelText="取消"
                  >
                    <Button type="link" size="small" danger>
                      <DeleteOutlined />
                    </Button>
                  </Popconfirm>
                </div>
              );
            })}
            {ruleFiles.length === 0 && (
              <Typography.Text type="secondary">暂无分类</Typography.Text>
            )}
          </div>
        </Space>
      </Modal>

      {/* 重命名分类弹窗 */}
      <Modal
        title={`重命名分类: ${renameTarget || ""}`}
        open={!!renameTarget}
        onOk={handleRenameCategory}
        onCancel={() => {
          setRenameTarget(null);
          setRenameNewName("");
        }}
        okText="保存"
        cancelText="取消"
        width={400}
      >
        <Input
          placeholder="输入新名称"
          value={renameNewName}
          onChange={(e) => setRenameNewName(e.target.value)}
          onPressEnter={handleRenameCategory}
        />
      </Modal>
    </div>
  );
}
