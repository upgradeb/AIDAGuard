import { useEffect, useState, useCallback } from "react";
import {
  Card,
  Space,
  Input,
  DatePicker,
  Button,
  Drawer,
  message,
  theme,
} from "antd";
import {
  SearchOutlined,
  ExportOutlined,
  ReloadOutlined,
} from "@ant-design/icons";
import { useAuditStore } from "../store/useAuditStore";
import AuditTable from "../components/AuditTable";
import AuditDetailPanel from "../components/AuditDetailPanel";
import type { Dayjs } from "dayjs";

const { RangePicker } = DatePicker;

export default function AuditLog() {
  const { token } = theme.useToken();
  const records = useAuditStore((s) => s.records);
  const total = useAuditStore((s) => s.total);
  const loading = useAuditStore((s) => s.loading);
  const selectedRecord = useAuditStore((s) => s.selectedRecord);
  const detailOpen = useAuditStore((s) => s.detailOpen);
  const page = useAuditStore((s) => s.page);
  const pageSize = useAuditStore((s) => s.pageSize);
  const fetchList = useAuditStore((s) => s.fetchList);
  const fetchDetail = useAuditStore((s) => s.fetchDetail);
  const removeRecord = useAuditStore((s) => s.removeRecord);
  const doExport = useAuditStore((s) => s.doExport);
  const closeDetail = useAuditStore((s) => s.closeDetail);
  const setPage = useAuditStore((s) => s.setPage);

  const [searchText, setSearchText] = useState("");
  const [dateRange, setDateRange] = useState<[Dayjs | null, Dayjs | null]>([null, null]);

  const loadData = useCallback(() => {
    const dateFrom = dateRange[0]?.startOf("day").valueOf();
    const dateTo = dateRange[1]?.endOf("day").valueOf();
    fetchList({
      pathFilter: searchText || undefined,
      dateFromMs: dateFrom,
      dateToMs: dateTo,
    });
  }, [searchText, dateRange, fetchList]);

  useEffect(() => {
    loadData();
  }, [page, pageSize]);

  const handleSearch = () => {
    setPage(1);
    loadData();
  };

  const handlePageChange = (p: number, ps: number) => {
    setPage(p, ps);
  };

  const handleExport = async (format: "csv" | "json") => {
    try {
      const path = await doExport(format);
      message.success(`已导出到: ${path}`);
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleViewDetail = (id: string) => {
    fetchDetail(id);
  };

  const handleDelete = (id: string) => {
    removeRecord(id).then(() => message.success("已删除"));
  };

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
            <Input
              placeholder="搜索规则名、请求路径"
              prefix={<SearchOutlined />}
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              onPressEnter={handleSearch}
              style={{ width: 280 }}
              allowClear
            />
            <RangePicker
              value={dateRange as [Dayjs, Dayjs]}
              onChange={(dates) => {
                setDateRange(dates ? [dates[0], dates[1]] : [null, null]);
              }}
              allowClear
            />
            <Button onClick={handleSearch} icon={<SearchOutlined />}>
              搜索
            </Button>
            <Button onClick={loadData} icon={<ReloadOutlined />}>
              刷新
            </Button>
          </Space>
          <Space>
            <Button icon={<ExportOutlined />} onClick={() => handleExport("csv")}>
              导出 CSV
            </Button>
            <Button icon={<ExportOutlined />} onClick={() => handleExport("json")}>
              导出 JSON
            </Button>
          </Space>
        </Space>

        {/* Table */}
        <AuditTable
          dataSource={records}
          loading={loading}
          total={total}
          page={page}
          pageSize={pageSize}
          onPageChange={handlePageChange}
          onViewDetail={handleViewDetail}
          onDelete={handleDelete}
        />
      </Card>

      {/* Detail Drawer */}
      <Drawer
        title="审计详情"
        placement="right"
        width={600}
        open={detailOpen}
        onClose={closeDetail}
      >
        {selectedRecord && <AuditDetailPanel record={selectedRecord} />}
      </Drawer>
    </div>
  );
}
