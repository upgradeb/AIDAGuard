import { invoke } from "@tauri-apps/api/core";
import type { RuleDef, TestRuleResult } from "../types";

export interface RuleWithCategory extends RuleDef {
  category: string;
}

export const getRules = (): Promise<RuleWithCategory[]> => invoke("get_rules");

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
