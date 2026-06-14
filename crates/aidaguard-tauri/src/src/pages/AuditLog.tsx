import { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
import { Search, Download, RefreshCw } from "lucide-react";
import { startOfDay, endOfDay } from "date-fns";
import { toast } from "sonner";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";

import { useAuditStore } from "../store/useAuditStore";
import AuditTable from "../components/AuditTable";
import AuditDetailPanel from "../components/AuditDetailPanel";

export default function AuditLog() {
  const { t } = useTranslation();
  const location = useLocation();
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
  const [dateRange, setDateRange] = useState<[string, string]>(["", ""]);

  const loadData = useCallback(() => {
    let dateFrom: number | undefined;
    let dateTo: number | undefined;
    if (dateRange[0]) {
      dateFrom = startOfDay(new Date(dateRange[0])).getTime();
    }
    if (dateRange[1]) {
      dateTo = endOfDay(new Date(dateRange[1])).getTime();
    }
    fetchGroups({
      pathFilter: searchText || undefined,
      dateFromMs: dateFrom,
      dateToMs: dateTo,
    });
  }, [searchText, dateRange, fetchGroups]);

  useEffect(() => {
    loadData();
  }, [page, pageSize, loadData]);

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
      toast.success(t("Exported to: {{path}}", { path }));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleViewDetail = (id: string) => {
    fetchDetail(id);
  };

  const handleDelete = (id: string, ruleId?: string, strategy?: string) => {
    removeRecord(id, ruleId, strategy).then(() => toast.success(t("Deleted")));
  };

  return (
    <div>
      <Card className="rounded-xl border-border/50">
        <CardContent className="p-4">
          {/* Toolbar */}
          <div className="mb-4 flex flex-wrap items-center justify-between gap-2">
            <div className="flex flex-wrap items-center gap-2">
              {/* Search input with icon */}
              <div className="relative">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder={t("Search Rule Name or Request Path")}
                  value={searchText}
                  onChange={(e) => setSearchText(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleSearch();
                  }}
                  className="w-80 pl-9 pr-8"
                />
                {searchText && (
                  <button
                    onClick={() => setSearchText("")}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  >
                    &times;
                  </button>
                )}
              </div>

              {/* Date range inputs */}
              <div className="flex items-center gap-1">
                <input
                  type="date"
                  value={dateRange[0]}
                  onChange={(e) =>
                    setDateRange([e.target.value, dateRange[1]])
                  }
                  className={cn(
                    "flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                    "disabled:cursor-not-allowed disabled:opacity-50"
                  )}
                />
                <span className="text-muted-foreground">~</span>
                <input
                  type="date"
                  value={dateRange[1]}
                  onChange={(e) =>
                    setDateRange([dateRange[0], e.target.value])
                  }
                  className={cn(
                    "flex h-10 rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background",
                    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
                    "disabled:cursor-not-allowed disabled:opacity-50"
                  )}
                />
                {(dateRange[0] || dateRange[1]) && (
                  <button
                    onClick={() => setDateRange(["", ""])}
                    className="ml-1 text-muted-foreground hover:text-foreground"
                  >
                    &times;
                  </button>
                )}
              </div>

              <Button variant="outline" size="sm" onClick={handleSearch}>
                <Search className="h-4 w-4" />
                {t("Search")}
              </Button>
              <Button variant="outline" size="sm" onClick={loadData}>
                <RefreshCw className="h-4 w-4" />
                {t("Refresh")}
              </Button>
            </div>

            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExport("csv")}
              >
                <Download className="h-4 w-4" />
                {t("Export CSV")}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExport("json")}
              >
                <Download className="h-4 w-4" />
                {t("Export JSON")}
              </Button>
            </div>
          </div>

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
        </CardContent>
      </Card>

      {/* Detail Sheet */}
      <Sheet open={detailOpen} onOpenChange={(open) => !open && closeDetail()}>
        <SheetContent side="right" className="w-[600px] sm:max-w-[600px]">
          <SheetHeader>
            <SheetTitle>{t("Audit Detail")}</SheetTitle>
          </SheetHeader>
          <div className="mt-4 overflow-y-auto">
            {selectedRecord && <AuditDetailPanel record={selectedRecord} />}
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}
