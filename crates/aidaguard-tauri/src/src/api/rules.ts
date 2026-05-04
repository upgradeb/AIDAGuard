import { invoke } from "@tauri-apps/api/core";
import type { RuleDef, TestRuleResult } from "../types";

export interface RuleWithCategory extends RuleDef {
  category: string;
}

export interface GetRulesResponse {
  rules: RuleWithCategory[];
  files: string[];
  rulesDir: string;
}

export const getRules = (): Promise<GetRulesResponse> => invoke("get_rules");

export const saveRule = (rule: RuleDef, category: string): Promise<void> =>
  invoke("save_rule", { rule, category });

export const deleteRule = (ruleId: string, category: string): Promise<void> =>
  invoke("delete_rule", { ruleId, category });

export const toggleRule = (ruleId: string, enabled: boolean): Promise<void> =>
  invoke("toggle_rule", { ruleId, enabled });

export const testRule = (
  pattern: string,
  testText: string
): Promise<TestRuleResult> => invoke("test_rule", { pattern, testText });

export const reloadRules = (): Promise<string> => invoke("reload_rules");

export const getRuleFiles = (): Promise<string[]> => invoke("get_rule_files");

export const createCategory = (name: string): Promise<void> =>
  invoke("create_category", { name });

export const deleteCategory = (name: string): Promise<void> =>
  invoke("delete_category", { name });

export const renameCategory = (oldName: string, newName: string): Promise<void> =>
  invoke("rename_category", { oldName, newName });

export interface GeneratedRule {
  name: string;
  pattern: string;
  strategy: string;
  mode: string;
  priority: number;
}

export const generateRule = (sampleText: string): Promise<GeneratedRule> =>
  invoke("generate_rule", { sampleText });
