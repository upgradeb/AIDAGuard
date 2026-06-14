import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";
import {
  AlertDialog,
  AlertDialogTrigger,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogAction,
  AlertDialogCancel,
} from "@/components/ui/alert-dialog";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Alert, AlertTitle, AlertDescription } from "@/components/ui/alert";
import { toast } from "sonner";
import {
  Plus,
  Pencil,
  Trash2,
  RefreshCw,
  FlaskConical,
  FolderOpen,
  Settings,
  Bot,
  Search,
  CircleAlert,
  TriangleAlert,
  Info,
  X,
} from "lucide-react";
import { useRulesStore } from "../store/useRulesStore";
import { useUpstreamStore } from "../store/useUpstreamStore";
import RuleEditor from "../components/RuleEditor";
import RuleTestPanel from "../components/RuleTestPanel";
import GenerateRuleModal from "../components/GenerateRuleModal";
import type { RuleWithCategory } from "../api/rules";
import type { RuleDef } from "../types";

function groupByCategory(rules: RuleWithCategory[]): Map<string, RuleWithCategory[]> {
  const map = new Map<string, RuleWithCategory[]>();
  for (const r of rules) {
    const cat = r.category || "未分类";
    if (!map.has(cat)) map.set(cat, []);
    map.get(cat)!.push(r);
  }
  return map;
}

