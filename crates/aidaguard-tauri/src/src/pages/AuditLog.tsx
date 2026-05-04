import { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
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
  const { t } = useTranslation();
  const location = useLocation();
  const { token } = theme.useToken();
  const groups = useAuditStore((s) => s.groups);
  const groupTotal = useAuditStore((s) => s.groupTotal);
  const loading = useAuditStore((s) => s.loading);
  const selectedRecord = useAuditStore((s) => s.selectedRecord);
  const detailOpen = useAuditStore((s) => s.detailOpen);
  const page = useAuditStore((s) => s.page);
  const pageSize = useAuditStore((s) => s.pageSize);
  const expandedRecords = useAuditStore((s) => s.expandedRecords);
  const expandedLoading = useAuditStore((s) => s.expandedLoading);
  const fetchGroups = useAuditStore((s) => s.fetchGroups);
  const expandGroup = useAuditStore((s) => s.expandGroup);
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
    fetchGroups({
      pathFilter: searchText || undefined,
      dateFromMs: dateFrom,
      dateToMs: dateTo,
    });
  }, [searchText, dateRange, fetchGroups]);

  useEffect(() => {
    loadData();
  }, [page, pageSize]);

  // Auto-open detail when navigated from dashboard
  useEffect(() => {
    const state = location.state as { openRecordId?: string } | null;
    if (state?.openRecordId) {
      fetchDetail(state.openRecordId);
      // Clear state so back/forward doesn't re-trigger
      window.history.replaceState({}, "");
    }
  }, [location.state]);

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
      message.success(t("已导出到: {{path}}", { path }));
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleViewDetail = (id: string) => {
    fetchDetail(id);
  };

  const handleDelete = (id: string, ruleId?: string, strategy?: string) => {
    removeRecord(id, ruleId, strategy).then(() => message.success(t("已删除")));
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
              placeholder={t("搜索规则名、请求路径")}
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
              {t("搜索")}
            </Button>
            <Button onClick={loadData} icon={<ReloadOutlined />}>
              {t("刷新")}
            </Button>
          </Space>
          <Space>
            <Button icon={<ExportOutlined />} onClick={() => handleExport("csv")}>
              {t("导出 CSV")}
            </Button>
            <Button icon={<ExportOutlined />} onClick={() => handleExport("json")}>
              {t("导出 JSON")}
            </Button>
          </Space>
        </Space>

        {/* Table */}
        <AuditTable
          groups={groups}
          groupTotal={groupTotal}
          loading={loading}
          page={page}
          pageSize={pageSize}
          expandedRecords={expandedRecords}
          expandedLoading={expandedLoading}
          onPageChange={handlePageChange}
          onExpand={(ruleId, strategy) => expandGroup(ruleId, strategy)}
          onViewDetail={handleViewDetail}
          onDelete={handleDelete}
        />
      </Card>

      {/* Detail Drawer */}
      <Drawer
        title={t("审计详情")}
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
