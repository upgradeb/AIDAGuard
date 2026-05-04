import { invoke } from "@tauri-apps/api/core";
import type { ToolInfo } from "../types";

export async function detectTools(): Promise<ToolInfo[]> {
  return invoke("detect_tools");
}

export async function applyToolConfig(toolId: string): Promise<string> {
  return invoke("apply_tool_config", { toolId });
}

export async function restoreToolConfig(toolId: string): Promise<string> {
  return invoke("restore_tool_config", { toolId });
}

export async function restoreAllTools(): Promise<string> {
  return invoke("restore_all_tools");
}
