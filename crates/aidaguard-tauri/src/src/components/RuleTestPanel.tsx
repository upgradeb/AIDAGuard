import { useState } from "react";
import { Play } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { TestRuleResult } from "../types";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface RuleTestPanelProps {
  open: boolean;
  testing: boolean;
  result: TestRuleResult | null;
  onTest: (pattern: string, text: string) => Promise<void>;
  onClose: () => void;
}

export default function RuleTestPanel({
  open,
  testing,
  result,
  onTest,
  onClose,
}: RuleTestPanelProps) {
  const [pattern, setPattern] = useState("");
  const [text, setText] = useState("");
  const { t } = useTranslation();

  return (
    <Sheet open={open} onOpenChange={(o) => { if (!o) onClose(); }}>
      <SheetContent className="w-[640px] sm:max-w-[640px] overflow-auto">
        <SheetHeader>
          <SheetTitle>{t("Rule Test")}</SheetTitle>
        </SheetHeader>

        <div className="mt-6 space-y-4">
          <div className="space-y-2">
            <Label>{t("Regex Pattern")}</Label>
            <Input
              value={pattern}
              onChange={(e) => setPattern(e.target.value)}
              placeholder={t("e.g. 1[3-9]\\d{9}")}
            />
          </div>

          <div className="space-y-2">
            <Label>{t("Test Text")}</Label>
            <Textarea
              rows={5}
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder={t("Enter test text containing sensitive data...")}
            />
          </div>

          <Button
            onClick={() => onTest(pattern, text)}
            disabled={!pattern || !text}
            className="w-full"
          >
            <Play className="h-4 w-4 mr-2" />
            {testing ? t("Testing...") : t("Run Test")}
          </Button>

          {result && (
            <>
              <Separator />

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm">
                    {t("Matches: {{count}}", { count: result.matches.length })}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {result.matches.map((m, i) => (
                    <div key={i} className="py-2 border-b last:border-b-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <Badge className="bg-amber-500/15 text-amber-600 hover:bg-amber-500/15">
                          {m.text}
                        </Badge>
                        <span className="text-xs text-muted-foreground">
                          pos {m.start}-{m.end}
                        </span>
                        <Badge variant="secondary">{m.strategy}</Badge>
                        <Badge variant={m.mode === "filter" ? "default" : "outline"}>
                          {m.mode === "filter" ? t("Filter") : t("Detect")}
                        </Badge>
                      </div>
                    </div>
                  ))}
                  {result.matches.length === 0 && (
                    <span className="text-sm text-muted-foreground">{t("No Matches")}</span>
                  )}
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm">{t("Sanitized Text")}</CardTitle>
                </CardHeader>
                <CardContent>
                  <pre className="bg-muted text-foreground p-3 rounded-md text-[13px] whitespace-pre-wrap break-all">
                    {result.sanitizedText}
                  </pre>
                </CardContent>
              </Card>
            </>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
