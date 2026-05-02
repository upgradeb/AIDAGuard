import { invoke } from "@tauri-apps/api/core";
import type { UpstreamConfig } from "../types";

export const getUpstreams = (): Promise<UpstreamConfig[]> =>
  invoke("get_upstreams");

export const addUpstream = (upstream: UpstreamConfig): Promise<void> =>
  invoke("add_upstream", { upstream });

export const updateUpstream = (
  name: string,
  upstream: UpstreamConfig
): Promise<void> => invoke("update_upstream", { name, upstream });

export const deleteUpstream = (name: string): Promise<void> =>
  invoke("delete_upstream", { name });

export const setDefaultUpstream = (name: string): Promise<void> =>
  invoke("set_default_upstream", { name });

export const testConnectivity = (
  url: string,
  apiKey: string,
  timeoutSecs: number
): Promise<string> =>
  invoke("test_upstream_connectivity", {
    url,
    apiKey,
    timeoutSecs,
  });
