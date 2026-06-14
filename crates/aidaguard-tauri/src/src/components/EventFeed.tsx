import { AlertTriangle, Copy, Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useState } from "react";
import type { DetectionRecord } from "../types";
import { format } from "date-fns";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface EventFeedProps {
  records: DetectionRecord[];
  onClickRecord?: (id: string) => void;
}

function StrategyLabel({ strategy }: { strategy: string }) {
  const { t } = useTranslation();
  let label: string, variant: "default" | "secondary" | "outline", className: string;
  switch (strategy) {
    case "placeholder":
    case "filter":
      label = t("Filtered"); variant = "default"; className = "bg-green-500/15 text-green-600 hover:bg-green-500/15"; break;
    case "detect":
      label = t("Detect Only"); variant = "secondary"; className = "bg-amber-500/15 text-amber-600 hover:bg-amber-500/15"; break;
    case "mask":
      label = t("Masked"); variant = "secondary"; className = "bg-purple-500/15 text-purple-600 hover:bg-purple-500/15"; break;
    default:
      label = strategy; variant = "outline"; className = ""; break;
  }
  return <Badge variant={variant} className={cn("text-xs", className)}>{label}</Badge>;
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const handleCopy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  return (
    <button onClick={handleCopy} className="text-muted-foreground hover:text-foreground transition-colors p-0.5">
      {copied ? <Check className="h-3 w-3 text-green-500" /> : <Copy className="h-3 w-3" />}
    </button>
  );
}

export default function EventFeed({ records, onClickRecord }: EventFeedProps) {
  const { t } = useTranslation();

  if (records.length === 0) {
    return (
      <div className="text-center py-8 text-muted-foreground">
        <AlertTriangle className="h-8 w-8 mx-auto mb-2" />
        {t("No Detection Events")}
      </div>
    );
  }

  return (
    <div>
      {records.map((item) => {
        return (
          <div
            key={item.id}
            className={cn(
              "py-2.5 border-b border-border",
              onClickRecord ? "cursor-pointer" : ""
            )}
            onClick={() => onClickRecord?.(item.id)}
          >
            {/* Row 1: strategy + rule name + tool + time */}
            <div className="flex justify-between items-center mb-1.5">
              <div className="flex items-center gap-1.5">
                <StrategyLabel strategy={item.strategy} />
                <span className="text-[13px] font-semibold">{item.ruleName || item.ruleId}</span>
                {item.toolName && (
                  <Badge variant="outline" className="text-[11px]">{item.toolName}</Badge>
                )}
              </div>
              <span className="text-[11px] text-muted-foreground">
                {format(item.timestampMs, "MM-dd HH:mm:ss")}
              </span>
            </div>

            {/* Row 2: original data + request path */}
            <div className="flex items-start gap-2">
              <span className="flex-1 text-[13px] text-destructive bg-destructive/10 px-2 py-0.5 rounded break-all leading-relaxed inline-flex items-start gap-1">
                <span className="flex-1 break-all">{item.original || "—"}</span>
                {item.original && <CopyButton text={item.original} />}
              </span>
              <span className="text-[11px] text-muted-foreground whitespace-nowrap shrink-0 max-w-[120px] truncate">
                {item.requestPath || "—"}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
