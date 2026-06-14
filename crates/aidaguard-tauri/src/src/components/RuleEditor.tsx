import { useForm, Controller } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useTranslation } from "react-i18next";
import type { RuleDef } from "../types";
import type { RuleWithCategory } from "../api/rules";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";

interface RuleEditorProps {
  open: boolean;
  editing: RuleWithCategory | null;
  ruleFiles: string[];
  onSave: (rule: RuleDef, category: string) => Promise<void>;
  onCancel: () => void;
}

export default function RuleEditor({
  open,
  editing,
  ruleFiles,
  onSave,
  onCancel,
}: RuleEditorProps) {
  const { t } = useTranslation();
  const isEdit = !!editing;

  const schema = z.object({
    id: z.string().regex(/^[a-z0-9_]+$/, t("Only lowercase letters, digits and underscores")),
    name: z.string().min(1, t("Please enter Rule Name")),
    pattern: z.string().min(1, t("Please enter Regex Pattern")),
    strategy: z.enum(["placeholder", "mask"]),
    mode: z.enum(["filter", "detect"]),
    priority: z.preprocess(
      (v) => (typeof v === "string" ? parseInt(v, 10) : v),
      z.number().min(1).max(999)
    ),
    enabled: z.boolean(),
    category: z.string().min(1, t("Please Select Category")),
  });

  type FormValues = z.input<typeof schema>;

  const { control, handleSubmit, reset, formState: { errors } } = useForm<FormValues>({
    resolver: zodResolver(schema) as any,
    defaultValues: {
      id: "",
      name: "",
      pattern: "",
      strategy: "placeholder" as const,
      mode: "filter" as const,
      priority: 100,
      enabled: true,
      category: ruleFiles[0] || "custom",
    },
  });

  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen) {
      if (editing) {
        reset({
          id: editing.id,
          name: editing.name,
          pattern: editing.pattern,
          strategy: editing.strategy as "placeholder" | "mask",
          mode: (editing.mode || "filter") as "filter" | "detect",
          priority: editing.priority,
          enabled: editing.enabled,
          category: editing.category,
        });
      } else {
        reset({
          id: "",
          name: "",
          pattern: "",
          strategy: "placeholder",
          mode: "filter",
          priority: 100,
          enabled: true,
          category: ruleFiles[0] || "custom",
        });
      }
    }
  };

  const onSubmit = async (values: any) => {
    const parsed = schema.safeParse(values);
    if (!parsed.success) return;
    const v = parsed.data;
    const rule: RuleDef = {
      id: v.id,
      name: v.name,
      pattern: v.pattern,
      strategy: v.strategy,
      mode: v.mode,
      priority: v.priority,
      enabled: v.enabled,
    };
    await onSave(rule, v.category);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[560px]">
        <DialogHeader>
          <DialogTitle>{isEdit ? t("Edit Rule") : t("Add Rule")}</DialogTitle>
        </DialogHeader>

        <form id="rule-form" onSubmit={handleSubmit(onSubmit)} className="space-y-4 mt-4 max-h-[60vh] overflow-auto pr-2">
          <div className="space-y-2">
            <Label htmlFor="id">{t("Rule ID")}</Label>
            <Controller
              name="id"
              control={control}
              render={({ field }) => (
                <Input id="id" placeholder={t("e.g. phone_cn")} {...field} />
              )}
            />
            {errors.id && <p className="text-xs text-destructive">{errors.id.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="name">{t("Name")}</Label>
            <Controller
              name="name"
              control={control}
              render={({ field }) => (
                <Input id="name" placeholder={t("e.g. Chinese Phone Number")} {...field} />
              )}
            />
            {errors.name && <p className="text-xs text-destructive">{errors.name.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="pattern">{t("Regex Pattern")}</Label>
            <Controller
              name="pattern"
              control={control}
              render={({ field }) => (
                <Textarea id="pattern" rows={3} placeholder={t("e.g. 1[3-9]\\d{9}")} {...field} />
              )}
            />
            {errors.pattern && <p className="text-xs text-destructive">{errors.pattern.message}</p>}
          </div>

          <div className="space-y-2">
            <Label>{t("Strategy")}</Label>
            <Controller
              name="strategy"
              control={control}
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="placeholder">{t("Placeholder — Replace Entire Match")}</SelectItem>
                    <SelectItem value="mask">{t("Mask — Partial Masking")}</SelectItem>
                  </SelectContent>
                </Select>
              )}
            />
          </div>

          <div className="space-y-2">
            <Label>{t("Mode")}</Label>
            <Controller
              name="mode"
              control={control}
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="filter">{t("Filter — Detect and Replace")}</SelectItem>
                    <SelectItem value="detect">{t("Detect — Log Only, No Replacement")}</SelectItem>
                  </SelectContent>
                </Select>
              )}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="priority">{t("Priority")}</Label>
            <Controller
              name="priority"
              control={control}
              render={({ field }) => (
                <Input
                  id="priority"
                  type="number"
                  min={1}
                  max={999}
                  value={field.value as any}
                  onChange={(e) => field.onChange(e.target.value)}
                  onBlur={field.onBlur}
                  name={field.name}
                  ref={field.ref}
                />
              )}
            />
          </div>

          <div className="flex items-center gap-3">
            <Controller
              name="enabled"
              control={control}
              render={({ field }) => (
                <Switch checked={field.value} onCheckedChange={field.onChange} />
              )}
            />
            <Label>{t("Enable")}</Label>
          </div>

          <div className="space-y-2">
            <Label>{t("Category")}</Label>
            <Controller
              name="category"
              control={control}
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {ruleFiles.map((f) => (
                      <SelectItem key={f} value={f}>{f}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
            {errors.category && <p className="text-xs text-destructive">{errors.category.message}</p>}
          </div>
        </form>

        <DialogFooter>
          <Button variant="outline" onClick={onCancel}>{t("Cancel")}</Button>
          <Button type="submit" form="rule-form">{t("Save")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
