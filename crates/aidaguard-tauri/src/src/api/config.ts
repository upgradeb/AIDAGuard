import { invoke } from "@tauri-apps/api/core";
import type { Config } from "../types";

export const getConfig = (): Promise<Config> => invoke("get_config");

export const saveConfig = (config: Config): Promise<void> =>
  invoke("save_config", { config });
