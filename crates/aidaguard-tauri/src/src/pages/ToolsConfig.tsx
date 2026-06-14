import { useEffect, useMemo } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
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
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { toast } from "sonner";
import {
  Settings,
  Undo2,
  CheckCircle2,
  AlertTriangle,
  Server,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useToolsStore } from "../store/useToolsStore";
import { useProxyStore } from "../store/useProxyStore";

export default function ToolsConfig() {
  const { t } = useTranslation();
  const tools = useToolsStore((s) => s.tools);
  const loading = useToolsStore((s) => s.loading);
  const applying = useToolsStore((s) => s.applying);
  const error = useToolsStore((s) => s.error);
  const fetchTools = useToolsStore((s) => s.fetchTools);
  const apply = useToolsStore((s) => s.apply);
  const restore = useToolsStore((s) => s.restore);
  const restoreAll = useToolsStore((s) => s.restoreAll);
  const togglePlugin = useToolsStore((s) => s.togglePlugin);
  const proxyStatus = useProxyStore((s) => s.status);

  const sortedTools = useMemo(
    () =>
      [...tools].sort((a, b) => {
        if (a.installed !== b.installed) return a.installed ? -1 : 1;
        if (a.configured !== b.configured) return a.configured ? -1 : 1;
        return a.toolName.localeCompare(b.toolName);
      }),
    [tools]
  );

  useEffect(() => {
    fetchTools();
  }, []);

  const isRunning = proxyStatus?.status === "running";
  const proxyUrl = `http://127.0.0.1:${proxyStatus?.port || 19000}`;

  const handleRescan = async () => {
    await fetchTools();
    const err = useToolsStore.getState().error;
    if (err) {
      toast.error(err);
    } else {
      toast.success(t("Rescan Complete"));
    }
  };

  const handleApply = async (toolId: string) => {
    try {
      await apply(toolId);
      toast.success(t("Configuration Applied"));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleRestore = async (toolId: string) => {
    try {
      await restore(toolId);
      toast.success(t("Configuration Restored"));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleTogglePlugin = async (toolId: string) => {
    try {
      await togglePlugin(toolId);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleRestoreAll = async () => {
    try {
      await restoreAll();
      toast.success(t("All Configurations Restored"));
    } catch (e) {
      toast.error(String(e));
    }
  };

  const configuredCount = sortedTools.filter((t) => t.configured).length;
  const installedCount = sortedTools.filter((t) => t.installed).length;

  return (
    <div className="h-full overflow-auto">
      {/* Warning alert */}
      {!isRunning && (
        <Alert className="mb-4 border-yellow-500/50 text-yellow-700 dark:text-yellow-400 [&>svg]:text-yellow-600">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>{t("Proxy Not Started")}</AlertTitle>
          <AlertDescription>
            {t(
              "Start the proxy to preview configuration effects below. Configuration will redirect all requests to the local proxy address."
            )}
          </AlertDescription>
        </Alert>
      )}
      {error && (
        <Alert variant="destructive" className="mb-4">
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle>{t("Operation Failed")}</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Overview card */}
      <Card className="mb-4">
        <CardContent className="p-4">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <div className="flex items-center gap-3">
              <span className="font-semibold">
                {t(
                  "{{installedCount}}/{{totalCount}} Tools Detected",
                  { installedCount, totalCount: sortedTools.length }
                )}
              </span>
              {configuredCount > 0 && (
                <Badge variant="secondary" className="bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                  {t("{{configuredCount}} Configured", { configuredCount })}
                </Badge>
              )}
              <Badge variant="outline" className="gap-1">
                <Server className="h-3 w-3" /> {proxyUrl}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleRescan}
                disabled={loading}
              >
                {loading && (
                  <span className="mr-1 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                )}
                {t("Rescan")}
              </Button>
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button variant="destructive" size="sm">
                    <Undo2 className="h-4 w-4" />
                    {t("Restore All")}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>
                      {t("Restore all tools to their original configuration?")}
                    </AlertDialogTitle>
                    <AlertDialogDescription>
                      {t(
                        "This will undo all configuration changes made by Aidaguard."
                      )}
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
                    <AlertDialogAction onClick={handleRestoreAll}>
                      {t("OK")}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tool list card */}
      <Card>
        <CardContent className="p-0">
          {loading && sortedTools.length === 0 ? (
            <div className="flex items-center justify-center py-12 text-muted-foreground">
              <span className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              {t("Loading...")}
            </div>
          ) : (
            <div className="divide-y">
              {sortedTools.map((tool) => (
                <div
                  key={tool.toolId}
                  className="flex items-center gap-4 px-5 py-3.5"
                >
                  {/* Status icon */}
                  <div className="mt-0.5 shrink-0">
                    {tool.installed ? (
                      tool.configured ? (
                        <CheckCircle2 className="h-5 w-5 text-green-600" />
                      ) : (
                        <AlertTriangle className="h-5 w-5 text-yellow-600" />
                      )
                    ) : (
                      <span className="block h-5 w-5 text-center text-muted-foreground/50">
                        &mdash;
                      </span>
                    )}
                  </div>

                  {/* Title + description */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">{tool.toolName}</span>
                      {tool.installed ? (
                        <Badge
                          variant="secondary"
                          className="bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300"
                        >
                          {t("Installed")}
                        </Badge>
                      ) : (
                        <Badge variant="outline">{t("Not Installed")}</Badge>
                      )}
                      {tool.configured && (
                        <Badge
                          variant="secondary"
                          className="bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300"
                        >
                          {t("Configured")}
                        </Badge>
                      )}
                    </div>
                    <div className="mt-0.5 text-sm">
                      <div className="text-muted-foreground">
                        {t("Config File: ")}
                        <code className="rounded bg-muted px-1 py-0.5 text-xs">
                          {tool.configPath}
                        </code>
                      </div>
                      {!tool.enabled && (
                        <div className="mt-0.5 font-medium text-yellow-600">
                          {t("Plugin is disabled — Click toggle to re-enable")}
                        </div>
                      )}
                      {tool.enabled && tool.configured && (
                        <div className="mt-0.5 font-medium text-primary">
                          {t(
                            "Proxied — Requests will be scanned by Aidaguard for sensitive data"
                          )}
                        </div>
                      )}
                      {tool.enabled && tool.installed && !tool.configured && (
                        <div className="mt-0.5 font-medium text-yellow-600">
                          {t(
                            'Not Proxied — Click "Configure" to route through local proxy'
                          )}
                        </div>
                      )}
                      {tool.enabled && !tool.installed && (
                        <div className="mt-0.5 text-muted-foreground/60">
                          {t(
                            "Install this tool to enable one-click configuration"
                          )}
                        </div>
                      )}
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex shrink-0 items-center gap-2">
                    {tool.installed && (
                      <Button
                        size="sm"
                        onClick={() => handleApply(tool.toolId)}
                        disabled={applying === tool.toolId}
                      >
                        {applying === tool.toolId ? (
                          <span className="mr-1 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                        ) : (
                          <Settings className="h-4 w-4" />
                        )}
                        {t("Configure")}
                      </Button>
                    )}
                    {tool.installed ? (
                      <AlertDialog>
                        <AlertDialogTrigger asChild>
                          <Button
                            variant="outline"
                            size="sm"
                            disabled={applying === tool.toolId}
                          >
                            {applying === tool.toolId ? (
                              <span className="mr-1 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                            ) : (
                              <Undo2 className="h-4 w-4" />
                            )}
                            {t("Restore")}
                          </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                          <AlertDialogHeader>
                            <AlertDialogTitle>
                              {t("Restore original configuration?")}
                            </AlertDialogTitle>
                            <AlertDialogDescription>
                              {t(
                                "This will revert the configuration file to its original state."
                              )}
                            </AlertDialogDescription>
                          </AlertDialogHeader>
                          <AlertDialogFooter>
                            <AlertDialogCancel>
                              {t("Cancel")}
                            </AlertDialogCancel>
                            <AlertDialogAction
                              onClick={() => handleRestore(tool.toolId)}
                            >
                              {t("OK")}
                            </AlertDialogAction>
                          </AlertDialogFooter>
                        </AlertDialogContent>
                      </AlertDialog>
                    ) : (
                      <span className="text-xs text-muted-foreground">
                        {t("Not Installed")}
                      </span>
                    )}
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <div>
                            <Switch
                              checked={tool.enabled}
                              onCheckedChange={() =>
                                handleTogglePlugin(tool.toolId)
                              }
                            />
                          </div>
                        </TooltipTrigger>
                        <TooltipContent>
                          {tool.enabled
                            ? t("Disable Plugin")
                            : t("Enable Plugin")}
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
