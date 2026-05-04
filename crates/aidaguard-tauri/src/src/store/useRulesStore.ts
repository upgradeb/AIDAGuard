import { create } from "zustand";
import {
  getRules,
  saveRule,
  deleteRule,
  toggleRule,
  testRule,
  reloadRules,
  getRuleFiles,
  createCategory,
  deleteCategory,
  renameCategory,
} from "../api/rules";
import type { RuleWithCategory } from "../api/rules";
import type { RuleDef, TestRuleResult } from "../types";

interface RulesState {
  rules: RuleWithCategory[];
  ruleFiles: string[];
  rulesDir: string;
  testResult: TestRuleResult | null;
  loading: boolean;
  saving: boolean;
  testing: boolean;
  error: string | null;

  fetchRules: () => Promise<void>;
  saveRule: (rule: RuleDef, category: string) => Promise<void>;
  deleteRule: (ruleId: string, category: string) => Promise<void>;
  toggleRule: (ruleId: string, enabled: boolean) => Promise<void>;
  testRule: (pattern: string, testText: string) => Promise<void>;
  reloadRules: () => Promise<void>;
  fetchRuleFiles: () => Promise<void>;
  clearTestResult: () => void;
  createCategory: (name: string) => Promise<void>;
  deleteCategory: (name: string) => Promise<void>;
  renameCategory: (oldName: string, newName: string) => Promise<void>;
}

export const useRulesStore = create<RulesState>((set) => ({
  rules: [],
  ruleFiles: [],
  rulesDir: "",
  testResult: null,
  loading: false,
  saving: false,
  testing: false,
  error: null,

  fetchRules: async () => {
    set({ loading: true, error: null });
    try {
      const resp = await getRules();
      set({
        rules: resp.rules,
        ruleFiles: resp.files,
        rulesDir: resp.rulesDir,
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  saveRule: async (rule, category) => {
    set({ saving: true, error: null });
    try {
      await saveRule(rule, category);
      set({ saving: false });
    } catch (e) {
      set({ error: String(e), saving: false });
      throw e;
    }
  },

  deleteRule: async (ruleId, category) => {
    set({ error: null });
    try {
      await deleteRule(ruleId, category);
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  toggleRule: async (ruleId, enabled) => {
    try {
      await toggleRule(ruleId, enabled);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  testRule: async (pattern, testText) => {
    set({ testing: true, error: null });
    try {
      const result = await testRule(pattern, testText);
      set({ testResult: result, testing: false });
    } catch (e) {
      set({ error: String(e), testing: false });
    }
  },

  reloadRules: async () => {
    try {
      await reloadRules();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchRuleFiles: async () => {
    try {
      const files = await getRuleFiles();
      set({ ruleFiles: files });
    } catch (_) {}
  },

  clearTestResult: () => set({ testResult: null }),

  createCategory: async (name) => {
    try {
      await createCategory(name);
    } catch (e) {
      throw e;
    }
  },

  deleteCategory: async (name) => {
    try {
      await deleteCategory(name);
    } catch (e) {
      throw e;
    }
  },

  renameCategory: async (oldName, newName) => {
    try {
      await renameCategory(oldName, newName);
    } catch (e) {
      throw e;
    }
  },
}));
