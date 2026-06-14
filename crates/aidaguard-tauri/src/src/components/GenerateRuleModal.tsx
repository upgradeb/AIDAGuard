import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Bot, Zap, Pencil, RefreshCw, Server } from "lucide-react";
import { generateRule, type GeneratedRule } from "../api/rules";
import type { RuleDef } from "../types";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { toast } from "sonner";
import { cn } from "@/lib/utils";

interface GenerateRuleModalProps {
  open: boolean;
  defaultModelLabel: string;
  onApply: (rule: RuleDef) => void;
  onClose: () => void;
}

export default function GenerateRuleModal({
  open,
  defaultModelLabel,
  onApply,
  onClose,
}: GenerateRuleModalProps) {
  const { t } = useTranslation();
  const [sampleText, setSampleText] = useState("");
  const [generating, setGenerating] = useState(false);
  const [result, setResult] = useState<GeneratedRule | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async () => {
    if (!sampleText.trim()) {
      toast.warning(t("Please enter test sample"));
      return;
    }
    setGenerating(true);
    setError(null);
    try {
      const rule = await generateRule(sampleText);
      setResult(rule);
    } catch (e) {
      setError(String(e));
    } finally {
      setGenerating(false);
    }
  };

  const handleApply = () => {
    if (!result) return;
    onApply({
      id: result.id || "",
      name: result.name,
      pattern: result.pattern,
      strategy: result.strategy as "placeholder" | "mask",
      mode: result.mode as "detect" | "filter",
      priority: result.priority,
      enabled: true,
    });
    setSampleText("");
    setResult(null);
    setError(null);
    onClose();
  };

  const handleClose = () => {
    setSampleText("");
    setResult(null);
    setError(null);
    onClose();
  };

  const modelLabel = result
    ? `${result.upstreamName} / ${result.model}`
    : defaultModelLabel;

  return (
    <Dialog open={open} onOpenChange={(o) => { if (!o) handleClose(); }}>
      <DialogContent className="sm:max-w-[640px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Bot className="h-5 w-5" />
            {t("AI-Generated Rules")}
          </DialogTitle>
        </DialogHeader>

        {/* Model info */}
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-md bg-primary/10 border border-primary/20 text-sm">
          <Server className="h-4 w-4 text-preset" />
          <span className="text-preset">
            {t("Model: ")}<strong>{modelLabel}</strong>
          </span>
        </div>

        <p className="text-sm text-muted-foreground">
          {t("Enter a test sample containing sensitive data. The LLM will analyze it and generate detection rules automatically. You can further refine the result in the rule editor.")}
        </p>

        <Textarea
          value={sampleText}
          onChange={(e) => setSampleText(e.target.value)}
          placeholder={t("Example:\nPatient Zhang San, Phone 13812345678, ID 320102199001011234")}
          rows={4}
        />

        {!result && (
          <Button
            onClick={handleGenerate}
            disabled={generating || !sampleText.trim()}
            className="w-full"
          >
            <Zap className="h-4 w-4 mr-2" />
            {generating ? t("Generating...") : t("Generate Rule")}
          </Button>
        )}

        {generating && !result && (
          <div className="flex flex-col items-center gap-2 py-6">
            <Skeleton className="h-4 w-[200px]" />
            <Skeleton className="h-4 w-[160px]" />
            <span className="text-sm text-muted-foreground">{t("LLM is analyzing sample...")}</span>
          </div>
        )}

        {error && (
          <Alert variant="destructive">
            <AlertTitle>{t("Generation Failed")}</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {result && (
          <div
            className={cn(
              "mt-4 p-4 rounded-lg border transition-colors",
              generating
                ? "bg-amber-500/10 border-amber-500/30"
                : "bg-green-500/10 border-green-500/30"
            )}
          >
            <span
              className={cn(
                "font-semibold block mb-3",
                generating ? "text-amber-600" : "text-green-600"
              )}
            >
              {generating
                ? t("Regenerating...")
                : t("Done — {{upstreamName}} / {{model}}", { upstreamName: result.upstreamName, model: result.model })}
            </span>

            <div className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
              <span className="text-muted-foreground">{t("Rule ID")}</span>
              <code className="text-xs">{result.id}</code>

              <span className="text-muted-foreground">{t("Rule Name")}</span>
              <span className="font-medium">{result.name}</span>

              <span className="text-muted-foreground">{t("Pattern")}</span>
              <code className="text-xs">{result.pattern}</code>

              <span className="text-muted-foreground">{t("Strategy")}</span>
              <Badge className={result.strategy === "placeholder" ? "bg-blue-500/15 text-blue-600 hover:bg-blue-500/15" : "bg-purple-500/15 text-purple-600 hover:bg-purple-500/15"}>
                {result.strategy === "placeholder" ? t("Placeholder Replacement") : t("Partial Mask")}
              </Badge>

              <span className="text-muted-foreground">{t("Mode")}</span>
              <Badge className={result.mode === "filter" ? "bg-green-500/15 text-green-600 hover:bg-green-500/15" : "bg-amber-500/15 text-amber-600 hover:bg-amber-500/15"}>
                {result.mode === "filter" ? t("Filter & Replace") : t("Detect Only")}
              </Badge>

              <span className="text-muted-foreground">{t("Priority")}</span>
              <span>{result.priority}</span>

              <span className="text-muted-foreground">{t("Generation Model")}</span>
              <code className="text-xs">{result.upstreamName} / {result.model}</code>
            </div>
          </div>
        )}

        <DialogFooter>
          {result ? (
            <>
              <Button variant="outline" onClick={handleClose}>{t("Close")}</Button>
              <Button variant="outline" onClick={handleGenerate} disabled={generating}>
                <RefreshCw className={cn("h-4 w-4 mr-2", generating && "animate-spin")} />
                {t("Regenerate")}
              </Button>
              <Button onClick={handleApply}>
                <Pencil className="h-4 w-4 mr-2" />
                {t("Apply to Editor")}
              </Button>
            </>
          ) : (
            <Button variant="outline" onClick={handleClose}>{t("Cancel")}</Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
