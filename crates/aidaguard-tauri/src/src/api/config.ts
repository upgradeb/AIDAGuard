import { invoke } from "@tauri-apps/api/core";
import type { Config, RegionInfo } from "../types";

export const getAppVersion = (): Promise<string> => invoke("get_app_version");

export const getConfig = (): Promise<Config> => invoke("get_config");

export const saveConfig = (config: Config): Promise<void> =>
  invoke("save_config", { config });

export const getAvailableRegions = (): Promise<RegionInfo[]> =>
  invoke("get_available_regions");

export const updateDetectionRegion = (
  primaryRegion: string,
  additionalRegions: string[],
): Promise<void> =>
  invoke("update_detection_region", {
    primaryRegion,
    additionalRegions,
  });
