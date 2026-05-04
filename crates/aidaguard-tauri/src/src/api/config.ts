import { invoke } from "@tauri-apps/api/core";
import type { Config } from "../types";

export const getAppVersion = (): Promise<string> => invoke("get_app_version");

export const getConfig = (): Promise<Config> => invoke("get_config");

export const saveConfig = (config: Config): Promise<void> =>
  invoke("save_config", { config });
