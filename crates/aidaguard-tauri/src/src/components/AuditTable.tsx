import { Table, Tag, Typography, Space, Button, Popconfirm, Badge } from "antd";
import { EyeOutlined, DeleteOutlined } from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import type { ColumnsType } from "antd/es/table";
import type { AuditGroup, DetectionRecord } from "../types";
import dayjs from "dayjs";

interface AuditTableProps {
  groups: AuditGroup[];
  groupTotal: number;
  loading: boolean;
  page: number;
  pageSize: number;
  expandedRecords: Record<string, DetectionRecord[]>;
  expandedLoading: Record<string, boolean>;
  onPageChange: (page: number, pageSize: number) => void;
  onExpand: (ruleId: string, strategy: string) => void;
  onViewDetail: (id: string) => void;
  onDelete: (id: string, ruleId: string, strategy: string) => void;
}

const groupKey = (g: AuditGroup) => `${g.ruleId}:${g.strategy}`;

export default function AuditTable({
  groups,
  groupTotal,
  loading,
  page,
  pageSize,
  expandedRecords,
  expandedLoading,
  onPageChange,
  onExpand,
  onViewDetail,
  onDelete,
}: AuditTableProps) {
  const { t } = useTranslation();

  const groupColumns: ColumnsType<AuditGroup> = [
    {
      title: t("最新时间"),
      dataIndex: "latestTimestampMs",
      key: "latestTime",
      width: 170,
      render: (val: number) => dayjs(val).format("YYYY-MM-DD HH:mm:ss"),
    },
    {
      title: t("审计策略"),
      dataIndex: "strategy",
      key: "strategy",
      width: 100,
      render: (val: string) => {
        if (val === "detect") return <Tag color="orange">{t("仅检测")}</Tag>;
        if (val === "mask") return <Tag color="purple">{t("部分掩码")}</Tag>;
        return <Tag color="blue">{t("占位符替换")}</Tag>;
      },
    },
    {
      title: t("规则名"),
      dataIndex: "ruleName",
      key: "rule",
      width: 160,
      render: (val: string, record) => (
        <Space size={8}>
          <Tag color="orange">{val || record.ruleId}</Tag>
          <Badge count={record.count} overflowCount={999} color="#3b82f6" />
        </Space>
      ),
    },
    {
      title: t("命中次数"),
      dataIndex: "count",
      key: "count",
      width: 80,
      align: "right",
      render: (val: number) => (
        <Typography.Text strong style={{ fontSize: 14 }}>{val}</Typography.Text>
      ),
    },
  ];

  const recordColumns: ColumnsType<DetectionRecord> = [
    {
      title: t("时间"),
      dataIndex: "timestampMs",
      key: "time",
      width: 150,
      render: (val: number) => dayjs(val).format("YYYY-MM-DD HH:mm:ss"),
    },
    {
      title: t("原始数据"),
      dataIndex: "original",
      key: "original",
      width: 140,
      ellipsis: true,
      render: (val: string) => (
        <Typography.Text style={{ color: "#ef4444", fontSize: 13 }} ellipsis copyable>
          {val || "—"}
        </Typography.Text>
      ),
    },
    {
      title: t("占位符"),
      dataIndex: "placeholder",
      key: "placeholder",
      width: 160,
      ellipsis: true,
      render: (val: string) => (
        <Typography.Text code style={{ fontSize: 12 }} ellipsis>
          {val || "—"}
        </Typography.Text>
      ),
    },
    {
      title: t("工具名"),
      dataIndex: "toolName",
      key: "tool",
      width: 100,
      ellipsis: true,
      render: (val: string) =>
        val ? <Tag color="geekblue">{val}</Tag> : <Typography.Text type="secondary">—</Typography.Text>,
    },
    {
      title: t("模型"),
      dataIndex: "requestPath",
      key: "path",
      width: 140,
      ellipsis: true,
      render: (val: string) => (
        <Typography.Text style={{ fontSize: 13 }} ellipsis>
          {val || "—"}
        </Typography.Text>
      ),
    },
    {
      title: t("操作"),
      key: "actions",
      width: 90,
      render: (_, record) => (
        <Space size={4}>
          <Button
            type="link"
            size="small"
            icon={<EyeOutlined />}
            onClick={() => onViewDetail(record.id)}
          />
          <Popconfirm
            title={t("确定删除此记录？")}
            onConfirm={() => onDelete(record.id, record.ruleId, record.strategy)}
            okText={t("删除")}
            cancelText={t("取消")}
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Table
      columns={groupColumns}
      dataSource={groups}
      rowKey={groupKey}
      loading={loading}
      size="small"
      scroll={{ x: "max-content", y: "calc(100vh - 300px)" }}
      expandable={{
        expandedRowRender: (group) => {
          const key = groupKey(group);
          const records = expandedRecords[key];
          const isLoading = expandedLoading[key];
          return (
            <Table
              columns={recordColumns}
              dataSource={records || []}
              rowKey="id"
              loading={isLoading}
              size="small"
              pagination={false}
              scroll={{ x: "max-content" }}
              style={{ margin: "-8px 0", background: "#fafafa", borderRadius: 6 }}
            />
          );
        },
        onExpand: (expanded, group) => {
          if (expanded) {
            onExpand(group.ruleId, group.strategy);
          }
        },
      }}
      pagination={{
        current: page,
        pageSize,
        total: groupTotal,
        showSizeChanger: true,
        pageSizeOptions: ["10", "20", "50"],
        onChange: onPageChange,
        showTotal: (total) => t("共 {{count}} 组", { count: total }),
      }}
    />
  );
}