export default function Rules() {
  const { t } = useTranslation();
  const rules = useRulesStore((s) => s.rules);
  const ruleFiles = useRulesStore((s) => s.ruleFiles);
  const rulesDir = useRulesStore((s) => s.rulesDir);
  const error = useRulesStore((s) => s.error);
  const loading = useRulesStore((s) => s.loading);
  const testing = useRulesStore((s) => s.testing);
  const testResult = useRulesStore((s) => s.testResult);
  const fetchRules = useRulesStore((s) => s.fetchRules);
  const save = useRulesStore((s) => s.saveRule);
  const remove = useRulesStore((s) => s.deleteRule);
  const toggle = useRulesStore((s) => s.toggleRule);
  const test = useRulesStore((s) => s.testRule);
  const reload = useRulesStore((s) => s.reloadRules);
  const clearTestResult = useRulesStore((s) => s.clearTestResult);
  const createCat = useRulesStore((s) => s.createCategory);
  const deleteCat = useRulesStore((s) => s.deleteCategory);
  const renameCat = useRulesStore((s) => s.renameCategory);
  const upstreams = useUpstreamStore((s) => s.upstreams);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);

  const defaultUpstream = upstreams.find((u) => u.default) || upstreams[0];
  const defaultModelLabel = defaultUpstream
    ? `${defaultUpstream.name} / ${defaultUpstream.models?.[0] || "—"}`
    : t("Not configured (add in LLM Upstreams first)");

  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRule, setEditingRule] = useState<RuleWithCategory | null>(null);
  const [testOpen, setTestOpen] = useState(false);
  const [generateOpen, setGenerateOpen] = useState(false);
  const [filterCat, setFilterCat] = useState<string>("");
  const [searchText, setSearchText] = useState("");
  const [catModalOpen, setCatModalOpen] = useState(false);
  const [newCatName, setNewCatName] = useState("");
  const [renameTarget, setRenameTarget] = useState<string | null>(null);
  const [renameNewName, setRenameNewName] = useState("");

  useEffect(() => {
    fetchRules();
    fetchUpstreams();
  }, []);

  const filtered = rules.filter((r) => {
    if (filterCat && r.category !== filterCat) return false;
    if (searchText && !r.id.includes(searchText) && !r.name.includes(searchText))
      return false;
    return true;
  });

  const grouped = groupByCategory(filtered);
  const categories = Array.from(grouped.keys()).sort();

  const handleSave = async (rule: RuleDef, category: string) => {
    try {
      await save(rule, category);
      toast.success(t("Rule Saved"));
      setEditorOpen(false);
      setEditingRule(null);
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleDelete = async (ruleId: string, category: string) => {
    try {
      await remove(ruleId, category);
      toast.success(t("Rule Deleted"));
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleToggleMode = async (record: RuleWithCategory) => {
    const newMode = record.mode === "filter" ? "detect" : "filter";
    const updated: RuleDef = {
      id: record.id,
      name: record.name,
      pattern: record.pattern,
      strategy: record.strategy,
      mode: newMode,
      priority: record.priority,
      enabled: record.enabled,
    };
    try {
      await save(updated, record.category);
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleCreateCategory = async () => {
    const name = newCatName.trim();
    if (!name) return;
    try {
      await createCat(name);
      toast.success(t("Category {{name}} Created", { name }));
      setNewCatName("");
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleDeleteCategory = async (name: string) => {
    try {
      await deleteCat(name);
      toast.success(t("Category {{name}} Deleted", { name }));
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleRenameCategory = async () => {
    if (!renameTarget) return;
    const newName = renameNewName.trim();
    if (!newName) return;
    try {
      await renameCat(renameTarget, newName);
      toast.success(t("Renamed to {{newName}}", { newName }));
      setRenameTarget(null);
      setRenameNewName("");
      fetchRules();
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleBulkToggleEnabled = async (category: string, enabled: boolean) => {
    const catRules = rules.filter((r) => r.category === category);
    for (const r of catRules) {
      const updated: RuleDef = {
        id: r.id, name: r.name, pattern: r.pattern,
        strategy: r.strategy, mode: r.mode, priority: r.priority,
        enabled,
      };
      try { await save(updated, category); } catch { /* continue */ }
    }
    fetchRules();
  };

  const handleBulkToggleMode = async (category: string, mode: "detect" | "filter") => {
    const catRules = rules.filter((r) => r.category === category);
    for (const r of catRules) {
      const updated: RuleDef = {
        id: r.id, name: r.name, pattern: r.pattern,
        strategy: r.strategy, mode, priority: r.priority,
        enabled: r.enabled,
      };
      try { await save(updated, category); } catch { /* continue */ }
    }
    fetchRules();
  };

  return (
    <div className="flex h-full flex-col gap-3">
      {/* Top info bar + toolbar */}
      <div className="shrink-0">
        {error && (
          <Alert variant="destructive" className="mb-3">
            <CircleAlert className="h-4 w-4" />
            <AlertTitle>{t("Rule Loading Failed")}</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {!loading && !error && rules.length === 0 && (
          <Alert className="mb-3 border-yellow-500/50 text-yellow-700 dark:text-yellow-400 [&>svg]:text-yellow-500">
            <TriangleAlert className="h-4 w-4" />
            <AlertTitle>{t("No Rule Files Found")}</AlertTitle>
            <AlertDescription>
              <span>
                {t("Rules Directory: ")}<code>{rulesDir || t("Unknown")}</code>
                {t(". Ensure ")}<code>.yaml</code>{t(" rule files exist in the directory, or change the rules directory in Settings.")}
              </span>
            </AlertDescription>
          </Alert>
        )}

        {rulesDir && (
          <div className="mb-3 flex items-center gap-4 rounded-md bg-muted px-3 py-1.5 text-xs">
            <span className="flex items-center gap-1">
              <FolderOpen className="h-3.5 w-3.5" />
              {t("Rules Directory: ")}<code className="text-xs">{rulesDir}</code>
            </span>
            <span className="text-muted-foreground">
              {t("{{ruleCount}} Rules · {{fileCount}} Files", { ruleCount: rules.length, fileCount: ruleFiles.length })}
            </span>
          </div>
        )}

        <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-2">
            <Select
              value={filterCat || undefined}
              onValueChange={(v) => setFilterCat(v || "")}
            >
              <SelectTrigger className="w-[170px]">
                <SelectValue placeholder={t("Filter by Category")} />
              </SelectTrigger>
              <SelectContent>
                {ruleFiles.map((f) => (
                  <SelectItem key={f} value={f}>
                    {f}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <div className="relative">
              <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder={t("Search Rule Name / ID")}
                className="w-[260px] pl-8"
                value={searchText}
                onChange={(e) => setSearchText(e.target.value)}
              />
              {searchText && (
                <button
                  className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  onClick={() => setSearchText("")}
                >
                  <X className="h-3.5 w-3.5" />
                </button>
              )}
            </div>
            <Button variant="outline" size="sm" onClick={reload}>
              <RefreshCw className="mr-1.5 h-4 w-4" />
              {t("Reload Rules")}
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm" onClick={() => setCatModalOpen(true)}>
              <Settings className="mr-1.5 h-4 w-4" />
              {t("Manage Categories")}
            </Button>
            <Button variant="outline" size="sm" onClick={() => setGenerateOpen(true)}>
              <Bot className="mr-1.5 h-4 w-4" />
              {t("Generate Rule")}
            </Button>
            <Button size="sm" onClick={() => {
              setEditingRule(null);
              setEditorOpen(true);
            }}>
              <Plus className="mr-1.5 h-4 w-4" />
              {t("Add Rule")}
            </Button>
          </div>
        </div>
      </div>

      {/* Rule details — scrollable area */}
      <div className="min-h-0 flex-1 overflow-auto">
        {categories.map((cat) => {
          const catRules = grouped.get(cat)!;
          const allEnabled = catRules.every((r) => r.enabled);
          const allFilter = catRules.every((r) => r.mode === "filter");
          return (
            <Card key={cat} className="mb-3">
              <CardHeader className="flex flex-row items-center justify-between space-y-0 px-4 py-3">
                <div className="flex items-center gap-2">
                  <Badge variant="default" className="bg-green-600 hover:bg-green-700">{cat}</Badge>
                  <span className="text-xs text-muted-foreground">
                    {t("{{count}} Rules", { count: catRules.length })}
                  </span>
                </div>
                <div className="flex items-center gap-3">
                  <div className="flex items-center gap-1.5">
                    <span className="text-[11px] text-muted-foreground">{t("Enable")}</span>
                    <Switch
                      className="h-4 w-7 [&>span]:h-3 [&>span]:w-3 [&>span]:data-[state=checked]:translate-x-3"
                      checked={allEnabled}
                      onCheckedChange={(v) => handleBulkToggleEnabled(cat, v)}
                    />
                  </div>
                  <div className="flex items-center gap-1.5">
                    <span className="text-[11px] text-muted-foreground">{t("Filter")}</span>
                    <Switch
                      className="h-4 w-7 [&>span]:h-3 [&>span]:w-3 [&>span]:data-[state=checked]:translate-x-3"
                      checked={allFilter}
                      onCheckedChange={(v) => handleBulkToggleMode(cat, v ? "filter" : "detect")}
                    />
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 text-xs"
                    onClick={() => {
                      setRenameTarget(cat);
                      setRenameNewName(cat);
                    }}
                  >
                    {t("Rename")}
                  </Button>
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button variant="ghost" size="sm" className="h-7 text-xs text-destructive hover:text-destructive">
                        {t("Delete")}
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>{t("Confirm Delete")}</AlertDialogTitle>
                        <AlertDialogDescription>
                          {t("Delete category {{cat}}? All rules in this category will also be deleted.", { cat })}
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
                        <AlertDialogAction
                          className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                          onClick={() => handleDeleteCategory(cat)}
                        >
                          {t("Delete")}
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </div>
              </CardHeader>
              <CardContent className="px-0 pb-0">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[60px]">{t("Enable")}</TableHead>
                      <TableHead className="w-[80px]">{t("Mode")}</TableHead>
                      <TableHead className="w-[150px]">{t("Rule Name")}</TableHead>
                      <TableHead className="w-[150px]">{t("ID")}</TableHead>
                      <TableHead>{t("Pattern")}</TableHead>
                      <TableHead className="w-[110px]">{t("Strategy")}</TableHead>
                      <TableHead className="w-[80px]">{t("Priority")}</TableHead>
                      <TableHead className="w-[130px]">{t("Actions")}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {catRules.map((record) => (
                      <TableRow key={record.id}>
                        <TableCell>
                          <Switch
                            className="h-4 w-7 [&>span]:h-3 [&>span]:w-3 [&>span]:data-[state=checked]:translate-x-3"
                            checked={record.enabled}
                            onCheckedChange={(checked) => {
                              toggle(record.id, checked);
                              fetchRules();
                            }}
                          />
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-1.5">
                            <Switch
                              className="h-4 w-7 [&>span]:h-3 [&>span]:w-3 [&>span]:data-[state=checked]:translate-x-3"
                              checked={record.mode === "filter"}
                              onCheckedChange={() => handleToggleMode(record)}
                            />
                            <span className="text-xs text-muted-foreground">
                              {record.mode === "filter" ? t("Filter") : t("Detect")}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell className="font-medium">{record.name}</TableCell>
                        <TableCell>
                          <Badge variant="secondary">{record.id}</Badge>
                        </TableCell>
                        <TableCell>
                          <code className="text-xs">{record.pattern}</code>
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant="secondary"
                            className={record.strategy === "placeholder" ? "bg-blue-100 text-blue-700 hover:bg-blue-200 dark:bg-blue-900 dark:text-blue-300" : "bg-purple-100 text-purple-700 hover:bg-purple-200 dark:bg-purple-900 dark:text-purple-300"}
                          >
                            {record.strategy}
                          </Badge>
                        </TableCell>
                        <TableCell>{record.priority}</TableCell>
                        <TableCell>
                          <div className="flex items-center gap-1">
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7"
                              onClick={() => {
                                setEditingRule(record);
                                setEditorOpen(true);
                              }}
                            >
                              <Pencil className="h-3.5 w-3.5" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7"
                              onClick={() => {
                                setTestOpen(true);
                                clearTestResult();
                              }}
                            >
                              <FlaskConical className="h-3.5 w-3.5" />
                            </Button>
                            <AlertDialog>
                              <AlertDialogTrigger asChild>
                                <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive hover:text-destructive">
                                  <Trash2 className="h-3.5 w-3.5" />
                                </Button>
                              </AlertDialogTrigger>
                              <AlertDialogContent>
                                <AlertDialogHeader>
                                  <AlertDialogTitle>{t("Confirm Delete")}</AlertDialogTitle>
                                  <AlertDialogDescription>
                                    {t("Delete this rule?")}
                                  </AlertDialogDescription>
                                </AlertDialogHeader>
                                <AlertDialogFooter>
                                  <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
                                  <AlertDialogAction
                                    className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                    onClick={() => handleDelete(record.id, record.category)}
                                  >
                                    {t("Delete")}
                                  </AlertDialogAction>
                                </AlertDialogFooter>
                              </AlertDialogContent>
                            </AlertDialog>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          );
        })}

        {filtered.length === 0 && !loading && (
          <Alert className="border-blue-500/50 text-blue-700 dark:text-blue-400 [&>svg]:text-blue-500">
            <Info className="h-4 w-4" />
            <AlertTitle>{t("No Matching Rules")}</AlertTitle>
          </Alert>
        )}
      </div>

      <RuleEditor
        open={editorOpen}
        editing={editingRule}
        ruleFiles={ruleFiles}
        onSave={handleSave}
        onCancel={() => {
          setEditorOpen(false);
          setEditingRule(null);
        }}
      />

      <RuleTestPanel
        open={testOpen}
        testing={testing}
        result={testResult}
        onTest={test}
        onClose={() => {
          setTestOpen(false);
          clearTestResult();
        }}
      />

      <GenerateRuleModal
        open={generateOpen}
        defaultModelLabel={defaultModelLabel}
        onApply={(rule) => {
          setEditingRule({
            id: rule.id || "",
            name: rule.name,
            pattern: rule.pattern,
            strategy: rule.strategy as "placeholder" | "mask",
            mode: rule.mode as "detect" | "filter",
            priority: rule.priority,
            enabled: true,
            category: ruleFiles[0] || "custom",
          });
          setEditorOpen(true);
        }}
        onClose={() => setGenerateOpen(false)}
      />

      {/* Category management dialog */}
      <Dialog open={catModalOpen} onOpenChange={(open) => {
        if (!open) setNewCatName("");
        setCatModalOpen(open);
      }}>
        <DialogContent className="sm:max-w-[480px]">
          <DialogHeader>
            <DialogTitle>{t("Manage Categories")}</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col gap-4">
            <div className="flex gap-2">
              <Input
                placeholder={t("Enter new category name (letters, digits, _, -)")}
                value={newCatName}
                onChange={(e) => setNewCatName(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") handleCreateCategory(); }}
                className="flex-1"
              />
              <Button onClick={handleCreateCategory}>
                {t("Create")}
              </Button>
            </div>

            <div>
              <p className="mb-2 text-sm font-semibold">{t("Existing Categories")}</p>
              {ruleFiles.map((f) => {
                const catRules = rules.filter((r) => r.category === f);
                return (
                  <div
                    key={f}
                    className="flex items-center justify-between border-b py-2 last:border-b-0"
                  >
                    <div className="flex items-center gap-2">
                      <Badge variant="default" className="bg-green-600 hover:bg-green-700">{f}</Badge>
                      <span className="text-xs text-muted-foreground">
                        {t("{{count}} Rules", { count: catRules.length })}
                      </span>
                    </div>
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button variant="ghost" size="icon" className="h-7 w-7 text-destructive hover:text-destructive">
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>{t("Confirm Delete")}</AlertDialogTitle>
                          <AlertDialogDescription>
                            {t("Delete category {{cat}}? All rules in this category will also be deleted.", { cat: f })}
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
                          <AlertDialogAction
                            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                            onClick={() => handleDeleteCategory(f)}
                          >
                            {t("Delete")}
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                );
              })}
              {ruleFiles.length === 0 && (
                <p className="text-sm text-muted-foreground">{t("No Categories")}</p>
              )}
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Rename category dialog */}
      <Dialog open={!!renameTarget} onOpenChange={(open) => {
        if (!open) {
          setRenameTarget(null);
          setRenameNewName("");
        }
      }}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>{t("Rename Category: {{name}}", { name: renameTarget || "" })}</DialogTitle>
          </DialogHeader>
          <Input
            placeholder={t("Enter New Name")}
            value={renameNewName}
            onChange={(e) => setRenameNewName(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") handleRenameCategory(); }}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => {
              setRenameTarget(null);
              setRenameNewName("");
            }}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleRenameCategory}>
              {t("Save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
