import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Plus, Pencil, Trash2, Server, Loader2 } from "lucide-react";
import { toast } from "sonner";

import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import { useUpstreamStore } from "../store/useUpstreamStore";
import type { UpstreamConfig } from "../types";

/* ---------- zod schema ---------- */

const upstreamSchema = z.object({
  name: z.string().min(1, "Name is required"),
  url: z.string().min(1, "API URL is required"),
  api_key: z.string(),
  default: z.boolean(),
  timeout_secs: z.number().min(1).max(600),
  rate_limit_qps: z.number().min(0).max(100),
  models: z.array(z.string()),
  protocol: z.enum(["openai", "anthropic"]),
});

type UpstreamFormValues = z.infer<typeof upstreamSchema>;

/* ---------- component ---------- */

export default function Upstreams() {
  const { t } = useTranslation();

  const upstreams = useUpstreamStore((s) => s.upstreams);
  const loading = useUpstreamStore((s) => s.loading);
  const saving = useUpstreamStore((s) => s.saving);
  const testing = useUpstreamStore((s) => s.testing);
  const testResult = useUpstreamStore((s) => s.testResult);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);
  const add = useUpstreamStore((s) => s.addUpstream);
  const update = useUpstreamStore((s) => s.updateUpstream);
  const remove = useUpstreamStore((s) => s.deleteUpstream);
  const testConn = useUpstreamStore((s) => s.testConnectivity);
  const clearTestResult = useUpstreamStore((s) => s.clearTestResult);

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<UpstreamConfig | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const {
    control,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<UpstreamFormValues>({
    resolver: zodResolver(upstreamSchema),
    defaultValues: {
      name: "",
      url: "",
      api_key: "",
      default: false,
      timeout_secs: 300,
      rate_limit_qps: 0,
      models: [],
      protocol: "openai",
    },
  });

  useEffect(() => {
    fetchUpstreams();
  }, []);

  useEffect(() => {
    if (editorOpen) {
      if (editingRecord) {
        reset({
          name: editingRecord.name,
          url: editingRecord.url,
          api_key: editingRecord.api_key || "",
          default: editingRecord.default,
          timeout_secs: editingRecord.timeout_secs || 300,
          rate_limit_qps: editingRecord.rate_limit_qps || 0,
          models: editingRecord.models || [],
          protocol: editingRecord.protocol || "openai",
        });
      } else {
        reset({
          name: "",
          url: "",
          api_key: "",
          default: false,
          timeout_secs: 300,
          rate_limit_qps: 0,
          models: [],
          protocol: "openai",
        });
      }
    }
  }, [editorOpen]);

  const onSubmit = async (values: UpstreamFormValues) => {
    const upstream: UpstreamConfig = {
      name: values.name,
      url: values.url,
      api_key: values.api_key || undefined,
      default: values.default || false,
      timeout_secs: values.timeout_secs || 300,
      rate_limit_qps: values.rate_limit_qps || 0,
      models: values.models || [],
      protocol: values.protocol || "openai",
    };
    try {
      if (editingRecord) {
        await update(editingRecord.name, upstream);
      } else {
        await add(upstream);
      }
      toast.success(t("Upstream Saved"));
      setEditorOpen(false);
      setEditingRecord(null);
      fetchUpstreams();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleDelete = async (name: string) => {
    try {
      await remove(name);
      toast.success(t("Upstream Deleted"));
      setDeleteTarget(null);
      fetchUpstreams();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleTest = async (record: UpstreamConfig) => {
    await testConn(
      record.name,
      record.url,
      record.api_key || "",
      record.timeout_secs || 10
    );
  };

  return (
    <div className="max-w-[960px] h-full overflow-auto">
      {/* Card wrapper */}
      <div className="rounded-xl border p-4">
        {/* Header row */}
        <div className="flex items-center justify-between mb-4">
          <span className="text-sm text-muted-foreground">
            {t("Manage LLM Upstream Services")}
          </span>
          <Button
            onClick={() => {
              setEditingRecord(null);
              clearTestResult();
              setEditorOpen(true);
            }}
          >
            <Plus className="h-4 w-4 mr-1" />
            {t("Add Upstream")}
          </Button>
        </div>

        {/* Table */}
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[70px]">{t("Default")}</TableHead>
              <TableHead className="w-[120px]">{t("Name")}</TableHead>
              <TableHead>{t("URL")}</TableHead>
              <TableHead className="w-[90px]">{t("Timeout(s)")}</TableHead>
              <TableHead className="w-[90px]">{t("QPS")}</TableHead>
              <TableHead className="w-[200px]">{t("Model")}</TableHead>
              <TableHead className="w-[110px]">{t("Protocol")}</TableHead>
              <TableHead className="w-[200px]">{t("Actions")}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading ? (
              <TableRow>
                <TableCell colSpan={8} className="h-24 text-center text-muted-foreground">
                  <Loader2 className="h-5 w-5 animate-spin mx-auto" />
                </TableCell>
              </TableRow>
            ) : upstreams.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} className="h-24 text-center text-muted-foreground">
                  {t("No Upstreams Configured")}
                </TableCell>
              </TableRow>
            ) : (
              upstreams.map((record) => (
                <TableRow key={record.name}>
                  <TableCell>
                    {record.default ? (
                      <Badge variant="secondary">{t("Default")}</Badge>
                    ) : null}
                  </TableCell>
                  <TableCell className="font-medium">{record.name}</TableCell>
                  <TableCell className="max-w-[200px] truncate">{record.url}</TableCell>
                  <TableCell>{record.timeout_secs}</TableCell>
                  <TableCell>
                    {record.rate_limit_qps > 0
                      ? record.rate_limit_qps
                      : t("Unlimited")}
                  </TableCell>
                  <TableCell>
                    {record.models && record.models.length > 0 ? (
                      <div className="flex flex-wrap gap-1">
                        {record.models.map((m) => (
                          <Badge key={m} variant="outline" className="text-xs">
                            {m}
                          </Badge>
                        ))}
                      </div>
                    ) : (
                      <span className="text-muted-foreground/50 text-xs">
                        {t("Unspecified")}
                      </span>
                    )}
                  </TableCell>
                  <TableCell>
                    {record.protocol === "anthropic" ? (
                      <Badge className="bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300 border-orange-200 dark:border-orange-800">
                        Anthropic
                      </Badge>
                    ) : (
                      <Badge className="bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300 border-blue-200 dark:border-blue-800">
                        OpenAI
                      </Badge>
                    )}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleTest(record)}
                        disabled={testing === record.name}
                      >
                        {testing === record.name ? (
                          <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                        ) : (
                          <Server className="h-3.5 w-3.5 mr-1" />
                        )}
                        {t("Test")}
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setEditingRecord(record);
                          setEditorOpen(true);
                        }}
                      >
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <AlertDialog
                        open={deleteTarget === record.name}
                        onOpenChange={(open) => {
                          if (!open) setDeleteTarget(null);
                        }}
                      >
                        <AlertDialogTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            className="text-destructive hover:text-destructive"
                            onClick={() => setDeleteTarget(record.name)}
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>
                              {t("Delete this upstream?")}
                            </AlertDialogTitle>
                            <AlertDialogDescription>
                              {t("This action cannot be undone.")}
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel>
                              {t("Cancel")}
                            </AlertDialogCancel>
                            <AlertDialogAction
                              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                              onClick={() => handleDelete(record.name)}
                            >
                              {t("Delete")}
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      {/* Connectivity test result */}
      {testResult && (
        <Alert
          variant={testResult.startsWith("✓") ? "default" : "destructive"}
          className="mt-4 rounded-lg"
        >
          <AlertTitle>{t("Connectivity Test Result")}</AlertTitle>
          <AlertDescription className="whitespace-pre-wrap text-xs">
            {testResult}
          </AlertDescription>
          <button
            className="absolute right-3 top-3 text-muted-foreground hover:text-foreground"
            onClick={clearTestResult}
          >
            &times;
          </button>
        </Alert>
      )}

      {/* Add / Edit dialog */}
      <Dialog
        open={editorOpen}
        onOpenChange={(open) => {
          if (!open) {
            setEditorOpen(false);
            setEditingRecord(null);
          }
        }}
      >
        <DialogContent className="sm:max-w-[560px]">
          <DialogHeader>
            <DialogTitle>
              {editingRecord ? t("Edit Upstream") : t("Add Upstream")}
            </DialogTitle>
          </DialogHeader>

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 mt-2">
            {/* Name */}
            <div className="space-y-1.5">
              <Label htmlFor="name">{t("Name")}</Label>
              <Controller
                name="name"
                control={control}
                render={({ field }) => (
                  <Input
                    id="name"
                    placeholder={t("e.g. qianfan-pro")}
                    disabled={!!editingRecord}
                    {...field}
                  />
                )}
              />
              {errors.name && (
                <p className="text-xs text-destructive">{errors.name.message}</p>
              )}
            </div>

            {/* URL */}
            <div className="space-y-1.5">
              <Label htmlFor="url">{t("API URL")}</Label>
              <Controller
                name="url"
                control={control}
                render={({ field }) => (
                  <Input
                    id="url"
                    placeholder="https://qianfan.baidubce.com/v2/coding"
                    {...field}
                  />
                )}
              />
              {errors.url && (
                <p className="text-xs text-destructive">{errors.url.message}</p>
              )}
            </div>

            {/* API Key */}
            <div className="space-y-1.5">
              <Label htmlFor="api_key">API Key</Label>
              <Controller
                name="api_key"
                control={control}
                render={({ field }) => (
                  <Input
                    id="api_key"
                    type="password"
                    placeholder={t("Leave empty to skip authentication header")}
                    {...field}
                  />
                )}
              />
            </div>

            {/* Timeout + QPS + Default row */}
            <div className="flex items-end gap-4">
              <div className="space-y-1.5">
                <Label htmlFor="timeout_secs">{t("Timeout (s)")}</Label>
                <Controller
                  name="timeout_secs"
                  control={control}
                  render={({ field }) => (
                    <Input
                      id="timeout_secs"
                      type="number"
                      min={1}
                      max={600}
                      className="w-[140px]"
                      {...field}
                      onChange={(e) => field.onChange(Number(e.target.value))}
                    />
                  )}
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="rate_limit_qps">{t("QPS Limit")}</Label>
                <Controller
                  name="rate_limit_qps"
                  control={control}
                  render={({ field }) => (
                    <Input
                      id="rate_limit_qps"
                      type="number"
                      min={0}
                      max={100}
                      className="w-[140px]"
                      {...field}
                      onChange={(e) => field.onChange(Number(e.target.value))}
                    />
                  )}
                />
              </div>
              <div className="flex items-center gap-2 pb-1">
                <Controller
                  name="default"
                  control={control}
                  render={({ field }) => (
                    <Switch
                      id="default"
                      checked={field.value}
                      onCheckedChange={field.onChange}
                    />
                  )}
                />
                <Label htmlFor="default" className="cursor-pointer">
                  {t("Set as Default")}
                </Label>
              </div>
            </div>

            {/* Protocol */}
            <div className="space-y-1.5">
              <Label>{t("Protocol Type")}</Label>
              <Controller
                name="protocol"
                control={control}
                render={({ field }) => (
                  <ToggleGroup
                    type="single"
                    value={field.value}
                    onValueChange={(val) => {
                      if (val) field.onChange(val);
                    }}
                    variant="outline"
                    className="justify-start"
                  >
                    <ToggleGroupItem value="openai">
                      {t("OpenAI Compatible")}
                    </ToggleGroupItem>
                    <ToggleGroupItem value="anthropic">
                      {t("Anthropic Compatible")}
                    </ToggleGroupItem>
                  </ToggleGroup>
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t("Select the upstream LLM API protocol format")}
              </p>
            </div>

            {/* Models */}
            <div className="space-y-1.5">
              <Label htmlFor="models">{t("Model List")}</Label>
              <Controller
                name="models"
                control={control}
                render={({ field }) => (
                  <Textarea
                    id="models"
                    rows={2}
                    placeholder="gpt-4, claude-3-opus"
                    value={field.value ? field.value.join(", ") : ""}
                    onChange={(e) => {
                      const val = e.target.value
                        .split(",")
                        .map((s: string) => s.trim())
                        .filter(Boolean);
                      field.onChange(val);
                    }}
                  />
                )}
              />
              <p className="text-xs text-muted-foreground">
                {t("Separate multiple models with commas")}
              </p>
            </div>

            {/* Footer */}
            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setEditorOpen(false);
                  setEditingRecord(null);
                }}
              >
                {t("Cancel")}
              </Button>
              <Button type="submit" disabled={saving}>
                {saving && <Loader2 className="h-4 w-4 mr-1 animate-spin" />}
                {t("Save")}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
