import { useState } from "react";
import { ChevronDown, ChevronUp, Copy, Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { DetectionRecord } from "../types";
import { format } from "date-fns";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface AuditDetailPanelProps {
  record: DetectionRecord;
}

const PREVIEW_MAX = 300;

function Copyable({ text, className }: { text: string; className?: string }) {
  const [copied, setCopied] = useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <span className={cn("inline-flex items-start gap-1", className)}>
      <span className="break-all">{text}</span>
      <button onClick={handleCopy} className="text-muted-foreground hover:text-foreground shrink-0 mt-0.5">
        {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
      </button>
    </span>
  );
}

function strategyBadge(strategy: string, t: (k: string) => string) {
  if (strategy === "detect")
    return <Badge className="bg-amber-500/15 text-amber-600 hover:bg-amber-500/15">{t("Detect Only")}</Badge>;
  if (strategy === "mask")
    return <Badge className="bg-purple-500/15 text-purple-600 hover:bg-purple-500/15">{t("Partial Mask")}</Badge>;
  return <Badge className="bg-blue-500/15 text-blue-600 hover:bg-blue-500/15">{t("Placeholder Replacement")}</Badge>;
}

export default function AuditDetailPanel({ record }: AuditDetailPanelProps) {
  const { t } = useTranslation();
  const [bodyExpanded, setBodyExpanded] = useState(false);

  const body = record.sanitizedBody || "";
  const bodyLen = body.length;
  const truncated = !bodyExpanded && bodyLen > PREVIEW_MAX;
  const displayBody = truncated ? body.slice(0, PREVIEW_MAX) + "…" : body;

  return (
    <div className="px-6 pb-4">
      {/* Description grid */}
      <div className="grid grid-cols-2 gap-x-6 gap-y-2 mb-4 text-sm border rounded-lg overflow-hidden">
        <DescItem label={t("Record ID")} span2>
          <Copyable text={record.id} className="text-xs" />
        </DescItem>
        <DescItem label={t("Time")}>
          {format(record.timestampMs, "yyyy-MM-dd HH:mm:ss.SSS")}
        </DescItem>
        <DescItem label={t("Response Status")}>
          <Badge variant={record.responseStatus < 300 ? "default" : "destructive"}>
            {record.responseStatus}
          </Badge>
        </DescItem>
        <DescItem label={t("Tool")}>{record.toolName || "—"}</DescItem>
        <DescItem label={t("Rule Name")}>
          <Badge className="bg-amber-500/15 text-amber-600 hover:bg-amber-500/15">
            {record.ruleName || record.ruleId}
          </Badge>
        </DescItem>
        <DescItem label={t("Audit Strategy")}>
          {strategyBadge(record.strategy, t)}
        </DescItem>
        <DescItem label={t("Placeholder")} span2>
          <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{record.placeholder}</code>
        </DescItem>
        <DescItem label={t("LLM / Model")} span2>
          {record.requestPath || "—"}
        </DescItem>
        <DescItem label={t("Original Data")} span2>
          <Copyable text={record.original} className="text-destructive" />
        </DescItem>
        <DescItem label={t("Context")} span2>
          <span className="break-all">{record.context || "—"}</span>
        </DescItem>
      </div>

      {/* Sanitized body */}
      <div className="flex items-center gap-2 mb-2">
        <span className="font-semibold text-sm">{t("Sanitized Request Body")}</span>
        <span className="text-xs text-muted-foreground">({bodyLen.toLocaleString()} chars)</span>
      </div>
      {body ? (
        <>
          <pre
            className={cn(
              "bg-muted text-foreground p-3 rounded-md text-xs whitespace-pre-wrap break-all m-0",
              truncated ? "max-h-[120px]" : "max-h-[320px]",
              "overflow-auto"
            )}
          >
            {displayBody}
          </pre>
          {bodyLen > PREVIEW_MAX && (
            <Button
              variant="link"
              size="sm"
              className="px-0 py-1 h-auto"
              onClick={() => setBodyExpanded(!bodyExpanded)}
            >
              {bodyExpanded ? <ChevronUp className="h-3 w-3 mr-1" /> : <ChevronDown className="h-3 w-3 mr-1" />}
              {bodyExpanded ? t("Collapse") : t("Show Full Body")}
            </Button>
          )}
        </>
      ) : (
        <span className="text-muted-foreground">—</span>
      )}
    </div>
  );
}

function DescItem({
  label,
  span2,
  children,
}: {
  label: string;
  span2?: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className={cn("px-3 py-2 border-b border-r last:border-r-0", span2 ? "col-span-2" : "")}>
      <div className="text-xs text-muted-foreground mb-0.5">{label}</div>
      <div>{children}</div>
    </div>
  );
}
