import { invoke } from "@tauri-apps/api/core";
import type { ProxyStatus } from "../types";

export const startProxy = (): Promise<string> => invoke("start_proxy");
export const stopProxy = (): Promise<string> => invoke("stop_proxy");
export const getProxyStatus = (): Promise<ProxyStatus> => invoke("get_proxy_status");
