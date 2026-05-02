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
      width: 160,
      render: (val: number) => dayjs(val).format("YYYY-MM-DD HH:mm:ss"),
    },
    {
      title: "规则",
      dataIndex: "ruleId",
      key: "rule",
      width: 140,
      render: (val: string) => <Tag color="orange">{val}</Tag>,
    },
    {
      title: "策略",
      dataIndex: "strategy",
      key: "strategy",
      width: 100,
      render: (val: string) => <Tag>{val}</Tag>,
    },
    {
      title: "请求路径",
      dataIndex: "requestPath",
      key: "path",
      ellipsis: true,
      render: (val: string) => (
        <Typography.Text style={{ fontSize: 13 }} ellipsis>
          {val || "/"}
        </Typography.Text>
      ),
    },
    {
      title: "状态码",
      dataIndex: "responseStatus",
      key: "status",
      width: 80,
      render: (val: number) => (
        <Tag color={val >= 200 && val < 300 ? "green" : "red"}>{val}</Tag>
      ),
    },
    {
      title: "操作",
      key: "actions",
      width: 120,
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
