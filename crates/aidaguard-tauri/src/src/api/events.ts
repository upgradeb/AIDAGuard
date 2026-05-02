import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DetectionEvent } from "../types";

export const onDetectionEvent = (
  cb: (e: DetectionEvent) => void
): Promise<UnlistenFn> =>
  listen<DetectionEvent>("detection-event", (event) => cb(event.payload));

export const onProxyStatusChanged = (
  cb: (status: { status: "running" | "stopped" | "error" }) => void
): Promise<UnlistenFn> =>
  listen<{ status: string }>("proxy-status-changed", (event) =>
    cb(event.payload as { status: "running" | "stopped" | "error" })
  );
