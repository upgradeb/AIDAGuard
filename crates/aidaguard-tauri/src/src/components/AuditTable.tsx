import { useState, Fragment } from "react";
import { Eye, Trash2, ChevronRight, Copy, Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { AuditGroup, DetectionRecord } from "../types";
import { format } from "date-fns";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

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

function strategyBadge(strategy: string, t: (k: string) => string) {
  if (strategy === "detect")
    return <Badge className="bg-amber-500/15 text-amber-600 hover:bg-amber-500/15">{t("Detect Only")}</Badge>;
  if (strategy === "mask")
    return <Badge className="bg-purple-500/15 text-purple-600 hover:bg-purple-500/15">{t("Partial Mask")}</Badge>;
  return <Badge className="bg-blue-500/15 text-blue-600 hover:bg-blue-500/15">{t("Placeholder Replacement")}</Badge>;
}

function CopyableText({ text, className }: { text: string; className?: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <span className={cn("inline-flex items-center gap-1", className)}>
      <span className="truncate">{text || "—"}</span>
      {text && (
        <button
          onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(text); setCopied(true); setTimeout(() => setCopied(false), 1500); }}
          className="text-muted-foreground hover:text-foreground shrink-0"
        >
          {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
        </button>
      )}
    </span>
  );
}

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
  const [expandedKeys, setExpandedKeys] = useState<Set<string>>(new Set());
  const [deleteTarget, setDeleteTarget] = useState<DetectionRecord | null>(null);

  const toggleExpand = (group: AuditGroup) => {
    const key = groupKey(group);
    const next = new Set(expandedKeys);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
      onExpand(group.ruleId, group.strategy);
    }
    setExpandedKeys(next);
  };

  const totalPages = Math.ceil(groupTotal / pageSize);

  if (loading) {
    return (
      <div className="space-y-2">
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-12 w-full" />
        ))}
      </div>
    );
  }

  return (
    <div>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="w-10" />
            <TableHead className="w-[170px]">{t("Latest Time")}</TableHead>
            <TableHead className="w-[120px]">{t("Audit Strategy")}</TableHead>
            <TableHead className="w-[160px]">{t("Rule Name")}</TableHead>
            <TableHead className="w-[80px] text-right">{t("Hits")}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {groups.map((group) => {
            const key = groupKey(group);
            const isExpanded = expandedKeys.has(key);
            const records = expandedRecords[key];
            const isLoading = expandedLoading[key];

            return (
              <Fragment key={key}>
                <TableRow
                  className="cursor-pointer hover:bg-accent/50"
                  onClick={() => toggleExpand(group)}
                >
                  <TableCell>
                    <ChevronRight className={cn("h-4 w-4 transition-transform", isExpanded && "rotate-90")} />
                  </TableCell>
                  <TableCell className="text-sm">
                    {format(group.latestTimestampMs, "yyyy-MM-dd HH:mm:ss")}
                  </TableCell>
                  <TableCell>{strategyBadge(group.strategy, t)}</TableCell>
                  <TableCell>
                    <div className="flex items-center gap-2">
                      <Badge className="bg-amber-500/15 text-amber-600 hover:bg-amber-500/15">
                        {group.ruleName || group.ruleId}
                      </Badge>
                      <span className="inline-flex items-center justify-center min-w-[20px] h-5 px-1.5 rounded-full bg-blue-500/15 text-blue-600 text-xs font-medium">
                        {group.count}
                      </span>
                    </div>
                  </TableCell>
                  <TableCell className="text-right font-semibold text-sm">{group.count}</TableCell>
                </TableRow>

                {isExpanded && (
                  <TableRow>
                    <TableCell colSpan={5} className="p-0">
                      <div className="bg-muted/50 px-4 py-2">
                        {isLoading ? (
                          <div className="space-y-1 py-2">
                            <Skeleton className="h-8 w-full" />
                            <Skeleton className="h-8 w-full" />
                          </div>
                        ) : records && records.length > 0 ? (
                          <Table>
                            <TableHeader>
                              <TableRow>
                                <TableHead className="w-[150px]">{t("Time")}</TableHead>
                                <TableHead className="w-[150px]">{t("Original Data")}</TableHead>
                                <TableHead className="w-[160px]">{t("Placeholder")}</TableHead>
                                <TableHead className="w-[100px]">{t("Tool")}</TableHead>
                                <TableHead className="w-[140px]">{t("Model")}</TableHead>
                                <TableHead className="w-[100px]">{t("Actions")}</TableHead>
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              {records.map((record) => (
                                <TableRow key={record.id}>
                                  <TableCell className="text-sm">
                                    {format(record.timestampMs, "yyyy-MM-dd HH:mm:ss")}
                                  </TableCell>
                                  <TableCell className="text-sm">
                                    <CopyableText text={record.original} className="text-destructive" />
                                  </TableCell>
                                  <TableCell className="text-sm">
                                    {record.placeholder ? (
                                      <code className="text-xs bg-muted px-1 py-0.5 rounded">{record.placeholder}</code>
                                    ) : "—"}
                                  </TableCell>
                                  <TableCell className="text-sm">
                                    {record.toolName ? (
                                      <Badge variant="outline">{record.toolName}</Badge>
                                    ) : (
                                      <span className="text-muted-foreground">—</span>
                                    )}
                                  </TableCell>
                                  <TableCell className="text-sm truncate max-w-[140px]">
                                    {record.requestPath || "—"}
                                  </TableCell>
                                  <TableCell>
                                    <div className="flex items-center gap-1">
                                      <Button
                                        variant="ghost"
                                        size="icon"
                                        className="h-7 w-7"
                                        onClick={() => onViewDetail(record.id)}
                                      >
                                        <Eye className="h-4 w-4" />
                                      </Button>
                                      <Button
                                        variant="ghost"
                                        size="icon"
                                        className="h-7 w-7 text-destructive"
                                        onClick={() => setDeleteTarget(record)}
                                      >
                                        <Trash2 className="h-4 w-4" />
                                      </Button>
                                    </div>
                                  </TableCell>
                                </TableRow>
                              ))}
                            </TableBody>
                          </Table>
                        ) : (
                          <div className="py-4 text-center text-muted-foreground text-sm">{t("No Data")}</div>
                        )}
                      </div>
                    </TableCell>
                  </TableRow>
                )}
              </Fragment>
            );
          })}
        </TableBody>
      </Table>

      {/* Pagination */}
      <div className="flex items-center justify-between mt-4">
        <span className="text-sm text-muted-foreground">
          {t("{{total}} Groups Total", { total: groupTotal })}
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={page <= 1}
            onClick={() => onPageChange(page - 1, pageSize)}
          >
            {t("Previous")}
          </Button>
          <span className="text-sm">
            {page} / {totalPages || 1}
          </span>
          <Button
            variant="outline"
            size="sm"
            disabled={page >= totalPages}
            onClick={() => onPageChange(page + 1, pageSize)}
          >
            {t("Next")}
          </Button>
          <select
            className="h-8 rounded-md border border-input bg-background px-2 text-sm"
            value={pageSize}
            onChange={(e) => onPageChange(1, Number(e.target.value))}
          >
            {[10, 20, 50].map((s) => (
              <option key={s} value={s}>{s}/page</option>
            ))}
          </select>
        </div>
      </div>

      {/* Delete confirmation */}
      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("Delete this record?")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("This action cannot be undone.")}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={() => {
                if (deleteTarget) {
                  onDelete(deleteTarget.id, deleteTarget.ruleId, deleteTarget.strategy);
                  setDeleteTarget(null);
                }
              }}
            >
              {t("Delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
