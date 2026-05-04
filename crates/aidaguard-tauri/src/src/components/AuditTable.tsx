import { Table, Tag, Typography, Space, Button, Popconfirm } from "antd";
import { EyeOutlined, DeleteOutlined } from "@ant-design/icons";
import type { ColumnsType } from "antd/es/table";
import type { DetectionRecord } from "../types";
import dayjs from "dayjs";

interface AuditTableProps {
  dataSource: DetectionRecord[];
  loading: boolean;
  total: number;
  page: number;
  pageSize: number;
  onPageChange: (page: number, pageSize: number) => void;
  onViewDetail: (id: string) => void;
  onDelete: (id: string) => void;
}

export default function AuditTable({
  dataSource,
  loading,
  total,
  page,
  pageSize,
  onPageChange,
  onViewDetail,
  onDelete,
}: AuditTableProps) {
  const columns: ColumnsType<DetectionRecord> = [
    {
      title: "时间",
      dataIndex: "timestampMs",
      key: "time",
      width: 150,
      render: (val: number) => dayjs(val).format("YYYY-MM-DD HH:mm:ss"),
    },
    {
      title: "工具名",
      dataIndex: "toolName",
      key: "tool",
      width: 100,
      ellipsis: true,
      render: (val: string) =>
        val ? <Tag color="geekblue">{val}</Tag> : <Typography.Text type="secondary">—</Typography.Text>,
    },
    {
      title: "规则名",
      dataIndex: "ruleName",
      key: "rule",
      width: 110,
      render: (val: string, record) => (
        <Tag color="orange">{val || record.ruleId}</Tag>
      ),
    },
    {
      title: "原始数据",
      dataIndex: "original",
      key: "original",
      width: 140,
      ellipsis: true,
      render: (val: string) => (
        <Typography.Text
          style={{ color: "#ef4444", fontSize: 13 }}
          ellipsis
          copyable
        >
          {val || "—"}
        </Typography.Text>
      ),
    },
    {
      title: "占位符",
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
      title: "大模型/模型",
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
      title: "操作",
      key: "actions",
      width: 90,
      fixed: "right",
      render: (_, record) => (
        <Space size={4}>
          <Button
            type="link"
            size="small"
            icon={<EyeOutlined />}
            onClick={() => onViewDetail(record.id)}
          />
          <Popconfirm
            title="确定删除此记录？"
            onConfirm={() => onDelete(record.id)}
            okText="删除"
            cancelText="取消"
          >
            <Button type="link" size="small" danger icon={<DeleteOutlined />} />
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Table
      columns={columns}
      dataSource={dataSource}
      rowKey="id"
      loading={loading}
      size="small"
      scroll={{ x: "max-content", y: "calc(100vh - 300px)" }}
      pagination={{
        current: page,
        pageSize,
        total,
        showSizeChanger: true,
        pageSizeOptions: ["20", "50", "100"],
        onChange: onPageChange,
        showTotal: (t) => `共 ${t} 条`,
      }}
    />
  );
}
