import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { onDetectionEvent } from "../api/events";
import i18n from "../i18n";
import type { DetectionEvent } from "../types";

const cooldownMap = new Map<string, number>();

export function useNotification() {
  const navigate = useNavigate();
  const navigateRef = useRef(navigate);
  navigateRef.current = navigate;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let permitted = false;

    if ("Notification" in window && Notification.permission === "granted") {
      permitted = true;
    } else if ("Notification" in window && Notification.permission === "default") {
      Notification.requestPermission().then((p) => {
        permitted = p === "granted";
      });
    }

    onDetectionEvent((event: DetectionEvent) => {
      if (!permitted) return;

      const now = Date.now();
      const last = cooldownMap.get(event.ruleId) || 0;
      if (now - last < 60_000) return;
      cooldownMap.set(event.ruleId, now);

      try {
        const n = new Notification(i18n.t("Aidaguard — 敏感数据检测"), {
          body: i18n.t("规则: {{ruleId}}  |  策略: {{strategy}}  |  路径: {{requestPath}}", {
            ruleId: event.ruleId,
            strategy: event.strategy,
            requestPath: event.requestPath,
          }),
        });
        n.onclick = () => {
          window.focus();
          navigateRef.current("/audit");
        };
      } catch {
        // Notification API not available
      }
    }).then((un) => {
      unlisten = un;
    });

    return () => {
      unlisten?.();
    };
  }, []);
}
